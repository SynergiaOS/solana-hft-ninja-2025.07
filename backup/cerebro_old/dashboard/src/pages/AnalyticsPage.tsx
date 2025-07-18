import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { motion } from 'framer-motion';
import { apiClient } from '@/services/api';

// Icons
import {
  ChartBarIcon,
  ArrowTrendingUpIcon,
  ArrowTrendingDownIcon,
  CurrencyDollarIcon,
  ClockIcon,
  BoltIcon
} from '@heroicons/react/24/outline';

const AnalyticsPage: React.FC = () => {
  // Fetch trading history
  const { data: tradingHistory, isLoading: historyLoading } = useQuery({
    queryKey: ['trading-history'],
    queryFn: () => apiClient.get('/api/trading/history'),
    refetchInterval: 30000,
  });

  // Fetch strategies
  const { data: strategies, isLoading: strategiesLoading } = useQuery({
    queryKey: ['strategies'],
    queryFn: () => apiClient.get('/api/strategies'),
    refetchInterval: 30000,
  });

  // Fetch system metrics
  const { data: systemMetrics } = useQuery({
    queryKey: ['system-metrics'],
    queryFn: () => apiClient.get('/api/system/metrics'),
    refetchInterval: 10000,
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

  // Calculate analytics
  const totalProfit = strategies?.reduce((sum: number, strategy: any) =>
    sum + (strategy.metrics.totalProfit || 0), 0) || 0;

  const avgSuccessRate = strategies?.reduce((sum: number, strategy: any) =>
    sum + (strategy.metrics.successRate || 0), 0) / (strategies?.length || 1) || 0;

  const totalTrades = strategies?.reduce((sum: number, strategy: any) =>
    sum + (strategy.metrics.totalTrades || 0), 0) || 0;

  if (historyLoading || strategiesLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold text-white">Analytics</h1>
          <p className="text-gray-400 mt-1">Deep insights and performance analytics</p>
        </div>
        <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-8 text-center">
          <p className="text-gray-400">Loading analytics...</p>
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
        <h1 className="text-3xl font-bold text-white">Analytics</h1>
        <p className="text-gray-400 mt-1">Deep insights and performance analytics</p>
      </motion.div>

      {/* Key Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {/* Total Profit */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Total Profit</p>
              <p className="text-2xl font-bold text-green-400">
                ${totalProfit.toFixed(2)}
              </p>
            </div>
            <CurrencyDollarIcon className="h-8 w-8 text-green-400" />
          </div>
          <div className="mt-2 flex items-center">
            <ArrowTrendingUpIcon className="h-4 w-4 text-green-400 mr-1" />
            <span className="text-green-400 text-sm">+12.5% this week</span>
          </div>
        </motion.div>

        {/* Success Rate */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Avg Success Rate</p>
              <p className="text-2xl font-bold text-blue-400">
                {avgSuccessRate.toFixed(1)}%
              </p>
            </div>
            <ChartBarIcon className="h-8 w-8 text-blue-400" />
          </div>
          <div className="mt-4 bg-gray-700 rounded-full h-2">
            <div
              className="bg-blue-400 h-2 rounded-full transition-all duration-300"
              style={{ width: `${avgSuccessRate}%` }}
            />
          </div>
        </motion.div>

        {/* Total Trades */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Total Trades</p>
              <p className="text-2xl font-bold text-white">
                {totalTrades.toLocaleString()}
              </p>
            </div>
            <BoltIcon className="h-8 w-8 text-yellow-400" />
          </div>
          <div className="mt-2">
            <span className="text-gray-400 text-sm">
              {systemMetrics?.trading?.total_trades_today || 0} today
            </span>
          </div>
        </motion.div>

        {/* Avg Latency */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Avg Latency</p>
              <p className="text-2xl font-bold text-purple-400">
                {systemMetrics?.trading?.avg_latency_ms || 0}ms
              </p>
            </div>
            <ClockIcon className="h-8 w-8 text-purple-400" />
          </div>
          <div className="mt-2">
            <span className={`text-xs px-2 py-1 rounded-full ${
              (systemMetrics?.trading?.avg_latency_ms || 0) < 100
                ? 'bg-green-900 text-green-300'
                : 'bg-yellow-900 text-yellow-300'
            }`}>
              {(systemMetrics?.trading?.avg_latency_ms || 0) < 100 ? 'Excellent' : 'Good'}
            </span>
          </div>
        </motion.div>
      </div>

      {/* Strategy Performance */}
      <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
        <h2 className="text-xl font-semibold text-white mb-6">Strategy Performance</h2>
        <div className="space-y-4">
          {strategies?.map((strategy: any) => (
            <div key={strategy.id} className="flex items-center justify-between p-4 bg-[#0F1419] rounded-lg">
              <div className="flex items-center space-x-4">
                <div className={`w-3 h-3 rounded-full ${
                  strategy.status === 'active' ? 'bg-green-400' : 'bg-gray-400'
                }`} />
                <div>
                  <p className="text-white font-medium">{strategy.name}</p>
                  <p className="text-gray-400 text-sm">{strategy.type}</p>
                </div>
              </div>
              <div className="text-right">
                <p className="text-white font-medium">
                  ${strategy.metrics.totalProfit.toFixed(2)}
                </p>
                <p className="text-gray-400 text-sm">
                  {strategy.metrics.successRate.toFixed(1)}% success
                </p>
              </div>
              <div className="text-right">
                <p className="text-white">{strategy.metrics.totalTrades}</p>
                <p className="text-gray-400 text-sm">trades</p>
              </div>
              <div className="text-right">
                <p className="text-white">{strategy.metrics.avgLatency}ms</p>
                <p className="text-gray-400 text-sm">latency</p>
              </div>
            </div>
          ))}
        </div>
      </motion.div>

      {/* Recent Trades */}
      <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
        <h2 className="text-xl font-semibold text-white mb-6">Recent Trades</h2>
        <div className="space-y-3">
          {tradingHistory?.slice(0, 5).map((trade: any) => (
            <div key={trade.id} className="flex items-center justify-between p-3 bg-[#0F1419] rounded-lg">
              <div className="flex items-center space-x-3">
                <div className={`w-2 h-2 rounded-full ${
                  trade.status === 'completed' ? 'bg-green-400' : 'bg-red-400'
                }`} />
                <div>
                  <p className="text-white text-sm">{trade.token}</p>
                  <p className="text-gray-400 text-xs">{trade.strategy}</p>
                </div>
              </div>
              <div className="text-right">
                <p className={`text-sm font-medium ${
                  trade.profit_sol > 0 ? 'text-green-400' : 'text-red-400'
                }`}>
                  {trade.profit_sol > 0 ? '+' : ''}{trade.profit_sol.toFixed(4)} SOL
                </p>
                <p className="text-gray-400 text-xs">
                  {new Date(trade.timestamp).toLocaleTimeString()}
                </p>
              </div>
            </div>
          ))}
        </div>
      </motion.div>
    </motion.div>
  );
};

export default AnalyticsPage;
