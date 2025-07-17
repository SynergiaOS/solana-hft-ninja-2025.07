#!/usr/bin/env python3
"""
LangGraph Logic Flow for Project Cerebro
Implements the thinking flow: PLAN → ACT → OBSERVE → REMEMBER → REPEAT/FINISH
"""

from typing import Dict, Any, List, Optional, TypedDict, Annotated
from enum import Enum
import json
import time
from datetime import datetime
from dataclasses import dataclass

from langgraph.graph import StateGraph, END
from langgraph.prebuilt import ToolExecutor
from langchain_core.messages import BaseMessage, HumanMessage, AIMessage, SystemMessage
from langchain_core.tools import BaseTool

class AgentState(TypedDict):
    """State of the Cerebro agent during execution"""
    messages: Annotated[List[BaseMessage], "The conversation messages"]
    user_query: str
    current_plan: Optional[str]
    actions_taken: List[Dict[str, Any]]
    observations: List[Dict[str, Any]]
    memory_context: List[Dict[str, Any]]
    iteration_count: int
    max_iterations: int
    should_continue: bool
    final_response: Optional[str]
    execution_metadata: Dict[str, Any]

class FlowStage(Enum):
    """Stages in the agent flow"""
    PLAN = "plan"
    ACT = "act"
    OBSERVE = "observe"
    REMEMBER = "remember"
    DECIDE = "decide"
    FINISH = "finish"

@dataclass
class ActionResult:
    """Result of an action execution"""
    success: bool
    data: Any
    error: Optional[str] = None
    execution_time: float = 0.0
    metadata: Dict[str, Any] = None

class CerebroLangGraphFlow:
    """
    Main LangGraph flow implementation for Cerebro AI agent
    """
    
    def __init__(self, tools: List[BaseTool], llm, memory_manager, max_iterations: int = 5):
        self.tools = tools
        self.llm = llm
        self.memory_manager = memory_manager
        self.max_iterations = max_iterations
        self.tool_executor = ToolExecutor(tools)
        
        # Build the graph
        self.graph = self._build_graph()
    
    def _build_graph(self) -> StateGraph:
        """Build the LangGraph state graph"""
        workflow = StateGraph(AgentState)
        
        # Add nodes
        workflow.add_node("plan", self._plan_node)
        workflow.add_node("act", self._act_node)
        workflow.add_node("observe", self._observe_node)
        workflow.add_node("remember", self._remember_node)
        workflow.add_node("decide", self._decide_node)
        workflow.add_node("finish", self._finish_node)
        
        # Set entry point
        workflow.set_entry_point("plan")
        
        # Add edges
        workflow.add_edge("plan", "act")
        workflow.add_edge("act", "observe")
        workflow.add_edge("observe", "remember")
        workflow.add_edge("remember", "decide")
        
        # Conditional edges from decide
        workflow.add_conditional_edges(
            "decide",
            self._should_continue,
            {
                "continue": "plan",
                "finish": "finish"
            }
        )
        
        workflow.add_edge("finish", END)
        
        return workflow.compile()
    
    def _plan_node(self, state: AgentState) -> AgentState:
        """PLAN: Analyze the query and create an action plan"""
        print(f"🧠 PLANNING (Iteration {state['iteration_count'] + 1})")
        
        # Get relevant context from memory
        memory_context = self.memory_manager.search_relevant_context(
            state["user_query"], 
            limit=5
        )
        
        # Create planning prompt
        planning_prompt = self._create_planning_prompt(
            state["user_query"],
            memory_context,
            state["actions_taken"],
            state["observations"]
        )
        
        # Get plan from LLM
        messages = [SystemMessage(content=planning_prompt)]
        response = self.llm.invoke(messages)
        
        # Update state
        state["current_plan"] = response.content
        state["memory_context"] = memory_context
        state["messages"].append(AIMessage(content=f"Plan: {response.content}"))
        
        print(f"📋 Plan: {response.content[:100]}...")
        return state
    
    def _act_node(self, state: AgentState) -> AgentState:
        """ACT: Execute actions based on the current plan"""
        print("⚡ ACTING")
        
        # Parse plan to extract actions
        actions = self._parse_plan_for_actions(state["current_plan"])
        
        action_results = []
        for action in actions:
            try:
                start_time = time.time()
                
                # Execute action using tools
                result = self.tool_executor.invoke({
                    "tool": action["tool"],
                    "tool_input": action["input"]
                })
                
                execution_time = time.time() - start_time
                
                action_result = ActionResult(
                    success=True,
                    data=result,
                    execution_time=execution_time,
                    metadata={"action": action}
                )
                
                print(f"✅ Action completed: {action['tool']}")
                
            except Exception as e:
                action_result = ActionResult(
                    success=False,
                    data=None,
                    error=str(e),
                    metadata={"action": action}
                )
                print(f"❌ Action failed: {action['tool']} - {e}")
            
            action_results.append(action_result)
        
        # Update state
        state["actions_taken"].extend([
            {
                "plan": state["current_plan"],
                "results": action_results,
                "timestamp": datetime.now().isoformat()
            }
        ])
        
        return state
    
    def _observe_node(self, state: AgentState) -> AgentState:
        """OBSERVE: Analyze the results of actions"""
        print("👁️ OBSERVING")
        
        latest_actions = state["actions_taken"][-1] if state["actions_taken"] else None
        
        if latest_actions:
            # Analyze action results
            observations = self._analyze_action_results(latest_actions["results"])
            
            # Create observation summary
            observation_summary = self._create_observation_summary(observations)
            
            state["observations"].append({
                "summary": observation_summary,
                "details": observations,
                "timestamp": datetime.now().isoformat()
            })
            
            print(f"📊 Observed: {observation_summary[:100]}...")
        
        return state
    
    def _remember_node(self, state: AgentState) -> AgentState:
        """REMEMBER: Store important information in memory"""
        print("🧠 REMEMBERING")
        
        # Extract key insights from current iteration
        insights = self._extract_insights(
            state["current_plan"],
            state["actions_taken"][-1] if state["actions_taken"] else None,
            state["observations"][-1] if state["observations"] else None
        )
        
        # Store in memory
        for insight in insights:
            self.memory_manager.store_context(
                content=insight["content"],
                context_type=insight["type"],
                metadata={
                    "source": "cerebro_agent",
                    "query": state["user_query"],
                    "iteration": state["iteration_count"],
                    "timestamp": datetime.now().isoformat()
                }
            )
        
        print(f"💾 Stored {len(insights)} insights in memory")
        return state
    
    def _decide_node(self, state: AgentState) -> AgentState:
        """DECIDE: Determine if we should continue or finish"""
        print("🤔 DECIDING")
        
        state["iteration_count"] += 1
        
        # Check if we should continue
        should_continue = self._evaluate_continuation(state)
        state["should_continue"] = should_continue
        
        if should_continue:
            print(f"🔄 Continuing to iteration {state['iteration_count'] + 1}")
        else:
            print("🏁 Ready to finish")
        
        return state
    
    def _finish_node(self, state: AgentState) -> AgentState:
        """FINISH: Generate final response"""
        print("🎯 FINISHING")
        
        # Generate comprehensive response
        final_response = self._generate_final_response(state)
        state["final_response"] = final_response
        
        # Update execution metadata
        state["execution_metadata"].update({
            "completed_at": datetime.now().isoformat(),
            "total_iterations": state["iteration_count"],
            "total_actions": len(state["actions_taken"]),
            "total_observations": len(state["observations"])
        })
        
        print("✅ Response generated")
        return state
    
    def _should_continue(self, state: AgentState) -> str:
        """Conditional edge function"""
        return "continue" if state["should_continue"] else "finish"
    
    def _create_planning_prompt(self, query: str, memory_context: List, 
                               actions_taken: List, observations: List) -> str:
        """Create prompt for planning phase"""
        prompt = f"""
You are Cerebro, an AI assistant for Solana HFT trading analysis.

USER QUERY: {query}

RELEVANT MEMORY CONTEXT:
{json.dumps(memory_context, indent=2) if memory_context else "No relevant context found"}

PREVIOUS ACTIONS TAKEN:
{json.dumps(actions_taken[-3:], indent=2) if actions_taken else "No previous actions"}

PREVIOUS OBSERVATIONS:
{json.dumps(observations[-3:], indent=2) if observations else "No previous observations"}

Create a specific action plan to address the user's query. Consider:
1. What information do you need to gather?
2. What tools should you use?
3. What analysis should you perform?
4. How will you provide value to the user?

Respond with a clear, actionable plan.
"""
        return prompt
    
    def _parse_plan_for_actions(self, plan: str) -> List[Dict[str, Any]]:
        """Parse plan text to extract actionable items"""
        # Simple implementation - in production, use more sophisticated parsing
        actions = []
        
        if "get_hft_stats" in plan.lower():
            actions.append({
                "tool": "get_hft_stats",
                "input": {}
            })
        
        if "market" in plan.lower() or "sentiment" in plan.lower():
            actions.append({
                "tool": "get_market_sentiment",
                "input": {}
            })
        
        if "prometheus" in plan.lower() or "metrics" in plan.lower():
            actions.append({
                "tool": "query_prometheus",
                "input": {"query": "hft_profit_total"}
            })
        
        # Default action if no specific tools identified
        if not actions:
            actions.append({
                "tool": "search_memory",
                "input": {"query": plan[:100]}
            })
        
        return actions
    
    def _analyze_action_results(self, results: List[ActionResult]) -> List[Dict[str, Any]]:
        """Analyze the results of executed actions"""
        observations = []
        
        for result in results:
            if result.success:
                observations.append({
                    "type": "success",
                    "tool": result.metadata["action"]["tool"],
                    "data": result.data,
                    "execution_time": result.execution_time
                })
            else:
                observations.append({
                    "type": "error",
                    "tool": result.metadata["action"]["tool"],
                    "error": result.error
                })
        
        return observations
    
    def _create_observation_summary(self, observations: List[Dict[str, Any]]) -> str:
        """Create a summary of observations"""
        successful = len([obs for obs in observations if obs["type"] == "success"])
        failed = len([obs for obs in observations if obs["type"] == "error"])
        
        return f"Executed {len(observations)} actions: {successful} successful, {failed} failed"
    
    def _extract_insights(self, plan: str, actions: Dict, observations: Dict) -> List[Dict[str, Any]]:
        """Extract key insights to store in memory"""
        insights = []
        
        if actions and observations:
            insights.append({
                "type": "execution_result",
                "content": f"Plan: {plan[:200]}... Results: {observations['summary']}"
            })
        
        return insights
    
    def _evaluate_continuation(self, state: AgentState) -> bool:
        """Decide if we should continue iterating"""
        # Stop if max iterations reached
        if state["iteration_count"] >= state["max_iterations"]:
            return False
        
        # Stop if no actions were taken in last iteration
        if not state["actions_taken"]:
            return False
        
        # Stop if all recent actions were successful
        latest_actions = state["actions_taken"][-1]
        if latest_actions and all(r.success for r in latest_actions["results"]):
            return False
        
        return True
    
    def _generate_final_response(self, state: AgentState) -> str:
        """Generate the final response to the user"""
        # Create response prompt
        response_prompt = f"""
Based on the analysis performed, generate a comprehensive response to the user's query: "{state['user_query']}"

Actions taken: {len(state['actions_taken'])}
Observations made: {len(state['observations'])}
Iterations completed: {state['iteration_count']}

Provide a helpful, actionable response that addresses the user's needs.
"""
        
        messages = [SystemMessage(content=response_prompt)]
        response = self.llm.invoke(messages)
        
        return response.content
    
    async def execute(self, user_query: str) -> Dict[str, Any]:
        """Execute the full agent flow"""
        print(f"🚀 Starting Cerebro analysis for: {user_query}")
        
        # Initialize state
        initial_state = AgentState(
            messages=[HumanMessage(content=user_query)],
            user_query=user_query,
            current_plan=None,
            actions_taken=[],
            observations=[],
            memory_context=[],
            iteration_count=0,
            max_iterations=self.max_iterations,
            should_continue=True,
            final_response=None,
            execution_metadata={
                "started_at": datetime.now().isoformat(),
                "query": user_query
            }
        )
        
        # Execute the graph
        final_state = self.graph.invoke(initial_state)
        
        return {
            "response": final_state["final_response"],
            "metadata": final_state["execution_metadata"],
            "iterations": final_state["iteration_count"],
            "actions_count": len(final_state["actions_taken"]),
            "observations_count": len(final_state["observations"])
        }
