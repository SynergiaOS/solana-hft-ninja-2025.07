#!/usr/bin/env python3
"""
Test suite for LLM Router
"""

import sys
sys.path.append('..')

from agent.llm_router import LLMRouter, ModelType, QueryIntent


def test_llm_router():
    """Test LLM Router functionality"""
    print("🧪 Testing LLM Router...")

    router = LLMRouter()

    # Test cases
    test_cases = [
        {
            "query": "Calculate the Sharpe ratio for my trading strategy",
            "expected_model": ModelType.DEEPSEEK_MATH,
            "expected_intent": QueryIntent.QUANTITATIVE_ANALYSIS
        },
        {
            "query": "Why am I losing money on my arbitrage strategy?",
            "expected_model": ModelType.FINGPT,
            "expected_intent": QueryIntent.FINANCIAL_ANALYSIS
        },
        {
            "query": "How can I optimize my sandwich attack parameters?",
            "expected_model": ModelType.FINGPT,
            "expected_intent": QueryIntent.STRATEGY_OPTIMIZATION
        },
        {
            "query": "What's the risk of my current position size?",
            "expected_model": ModelType.FINGPT,
            "expected_intent": QueryIntent.RISK_ASSESSMENT
        },
        {
            "query": "Analyze current market sentiment for SOL",
            "expected_model": ModelType.FINGPT,
            "expected_intent": QueryIntent.MARKET_ANALYSIS
        },
        {
            "query": "Show me my performance report for last week",
            "expected_model": ModelType.FINGPT,
            "expected_intent": QueryIntent.PERFORMANCE_REVIEW
        },
        {
            "query": "My bot crashed with a timeout error",
            "expected_model": ModelType.DEEPSEEK_MATH,
            "expected_intent": QueryIntent.TECHNICAL_ISSUE
        },
        {
            "query": "Hello, how are you?",
            "expected_model": ModelType.FINGPT,
            "expected_intent": QueryIntent.GENERAL_QUESTION
        }
    ]

    passed = 0
    total = len(test_cases)

    for i, test_case in enumerate(test_cases, 1):
        print(f"\n📝 Test {i}: {test_case['query']}")

        decision = router.route_query(test_case['query'])

        print(f"   Model: {decision.model_type.value}")
        print(f"   Intent: {decision.intent.value}")
        print(f"   Confidence: {decision.confidence:.2f}")
        print(f"   Reasoning: {decision.reasoning}")

        # Check if routing is correct
        model_correct = decision.model_type == test_case['expected_model']
        intent_correct = decision.intent == test_case['expected_intent']

        if model_correct and intent_correct:
            print("   ✅ PASS")
            passed += 1
        else:
            print("   ❌ FAIL")
            if not model_correct:
                print(f"      Expected model: {test_case['expected_model'].value}")
            if not intent_correct:
                print(f"      Expected intent: {test_case['expected_intent'].value}")

    print(f"\n📊 Results: {passed}/{total} tests passed")

    if passed == total:
        print("🎉 All LLM Router tests passed!")
        return True
    else:
        print("⚠️ Some tests failed. Router may need tuning.")
        return False


def test_prompt_optimization():
    """Test prompt optimization"""
    print("\n🧪 Testing Prompt Optimization...")

    router = LLMRouter()

    query = "Calculate my portfolio's risk-adjusted returns"
    context = {
        "hft_stats": {"total_trades": 150, "profit_sol": 0.5},
        "market_conditions": "volatile"
    }

    decision = router.route_query(query, context)

    print(f"Original query: {query}")
    print(f"Optimized prompt:\n{decision.suggested_prompt}")

    # Check if context was included
    if "Current HFT Performance" in decision.suggested_prompt:
        print("✅ Context integration: PASS")
        return True
    else:
        print("❌ Context integration: FAIL")
        return False


if __name__ == "__main__":
    success1 = test_llm_router()
    success2 = test_prompt_optimization()

    if success1 and success2:
        print("\n🎉 All LLM Router tests completed successfully!")
        exit(0)
    else:
        print("\n⚠️ Some tests failed.")
        exit(1)