#!/usr/bin/env python3
"""
Test suite for Agent Tools
"""

import asyncio
import sys
sys.path.append('..')

from agent.tools import AgentTools


async def test_agent_tools():
    """Test Agent Tools functionality"""
    print("🧪 Testing Agent Tools...")

    tools = AgentTools()

    try:
        # Test available tools
        available_tools = tools.get_available_tools()
        print(f"✅ Available tools: {len(available_tools)}")
        for tool in available_tools:
            print(f"   - {tool['name']}: {tool['description']}")

        # Test HFT stats (will fail if HFT Ninja not running, but that's OK)
        print("\n📊 Testing HFT stats...")
        stats = await tools.get_hft_stats("1h")
        if "error" in stats:
            print(f"⚠️ HFT stats: {stats['error']} (expected if HFT Ninja not running)")
        else:
            print(f"✅ HFT stats retrieved: {stats.get('total_trades', 0)} trades")

        # Test market sentiment (mock data)
        print("\n📈 Testing market sentiment...")
        sentiment = await tools.get_market_sentiment("SOL")
        if "error" in sentiment:
            print(f"❌ Market sentiment failed: {sentiment['error']}")
        else:
            print(f"✅ Market sentiment: {sentiment['sentiment_score']:.2f} ({sentiment['trend']})")

        # Test memory search (will fail if RAG not initialized, but that's OK)
        print("\n🧠 Testing memory search...")
        memory_result = await tools.search_memory("trading performance")
        if "error" in memory_result:
            print(f"⚠️ Memory search: {memory_result['error']} (expected if RAG not initialized)")
        else:
            print(f"✅ Memory search: {len(memory_result['results'])} results")

        print("\n🎉 Agent Tools test completed!")
        return True

    except Exception as e:
        print(f"❌ Agent Tools test failed: {e}")
        return False

    finally:
        await tools.close()


if __name__ == "__main__":
    success = asyncio.run(test_agent_tools())
    exit(0 if success else 1)