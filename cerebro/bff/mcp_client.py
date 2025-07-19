#!/usr/bin/env python3
"""
MCP (Machine-readable Cooperative Protocol) Client for Cerebro
Enables integration with external MCP servers and n8n workflows
"""

import asyncio
import json
import logging
from typing import Dict, Any, Optional, List
from datetime import datetime
import httpx
from pydantic import BaseModel, Field

logger = logging.getLogger(__name__)

class MCPTool(BaseModel):
    """MCP Tool definition"""
    name: str
    description: str
    parameters: Dict[str, Any]
    endpoint: str
    method: str = "GET"

class MCPServer(BaseModel):
    """MCP Server configuration"""
    name: str
    base_url: str
    tools: List[MCPTool]
    authentication: Optional[Dict[str, Any]] = None
    rate_limit: Optional[int] = 60  # requests per minute

class MCPClient:
    """
    MCP Client for communicating with external MCP servers
    Supports n8n workflows, external AI services, and other MCP-compatible tools
    """
    
    def __init__(self):
        self.servers: Dict[str, MCPServer] = {}
        self.http_client = httpx.AsyncClient(timeout=30.0)
        self.rate_limiters: Dict[str, Dict] = {}
        
    async def register_server(self, server: MCPServer):
        """Register an MCP server"""
        self.servers[server.name] = server
        self.rate_limiters[server.name] = {
            "requests": [],
            "limit": server.rate_limit or 60
        }
        logger.info(f"Registered MCP server: {server.name} with {len(server.tools)} tools")
    
    async def discover_tools(self, server_name: str) -> List[MCPTool]:
        """Discover available tools from an MCP server"""
        if server_name not in self.servers:
            raise ValueError(f"Server {server_name} not registered")
        
        server = self.servers[server_name]
        try:
            response = await self.http_client.get(f"{server.base_url}/tools")
            if response.status_code == 200:
                tools_data = response.json()
                tools = [MCPTool(**tool) for tool in tools_data.get("tools", [])]
                server.tools = tools
                logger.info(f"Discovered {len(tools)} tools from {server_name}")
                return tools
        except Exception as e:
            logger.error(f"Failed to discover tools from {server_name}: {e}")
            return server.tools
    
    async def call_tool(self, server_name: str, tool_name: str, parameters: Dict[str, Any] = None) -> Dict[str, Any]:
        """Call a tool on an MCP server"""
        if server_name not in self.servers:
            raise ValueError(f"Server {server_name} not registered")
        
        server = self.servers[server_name]
        tool = next((t for t in server.tools if t.name == tool_name), None)
        
        if not tool:
            raise ValueError(f"Tool {tool_name} not found on server {server_name}")
        
        # Check rate limiting
        if not await self._check_rate_limit(server_name):
            raise Exception(f"Rate limit exceeded for server {server_name}")
        
        # Prepare request
        url = f"{server.base_url}{tool.endpoint}"
        headers = {"Content-Type": "application/json"}
        
        # Add authentication if configured
        if server.authentication:
            if server.authentication.get("type") == "bearer":
                token = server.authentication.get("token")
                headers["Authorization"] = f"Bearer {token}"
        
        try:
            if tool.method.upper() == "GET":
                response = await self.http_client.get(url, headers=headers, params=parameters or {})
            else:
                response = await self.http_client.request(
                    tool.method.upper(),
                    url,
                    headers=headers,
                    json=parameters or {}
                )
            
            if response.status_code == 200:
                return response.json()
            else:
                logger.error(f"Tool call failed: {response.status_code} - {response.text}")
                return {"error": f"HTTP {response.status_code}", "message": response.text}
                
        except Exception as e:
            logger.error(f"Error calling tool {tool_name} on {server_name}: {e}")
            return {"error": "request_failed", "message": str(e)}
    
    async def _check_rate_limit(self, server_name: str) -> bool:
        """Check if request is within rate limits"""
        now = datetime.now()
        rate_limiter = self.rate_limiters[server_name]
        
        # Remove requests older than 1 minute
        rate_limiter["requests"] = [
            req_time for req_time in rate_limiter["requests"]
            if (now - req_time).total_seconds() < 60
        ]
        
        # Check if under limit
        if len(rate_limiter["requests"]) >= rate_limiter["limit"]:
            return False
        
        # Add current request
        rate_limiter["requests"].append(now)
        return True
    
    async def get_available_tools(self) -> Dict[str, List[str]]:
        """Get all available tools across all servers"""
        tools_by_server = {}
        for server_name, server in self.servers.items():
            tools_by_server[server_name] = [tool.name for tool in server.tools]
        return tools_by_server
    
    async def close(self):
        """Close the HTTP client"""
        await self.http_client.aclose()

# Pre-configured MCP servers for common integrations
class CerebroMCPServers:
    """Pre-configured MCP servers for Cerebro ecosystem"""
    
    @staticmethod
    def n8n_server(base_url: str = "http://n8n:5678") -> MCPServer:
        """n8n workflow automation server"""
        return MCPServer(
            name="n8n_workflows",
            base_url=base_url,
            tools=[
                MCPTool(
                    name="trigger_workflow",
                    description="Trigger an n8n workflow",
                    parameters={
                        "type": "object",
                        "properties": {
                            "workflow_id": {"type": "string"},
                            "data": {"type": "object"}
                        },
                        "required": ["workflow_id"]
                    },
                    endpoint="/api/v1/workflows/{workflow_id}/activate",
                    method="POST"
                ),
                MCPTool(
                    name="get_workflow_status",
                    description="Get status of an n8n workflow",
                    parameters={
                        "type": "object",
                        "properties": {
                            "workflow_id": {"type": "string"}
                        },
                        "required": ["workflow_id"]
                    },
                    endpoint="/api/v1/workflows/{workflow_id}",
                    method="GET"
                )
            ],
            rate_limit=30
        )
    
    @staticmethod
    def brave_search_server() -> MCPServer:
        """Brave Search MCP server"""
        return MCPServer(
            name="brave_search",
            base_url="https://api.search.brave.com/res/v1",
            tools=[
                MCPTool(
                    name="web_search",
                    description="Search the web using Brave Search",
                    parameters={
                        "type": "object",
                        "properties": {
                            "q": {"type": "string", "description": "Search query"},
                            "count": {"type": "integer", "default": 10},
                            "offset": {"type": "integer", "default": 0}
                        },
                        "required": ["q"]
                    },
                    endpoint="/web/search",
                    method="GET"
                ),
                MCPTool(
                    name="news_search",
                    description="Search for news using Brave Search",
                    parameters={
                        "type": "object",
                        "properties": {
                            "q": {"type": "string", "description": "Search query"},
                            "count": {"type": "integer", "default": 10}
                        },
                        "required": ["q"]
                    },
                    endpoint="/news/search",
                    method="GET"
                )
            ],
            authentication={
                "type": "bearer",
                "token": "BSA_API_KEY"  # Should be set via environment
            },
            rate_limit=100
        )
    
    @staticmethod
    def helius_server() -> MCPServer:
        """Helius Solana RPC MCP server"""
        return MCPServer(
            name="helius_solana",
            base_url="https://api.helius.xyz/v0",
            tools=[
                MCPTool(
                    name="get_token_metadata",
                    description="Get Solana token metadata",
                    parameters={
                        "type": "object",
                        "properties": {
                            "mint_accounts": {"type": "array", "items": {"type": "string"}}
                        },
                        "required": ["mint_accounts"]
                    },
                    endpoint="/token-metadata",
                    method="GET"
                ),
                MCPTool(
                    name="get_nft_events",
                    description="Get NFT events from Solana",
                    parameters={
                        "type": "object",
                        "properties": {
                            "accounts": {"type": "array", "items": {"type": "string"}},
                            "types": {"type": "array", "items": {"type": "string"}}
                        },
                        "required": ["accounts"]
                    },
                    endpoint="/nft-events",
                    method="GET"
                )
            ],
            authentication={
                "type": "bearer",
                "token": "HELIUS_API_KEY"  # Should be set via environment
            },
            rate_limit=100
        )

# Factory function to create and configure MCP client
async def create_mcp_client() -> MCPClient:
    """Create and configure MCP client with default servers"""
    client = MCPClient()
    
    # Register default servers
    await client.register_server(CerebroMCPServers.n8n_server())
    await client.register_server(CerebroMCPServers.brave_search_server())
    await client.register_server(CerebroMCPServers.helius_server())
    
    return client

# Example usage functions
async def example_n8n_integration():
    """Example of using n8n via MCP"""
    client = await create_mcp_client()
    
    try:
        # Trigger status monitoring workflow
        result = await client.call_tool(
            "n8n_workflows",
            "trigger_workflow",
            {"workflow_id": "cerebro-status-monitor", "data": {}}
        )
        logger.info(f"Workflow triggered: {result}")
        
        # Get workflow status
        status = await client.call_tool(
            "n8n_workflows",
            "get_workflow_status",
            {"workflow_id": "cerebro-status-monitor"}
        )
        logger.info(f"Workflow status: {status}")
        
    finally:
        await client.close()

async def example_external_search():
    """Example of using external search via MCP"""
    client = await create_mcp_client()
    
    try:
        # Search for Solana news
        news = await client.call_tool(
            "brave_search",
            "news_search",
            {"q": "Solana blockchain news", "count": 5}
        )
        logger.info(f"Latest Solana news: {news}")
        
    finally:
        await client.close()

if __name__ == "__main__":
    # Test the MCP client
    asyncio.run(example_n8n_integration())
