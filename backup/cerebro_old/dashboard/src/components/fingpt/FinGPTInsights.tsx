import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { 
  CpuChipIcon, 
  ChartBarIcon, 
  LightBulbIcon,
  ArrowPathIcon,
  SparklesIcon 
} from '@heroicons/react/24/outline';

const FinGPTInsights: React.FC = () => {
  const [isLoading, setIsLoading] = useState(false);

  const handleRefresh = async () => {
    setIsLoading(true);
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 2000));
    setIsLoading(false);
  };

  return (
    <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-2">
          <div className="w-8 h-8 bg-gradient-to-br from-purple-500 to-purple-700 rounded-lg flex items-center justify-center">
            <CpuChipIcon className="w-5 h-5 text-white" />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-white">FinGPT AI Insights</h3>
            <p className="text-xs text-gray-400">Powered by AI4Finance</p>
          </div>
        </div>
        <button
          onClick={handleRefresh}
          disabled={isLoading}
          className="p-2 rounded-lg hover:bg-[#2A2D3A] transition-colors disabled:opacity-50"
        >
          <ArrowPathIcon className={`w-4 h-4 text-gray-400 ${isLoading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* Market Sentiment */}
      <div className="mb-6">
        <div className="flex items-center space-x-2 mb-3">
          <ChartBarIcon className="w-4 h-4 text-purple-400" />
          <span className="text-sm font-medium text-white">Market Sentiment</span>
        </div>
        <div className="bg-[#0F1419] rounded-lg p-4 border border-[#2A2D3A]">
          <div className="flex items-center justify-between mb-2">
            <span className="text-green-400 font-semibold">Bullish</span>
            <span className="text-sm text-gray-400">Confidence: 78%</span>
          </div>
          <div className="w-full bg-[#2A2D3A] rounded-full h-2 mb-3">
            <motion.div 
              className="bg-green-400 h-2 rounded-full"
              initial={{ width: 0 }}
              animate={{ width: '78%' }}
              transition={{ duration: 1, delay: 0.5 }}
            />
          </div>
          <p className="text-sm text-gray-300">
            FinGPT analysis indicates positive market sentiment driven by increased DeFi activity and institutional adoption.
          </p>
        </div>
      </div>

      {/* Price Forecast */}
      <div className="mb-6">
        <div className="flex items-center space-x-2 mb-3">
          <SparklesIcon className="w-4 h-4 text-purple-400" />
          <span className="text-sm font-medium text-white">SOL Price Forecast</span>
        </div>
        <div className="bg-[#0F1419] rounded-lg p-4 border border-[#2A2D3A]">
          <div className="flex items-center justify-between mb-2">
            <div>
              <span className="text-lg font-bold text-white">$105-$110</span>
              <span className="text-sm text-gray-400 ml-2">(7 days)</span>
            </div>
            <span className="text-green-400 text-sm font-medium">↗ Upward</span>
          </div>
          <p className="text-sm text-gray-300">
            FinGPT forecaster predicts upward price movement based on technical indicators and market dynamics.
          </p>
        </div>
      </div>

      {/* Trading Recommendations */}
      <div className="mb-6">
        <div className="flex items-center space-x-2 mb-3">
          <LightBulbIcon className="w-4 h-4 text-purple-400" />
          <span className="text-sm font-medium text-white">AI Recommendations</span>
        </div>
        <div className="space-y-2">
          <div className="bg-[#0F1419] rounded-lg p-3 border border-[#2A2D3A]">
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-green-400 rounded-full"></div>
              <span className="text-sm text-white">Increase sandwich strategy allocation</span>
            </div>
          </div>
          <div className="bg-[#0F1419] rounded-lg p-3 border border-[#2A2D3A]">
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-yellow-400 rounded-full"></div>
              <span className="text-sm text-white">Monitor arbitrage opportunities on Raydium</span>
            </div>
          </div>
          <div className="bg-[#0F1419] rounded-lg p-3 border border-[#2A2D3A]">
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-blue-400 rounded-full"></div>
              <span className="text-sm text-white">Optimize gas fees for better profitability</span>
            </div>
          </div>
        </div>
      </div>

      {/* Chat with FinGPT */}
      <button className="w-full flex items-center justify-center space-x-2 p-3 bg-gradient-to-r from-purple-600 to-purple-700 hover:from-purple-700 hover:to-purple-800 text-white rounded-lg font-medium transition-all duration-200">
        <CpuChipIcon className="w-4 h-4" />
        <span>Ask FinGPT AI</span>
      </button>
    </div>
  );
};

export default FinGPTInsights;
