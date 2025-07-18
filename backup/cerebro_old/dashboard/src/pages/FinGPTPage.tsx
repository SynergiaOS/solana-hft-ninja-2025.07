import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { motion } from 'framer-motion';
import { apiClient } from '@/services/api';

// Icons
import {
  SparklesIcon,
  ChatBubbleLeftRightIcon,
  ChartBarIcon,
  LightBulbIcon,
  CpuChipIcon,
  ArrowPathIcon
} from '@heroicons/react/24/outline';

const FinGPTPage: React.FC = () => {
  const [chatInput, setChatInput] = useState('');
  const [chatHistory, setChatHistory] = useState([
    {
      type: 'ai',
      message: 'Hello! I\'m FinGPT, your AI trading assistant. I can analyze market data, suggest strategies, and provide insights. How can I help you today?',
      timestamp: new Date().toISOString()
    }
  ]);

  // Fetch FinGPT models info
  const { data: fingptModels, isLoading } = useQuery({
    queryKey: ['fingpt-models'],
    queryFn: () => apiClient.get('/api/fingpt/models'),
    refetchInterval: 60000,
  });

  // Fetch trading insights
  const { data: insights } = useQuery({
    queryKey: ['trading-insights'],
    queryFn: () => apiClient.get('/api/fingpt/insights'),
    refetchInterval: 30000,
  });

  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: { staggerChildren: 0.1 },
    },
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    visible: { opacity: 1, y: 0 },
  };

  const handleSendMessage = () => {
    if (!chatInput.trim()) return;

    // Add user message
    const userMessage = {
      type: 'user',
      message: chatInput,
      timestamp: new Date().toISOString()
    };

    // Simulate AI response
    const aiResponses = [
      "Based on current market data, I see increased volatility in SOL/USDC pair. Consider adjusting your sandwich strategy parameters.",
      "The arbitrage opportunities between Raydium and Orca have increased by 15% in the last hour. Your strategy is well-positioned.",
      "I notice your success rate has improved to 89.2%. The recent optimizations to latency are working well.",
      "Market sentiment analysis shows bullish trends for meme tokens. Consider increasing allocation to sniping strategy.",
      "Risk analysis: Current drawdown is within acceptable limits. Portfolio diversification looks optimal."
    ];

    const aiMessage = {
      type: 'ai',
      message: aiResponses[Math.floor(Math.random() * aiResponses.length)],
      timestamp: new Date().toISOString()
    };

    setChatHistory(prev => [...prev, userMessage, aiMessage]);
    setChatInput('');
  };

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold text-white">FinGPT AI</h1>
          <p className="text-gray-400 mt-1">AI-powered financial analysis and insights</p>
        </div>
        <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-8 text-center">
          <ArrowPathIcon className="h-8 w-8 text-blue-400 mx-auto mb-4 animate-spin" />
          <p className="text-gray-400">Loading FinGPT models...</p>
        </div>
      </div>
    );
  }

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      className="space-y-6"
    >
      {/* Page Header */}
      <motion.div variants={itemVariants}>
        <h1 className="text-3xl font-bold text-white">FinGPT AI</h1>
        <p className="text-gray-400 mt-1">AI-powered financial analysis and insights</p>
      </motion.div>

      {/* FinGPT Models Status */}
      <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
        <h2 className="text-xl font-semibold text-white mb-6 flex items-center">
          <CpuChipIcon className="h-6 w-6 mr-2 text-blue-400" />
          Active FinGPT Models
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {fingptModels?.models?.map((model: any, index: number) => (
            <div key={index} className="bg-[#0F1419] rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-white font-medium">{model.name}</h3>
                <span className="text-xs px-2 py-1 rounded-full bg-green-900 text-green-300">
                  Active
                </span>
              </div>
              <p className="text-gray-400 text-sm mb-2">{model.description}</p>
              <div className="flex items-center justify-between text-sm">
                <span className="text-gray-400">Performance:</span>
                <span className="text-blue-400">{(model.performance?.multi_task_score * 100).toFixed(1)}%</span>
              </div>
            </div>
          ))}
        </div>
      </motion.div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* AI Chat Interface */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <h2 className="text-xl font-semibold text-white mb-6 flex items-center">
            <ChatBubbleLeftRightIcon className="h-6 w-6 mr-2 text-green-400" />
            AI Assistant
          </h2>

          {/* Chat History */}
          <div className="h-80 overflow-y-auto mb-4 space-y-3">
            {chatHistory.map((message, index) => (
              <div key={index} className={`flex ${message.type === 'user' ? 'justify-end' : 'justify-start'}`}>
                <div className={`max-w-xs lg:max-w-md px-4 py-2 rounded-lg ${
                  message.type === 'user'
                    ? 'bg-blue-600 text-white'
                    : 'bg-[#0F1419] text-gray-300'
                }`}>
                  <p className="text-sm">{message.message}</p>
                  <p className="text-xs opacity-70 mt-1">
                    {new Date(message.timestamp).toLocaleTimeString()}
                  </p>
                </div>
              </div>
            ))}
          </div>

          {/* Chat Input */}
          <div className="flex space-x-2">
            <input
              type="text"
              value={chatInput}
              onChange={(e) => setChatInput(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleSendMessage()}
              placeholder="Ask FinGPT about trading strategies..."
              className="flex-1 bg-[#0F1419] border border-[#2A2D3A] rounded-lg px-4 py-2 text-white placeholder-gray-400 focus:outline-none focus:border-blue-400"
            />
            <button
              onClick={handleSendMessage}
              className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition-colors"
            >
              Send
            </button>
          </div>
        </motion.div>

        {/* AI Insights */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <h2 className="text-xl font-semibold text-white mb-6 flex items-center">
            <LightBulbIcon className="h-6 w-6 mr-2 text-yellow-400" />
            AI Insights
          </h2>

          <div className="space-y-4">
            <div className="bg-[#0F1419] rounded-lg p-4">
              <div className="flex items-center mb-2">
                <SparklesIcon className="h-5 w-5 text-yellow-400 mr-2" />
                <h3 className="text-white font-medium">Market Analysis</h3>
              </div>
              <p className="text-gray-400 text-sm">
                Current market conditions favor arbitrage strategies. Detected 23% increase in cross-DEX price differences.
              </p>
              <div className="mt-2">
                <span className="text-xs px-2 py-1 rounded-full bg-yellow-900 text-yellow-300">
                  High Confidence
                </span>
              </div>
            </div>

            <div className="bg-[#0F1419] rounded-lg p-4">
              <div className="flex items-center mb-2">
                <ChartBarIcon className="h-5 w-5 text-blue-400 mr-2" />
                <h3 className="text-white font-medium">Strategy Optimization</h3>
              </div>
              <p className="text-gray-400 text-sm">
                Sandwich strategy latency can be improved by 12ms with adjusted gas parameters.
              </p>
              <div className="mt-2">
                <span className="text-xs px-2 py-1 rounded-full bg-blue-900 text-blue-300">
                  Actionable
                </span>
              </div>
            </div>

            <div className="bg-[#0F1419] rounded-lg p-4">
              <div className="flex items-center mb-2">
                <SparklesIcon className="h-5 w-5 text-green-400 mr-2" />
                <h3 className="text-white font-medium">Risk Assessment</h3>
              </div>
              <p className="text-gray-400 text-sm">
                Portfolio risk is within optimal range. Consider increasing position size by 15%.
              </p>
              <div className="mt-2">
                <span className="text-xs px-2 py-1 rounded-full bg-green-900 text-green-300">
                  Low Risk
                </span>
              </div>
            </div>
          </div>
        </motion.div>
      </div>

      {/* Quick Actions */}
      <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
        <h2 className="text-xl font-semibold text-white mb-6">Quick AI Actions</h2>
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <button className="bg-[#0F1419] hover:bg-[#1A1D29] border border-[#2A2D3A] rounded-lg p-4 text-left transition-colors">
            <SparklesIcon className="h-6 w-6 text-blue-400 mb-2" />
            <h3 className="text-white font-medium mb-1">Analyze Market</h3>
            <p className="text-gray-400 text-sm">Get current market insights</p>
          </button>

          <button className="bg-[#0F1419] hover:bg-[#1A1D29] border border-[#2A2D3A] rounded-lg p-4 text-left transition-colors">
            <ChartBarIcon className="h-6 w-6 text-green-400 mb-2" />
            <h3 className="text-white font-medium mb-1">Optimize Strategies</h3>
            <p className="text-gray-400 text-sm">AI-powered optimization</p>
          </button>

          <button className="bg-[#0F1419] hover:bg-[#1A1D29] border border-[#2A2D3A] rounded-lg p-4 text-left transition-colors">
            <LightBulbIcon className="h-6 w-6 text-yellow-400 mb-2" />
            <h3 className="text-white font-medium mb-1">Risk Analysis</h3>
            <p className="text-gray-400 text-sm">Comprehensive risk assessment</p>
          </button>

          <button className="bg-[#0F1419] hover:bg-[#1A1D29] border border-[#2A2D3A] rounded-lg p-4 text-left transition-colors">
            <CpuChipIcon className="h-6 w-6 text-purple-400 mb-2" />
            <h3 className="text-white font-medium mb-1">Model Training</h3>
            <p className="text-gray-400 text-sm">Retrain with latest data</p>
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
};

export default FinGPTPage;
