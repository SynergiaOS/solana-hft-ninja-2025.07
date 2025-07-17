import React from 'react';
import { motion } from 'framer-motion';

// Components
import MetricCard from '@/components/ui/MetricCard';
import TradingChart from '@/components/charts/TradingChart';
import StrategyCard from '@/components/trading/StrategyCard';
import RecentTrades from '@/components/trading/RecentTrades';
import FinGPTInsights from '@/components/fingpt/FinGPTInsights';
import TokenBalances from '@/web3/components/TokenBalances';

// Hooks
import { useTradingMetrics } from '@/hooks/useTradingMetrics';
import { useStrategies } from '@/hooks/useStrategies';

const OverviewPage: React.FC = () => {
  const { metrics, isLoading: metricsLoading } = useTradingMetrics();
  const { strategies, isLoading: strategiesLoading } = useStrategies();

  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        staggerChildren: 0.1,
      },
    },
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    visible: { opacity: 1, y: 0 },
  };

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      className="space-y-6"
    >
      {/* Page Header */}
      <motion.div variants={itemVariants} className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white">Trading Dashboard</h1>
          <p className="text-gray-400 mt-1">
            Monitor your HFT strategies and AI-powered insights
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <div className="flex items-center space-x-2 px-4 py-2 bg-[#1A1D29] rounded-lg border border-[#2A2D3A]">
            <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
            <span className="text-sm text-green-400 font-medium">Live Trading</span>
          </div>
          <button className="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-medium transition-colors">
            New Strategy
          </button>
        </div>
      </motion.div>

      {/* Key Metrics */}
      <motion.div variants={itemVariants}>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <MetricCard
            title="Total P&L"
            value="$47,892.34"
            change="+12.34%"
            changeType="positive"
            icon="💰"
            loading={metricsLoading}
          />
          <MetricCard
            title="24h P&L"
            value="$1,247.89"
            change="+5.67%"
            changeType="positive"
            icon="📈"
            loading={metricsLoading}
          />
          <MetricCard
            title="Success Rate"
            value="87.3%"
            change="+2.1%"
            changeType="positive"
            icon="🎯"
            loading={metricsLoading}
          />
          <MetricCard
            title="Active Trades"
            value="12"
            change="3 new"
            changeType="neutral"
            icon="⚡"
            loading={metricsLoading}
          />
        </div>
      </motion.div>

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Column - Charts and Trading */}
        <div className="lg:col-span-2 space-y-6">
          {/* Trading Performance Chart */}
          <motion.div variants={itemVariants}>
            <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
              <div className="flex items-center justify-between mb-6">
                <div>
                  <h2 className="text-xl font-semibold text-white">Portfolio Performance</h2>
                  <p className="text-gray-400 text-sm">Last 30 days trading activity</p>
                </div>
                <div className="flex items-center space-x-2">
                  <button className="px-3 py-1 text-xs bg-purple-600 text-white rounded-lg">24H</button>
                  <button className="px-3 py-1 text-xs text-gray-400 hover:text-white">7D</button>
                  <button className="px-3 py-1 text-xs text-gray-400 hover:text-white">30D</button>
                </div>
              </div>
              <TradingChart />
            </div>
          </motion.div>

          {/* Top Strategies */}
          <motion.div variants={itemVariants}>
            <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-xl font-semibold text-white">Top Performing Strategies</h2>
                <button className="text-purple-400 hover:text-purple-300 text-sm font-medium">
                  View All →
                </button>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <StrategyCard
                  name="Sandwich Strategy"
                  type="sandwich"
                  performance="+23.45%"
                  trades={156}
                  status="active"
                  loading={strategiesLoading}
                />
                <StrategyCard
                  name="Arbitrage Bot"
                  type="arbitrage"
                  performance="+18.92%"
                  trades={89}
                  status="active"
                  loading={strategiesLoading}
                />
                <StrategyCard
                  name="Liquidation Hunter"
                  type="liquidation"
                  performance="+15.67%"
                  trades={34}
                  status="paused"
                  loading={strategiesLoading}
                />
                <StrategyCard
                  name="Market Maker"
                  type="market_making"
                  performance="+12.34%"
                  trades={267}
                  status="active"
                  loading={strategiesLoading}
                />
              </div>
            </div>
          </motion.div>

          {/* Recent Trades */}
          <motion.div variants={itemVariants}>
            <RecentTrades />
          </motion.div>
        </div>

        {/* Right Column - AI Insights and Quick Actions */}
        <div className="space-y-6">
          {/* FinGPT AI Insights */}
          <motion.div variants={itemVariants}>
            <FinGPTInsights />
          </motion.div>

          {/* Token Balances */}
          <motion.div variants={itemVariants}>
            <TokenBalances />
          </motion.div>

          {/* Quick Actions */}
          <motion.div variants={itemVariants}>
            <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
              <h3 className="text-lg font-semibold text-white mb-4">Quick Actions</h3>
              <div className="space-y-3">
                <button className="w-full flex items-center justify-between p-3 bg-[#0F1419] hover:bg-[#2A2D3A] rounded-lg transition-colors group">
                  <div className="flex items-center space-x-3">
                    <div className="w-8 h-8 bg-purple-600 rounded-lg flex items-center justify-center">
                      <span className="text-white text-sm">🤖</span>
                    </div>
                    <span className="text-white font-medium">Ask Cerebro AI</span>
                  </div>
                  <span className="text-gray-400 group-hover:text-white">→</span>
                </button>
                
                <button className="w-full flex items-center justify-between p-3 bg-[#0F1419] hover:bg-[#2A2D3A] rounded-lg transition-colors group">
                  <div className="flex items-center space-x-3">
                    <div className="w-8 h-8 bg-green-600 rounded-lg flex items-center justify-center">
                      <span className="text-white text-sm">⚡</span>
                    </div>
                    <span className="text-white font-medium">Quick Trade</span>
                  </div>
                  <span className="text-gray-400 group-hover:text-white">→</span>
                </button>
                
                <button className="w-full flex items-center justify-between p-3 bg-[#0F1419] hover:bg-[#2A2D3A] rounded-lg transition-colors group">
                  <div className="flex items-center space-x-3">
                    <div className="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center">
                      <span className="text-white text-sm">📊</span>
                    </div>
                    <span className="text-white font-medium">Market Analysis</span>
                  </div>
                  <span className="text-gray-400 group-hover:text-white">→</span>
                </button>
                
                <button className="w-full flex items-center justify-between p-3 bg-[#0F1419] hover:bg-[#2A2D3A] rounded-lg transition-colors group">
                  <div className="flex items-center space-x-3">
                    <div className="w-8 h-8 bg-orange-600 rounded-lg flex items-center justify-center">
                      <span className="text-white text-sm">⚙️</span>
                    </div>
                    <span className="text-white font-medium">Strategy Builder</span>
                  </div>
                  <span className="text-gray-400 group-hover:text-white">→</span>
                </button>
              </div>
            </div>
          </motion.div>

          {/* System Health */}
          <motion.div variants={itemVariants}>
            <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
              <h3 className="text-lg font-semibold text-white mb-4">System Health</h3>
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-gray-400">API Status</span>
                  <div className="flex items-center space-x-2">
                    <div className="w-2 h-2 bg-green-400 rounded-full"></div>
                    <span className="text-green-400 text-sm">Healthy</span>
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-gray-400">DragonflyDB</span>
                  <div className="flex items-center space-x-2">
                    <div className="w-2 h-2 bg-green-400 rounded-full"></div>
                    <span className="text-green-400 text-sm">Connected</span>
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-gray-400">FinGPT Models</span>
                  <div className="flex items-center space-x-2">
                    <div className="w-2 h-2 bg-green-400 rounded-full"></div>
                    <span className="text-green-400 text-sm">Active</span>
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-gray-400">Trading Engine</span>
                  <div className="flex items-center space-x-2">
                    <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
                    <span className="text-green-400 text-sm">Running</span>
                  </div>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </div>
    </motion.div>
  );
};

export default OverviewPage;
