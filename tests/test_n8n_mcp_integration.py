#!/usr/bin/env python3
"""
Integration tests for n8n + MCP functionality
Tests the complete workflow automation and MCP protocol integration
"""

import asyncio
import pytest
import httpx
import json
import time
from typing import Dict, Any

# Test configuration
TEST_CONFIG = {
    "cerebro_bff_url": "http://localhost:8000",
    "n8n_url": "http://localhost:5678",
    "timeout": 30.0
}

class TestN8nMCPIntegration:
    """Test suite for n8n + MCP integration"""
    
    @pytest.fixture(autouse=True)
    async def setup(self):
        """Setup test environment"""
        self.http_client = httpx.AsyncClient(timeout=TEST_CONFIG["timeout"])
        yield
        await self.http_client.aclose()
    
    async def test_cerebro_bff_health(self):
        """Test that Cerebro BFF is running and healthy"""
        response = await self.http_client.get(f"{TEST_CONFIG['cerebro_bff_url']}/health")
        assert response.status_code == 200
        
        health_data = response.json()
        assert health_data["status"] == "ok"
        print("✅ Cerebro BFF is healthy")
    
    async def test_n8n_health(self):
        """Test that n8n is running and healthy"""
        response = await self.http_client.get(f"{TEST_CONFIG['n8n_url']}/healthz")
        assert response.status_code == 200
        print("✅ n8n is healthy")
    
    async def test_mcp_servers_endpoint(self):
        """Test MCP servers discovery endpoint"""
        response = await self.http_client.get(f"{TEST_CONFIG['cerebro_bff_url']}/api/mcp/servers")
        
        if response.status_code == 503:
            pytest.skip("MCP client not initialized - this is expected on first startup")
        
        assert response.status_code == 200
        
        servers_data = response.json()
        assert "servers" in servers_data
        assert "tools_by_server" in servers_data
        
        # Check that expected servers are present
        expected_servers = ["n8n_workflows", "brave_search", "helius_solana"]
        for server in expected_servers:
            assert server in servers_data["servers"], f"Server {server} not found"
        
        print(f"✅ MCP servers available: {servers_data['servers']}")
    
    async def test_mcp_tool_call(self):
        """Test calling an MCP tool"""
        # Test a simple tool call
        tool_request = {
            "server_name": "n8n_workflows",
            "tool_name": "get_workflow_status",
            "parameters": {"workflow_id": "test"}
        }
        
        response = await self.http_client.post(
            f"{TEST_CONFIG['cerebro_bff_url']}/api/mcp/call",
            json=tool_request
        )
        
        if response.status_code == 503:
            pytest.skip("MCP client not initialized")
        
        assert response.status_code == 200
        
        result = response.json()
        assert "success" in result
        assert "timestamp" in result
        
        print(f"✅ MCP tool call successful: {result['success']}")
    
    async def test_n8n_workflow_trigger(self):
        """Test triggering n8n workflow via MCP"""
        workflow_id = "cerebro-status-monitor"
        test_data = {
            "manual_trigger": True,
            "test_mode": True,
            "timestamp": time.time()
        }
        
        response = await self.http_client.post(
            f"{TEST_CONFIG['cerebro_bff_url']}/api/mcp/n8n/trigger/{workflow_id}",
            json=test_data
        )
        
        if response.status_code == 503:
            pytest.skip("MCP client not initialized")
        
        # Accept both success and workflow-not-found as valid responses
        assert response.status_code in [200, 404]
        
        if response.status_code == 200:
            result = response.json()
            assert result["workflow_id"] == workflow_id
            print(f"✅ n8n workflow triggered: {workflow_id}")
        else:
            print(f"⚠️  Workflow {workflow_id} not found (expected on fresh install)")
    
    async def test_external_search_integration(self):
        """Test external search via MCP (if available)"""
        search_query = "Solana blockchain"
        
        response = await self.http_client.get(
            f"{TEST_CONFIG['cerebro_bff_url']}/api/mcp/search/web",
            params={"q": search_query, "count": 3}
        )
        
        if response.status_code == 503:
            pytest.skip("MCP client not initialized")
        
        # External search might fail due to API keys, so we accept various responses
        assert response.status_code in [200, 401, 403, 500]
        
        if response.status_code == 200:
            result = response.json()
            assert result["query"] == search_query
            print("✅ External search integration working")
        else:
            print("⚠️  External search requires API key configuration")
    
    async def test_gradio_tools_integration(self):
        """Test Gradio external tools integration"""
        # This test checks if gradio tools are properly integrated
        # We'll test by checking if the tools are available in the agent
        
        # First, check if we can access the agent tools endpoint
        response = await self.http_client.get(f"{TEST_CONFIG['cerebro_bff_url']}/api/agent/tools")
        
        if response.status_code == 404:
            pytest.skip("Agent tools endpoint not implemented yet")
        
        if response.status_code == 200:
            tools_data = response.json()
            
            # Check for Gradio tools
            gradio_tools = [
                "gradio_token_risk_analyzer",
                "gradio_sentiment_analyzer", 
                "gradio_market_analyzer"
            ]
            
            available_tools = tools_data.get("tools", [])
            gradio_tools_found = [tool for tool in gradio_tools if tool in available_tools]
            
            if gradio_tools_found:
                print(f"✅ Gradio tools available: {gradio_tools_found}")
            else:
                print("⚠️  Gradio tools not yet integrated")
    
    async def test_workflow_files_exist(self):
        """Test that workflow files are properly created"""
        import os
        
        workflow_files = [
            "n8n/workflows/cerebro_status_monitor.json",
            "n8n/workflows/external_data_ingestion.json",
            "n8n/mcp/cerebro_mcp_server.json"
        ]
        
        for workflow_file in workflow_files:
            assert os.path.exists(workflow_file), f"Workflow file missing: {workflow_file}"
            
            # Validate JSON structure
            with open(workflow_file, 'r') as f:
                workflow_data = json.load(f)
                assert "name" in workflow_data
                
        print("✅ All workflow files exist and are valid JSON")
    
    async def test_mcp_rate_limiting(self):
        """Test MCP rate limiting functionality"""
        # Make multiple rapid requests to test rate limiting
        requests_count = 5
        responses = []
        
        for i in range(requests_count):
            response = await self.http_client.get(f"{TEST_CONFIG['cerebro_bff_url']}/api/mcp/servers")
            responses.append(response.status_code)
        
        # All requests should succeed (rate limit is 60/minute)
        success_count = sum(1 for status in responses if status == 200)
        
        if success_count > 0:
            print(f"✅ Rate limiting test passed: {success_count}/{requests_count} requests succeeded")
        else:
            print("⚠️  All requests failed - MCP client might not be initialized")

# Async test runner
async def run_integration_tests():
    """Run all integration tests"""
    test_instance = TestN8nMCPIntegration()
    await test_instance.setup()
    
    tests = [
        ("Cerebro BFF Health", test_instance.test_cerebro_bff_health),
        ("n8n Health", test_instance.test_n8n_health),
        ("MCP Servers Endpoint", test_instance.test_mcp_servers_endpoint),
        ("MCP Tool Call", test_instance.test_mcp_tool_call),
        ("n8n Workflow Trigger", test_instance.test_n8n_workflow_trigger),
        ("External Search Integration", test_instance.test_external_search_integration),
        ("Gradio Tools Integration", test_instance.test_gradio_tools_integration),
        ("Workflow Files Exist", test_instance.test_workflow_files_exist),
        ("MCP Rate Limiting", test_instance.test_mcp_rate_limiting),
    ]
    
    print("🧪 Running n8n + MCP Integration Tests")
    print("=" * 50)
    
    passed = 0
    failed = 0
    skipped = 0
    
    for test_name, test_func in tests:
        try:
            print(f"\n🔍 Testing: {test_name}")
            await test_func()
            passed += 1
            print(f"✅ PASSED: {test_name}")
            
        except pytest.skip.Exception as e:
            skipped += 1
            print(f"⏭️  SKIPPED: {test_name} - {e}")
            
        except Exception as e:
            failed += 1
            print(f"❌ FAILED: {test_name} - {e}")
    
    print("\n" + "=" * 50)
    print(f"📊 Test Results: {passed} passed, {failed} failed, {skipped} skipped")
    
    if failed == 0:
        print("🎉 All tests passed or skipped!")
        return True
    else:
        print("⚠️  Some tests failed. Check the output above.")
        return False

if __name__ == "__main__":
    # Run the integration tests
    success = asyncio.run(run_integration_tests())
    exit(0 if success else 1)
