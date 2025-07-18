import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { motion } from 'framer-motion';
import { apiClient } from '@/services/api';

// Icons
import {
  BoltIcon,
  ChartBarIcon,
  CurrencyDollarIcon,
  ClockIcon,
  CheckCircleIcon,
  XCircleIcon,
  PlayIcon,
  PauseIcon,
  Cog6ToothIcon
} from '@heroicons/react/24/outline';

const TradingPage: React.FC = () => {
  const [selectedStrategy, setSelectedStrategy] = useState<string | null>(null);

  // Fetch trading data
  const { data: tradingStatus } = useQuery({
    queryKey: ['trading-status'],
    queryFn: () => apiClient.get('/api/trading/status'),
    refetchInterval: 5000,
  });

  const { data: strategies } = useQuery({
    queryKey: ['strategies'],
    queryFn: () => apiClient.get('/api/strategies'),
    refetchInterval: 10000,
  });

  const { data: tradingHistory } = useQuery({
    queryKey: ['trading-history'],
    queryFn: () => apiClient.get('/api/trading/history'),
    refetchInterval: 15000,
  });

  const { data: portfolio } = useQuery({
    queryKey: ['portfolio'],
    queryFn: () => apiClient.get('/api/portfolio'),
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

  const handleStrategyToggle = (strategyId: string) => {
    // In real implementation, this would call API to toggle strategy
    console.log('Toggle strategy:', strategyId);
  };

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      className="space-y-6"
    >
      {/* Page Header */}
      <motion.div variants={itemVariants}>
        <h1 className="text-3xl font-bold text-white">Trading</h1>
        <p className="text-gray-400 mt-1">Advanced trading interface and portfolio management</p>
      </motion.div>

      {/* Trading Status */}
      <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-semibold text-white">Trading Status</h2>
          <div className="flex items-center space-x-2">
            <div className={`w-3 h-3 rounded-full ${
              tradingStatus?.trading_enabled ? 'bg-green-400' : 'bg-red-400'
            }`} />
            <span className="text-white">
              {tradingStatus?.current_mode === 'dry_run' ? 'Dry Run Mode' : 'Live Trading'}
            </span>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="text-center">
            <p className="text-gray-400 text-sm">Active Strategies</p>
            <p className="text-2xl font-bold text-white">
              {tradingStatus?.strategies_active?.length || 0}
            </p>
          </div>
          <div className="text-center">
            <p className="text-gray-400 text-sm">Uptime</p>
            <p className="text-2xl font-bold text-white">
              {Math.floor((tradingStatus?.uptime_seconds || 0) / 3600)}h
            </p>
          </div>
          <div className="text-center">
            <p className="text-gray-400 text-sm">Mode</p>
            <p className="text-2xl font-bold text-blue-400">
              {tradingStatus?.current_mode?.toUpperCase() || 'UNKNOWN'}
            </p>
          </div>
          <div className="text-center">
            <p className="text-gray-400 text-sm">Last Update</p>
            <p className="text-sm text-gray-400">
              {tradingStatus?.last_update ? new Date(tradingStatus.last_update).toLocaleTimeString() : 'N/A'}
            </p>
          </div>
        </div>
      </motion.div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Portfolio Overview */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <h2 className="text-xl font-semibold text-white mb-6 flex items-center">
            <CurrencyDollarIcon className="h-6 w-6 mr-2 text-green-400" />
            Portfolio
          </h2>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Total Value</span>
              <span className="text-2xl font-bold text-white">
                ${portfolio?.totalValue?.toFixed(2) || '0.00'}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-gray-400">SOL Balance</span>
              <span className="text-xl font-semibold text-white">
                {portfolio?.solBalance?.toFixed(3) || '0.000'} SOL
              </span>
            </div>

            <div className="border-t border-[#2A2D3A] pt-4">
              <h3 className="text-white font-medium mb-3">Performance</h3>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-gray-400 text-sm">Daily</span>
                  <span className={`text-sm font-medium ${
                    (portfolio?.performance?.daily || 0) >= 0 ? 'text-green-400' : 'text-red-400'
                  }`}>
                    {(portfolio?.performance?.daily || 0) >= 0 ? '+' : ''}{portfolio?.performance?.daily?.toFixed(2) || '0.00'}%
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-gray-400 text-sm">Weekly</span>
                  <span className={`text-sm font-medium ${
                    (portfolio?.performance?.weekly || 0) >= 0 ? 'text-green-400' : 'text-red-400'
                  }`}>
                    {(portfolio?.performance?.weekly || 0) >= 0 ? '+' : ''}{portfolio?.performance?.weekly?.toFixed(2) || '0.00'}%
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-gray-400 text-sm">Monthly</span>
                  <span className={`text-sm font-medium ${
                    (portfolio?.performance?.monthly || 0) >= 0 ? 'text-green-400' : 'text-red-400'
                  }`}>
                    {(portfolio?.performance?.monthly || 0) >= 0 ? '+' : ''}{portfolio?.performance?.monthly?.toFixed(2) || '0.00'}%
                  </span>
                </div>
              </div>
            </div>
          </div>
        </motion.div>

        {/* Strategy Controls */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <h2 className="text-xl font-semibold text-white mb-6 flex items-center">
            <BoltIcon className="h-6 w-6 mr-2 text-yellow-400" />
            Strategy Controls
          </h2>

          <div className="space-y-4">
            {strategies?.map((strategy: any) => (
              <div key={strategy.id} className="bg-[#0F1419] rounded-lg p-4">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center space-x-3">
                    <div className={`w-3 h-3 rounded-full ${
                      strategy.status === 'active' ? 'bg-green-400' : 'bg-gray-400'
                    }`} />
                    <div>
                      <h3 className="text-white font-medium">{strategy.name}</h3>
                      <p className="text-gray-400 text-sm">{strategy.type}</p>
                    </div>
                  </div>
                  <button
                    onClick={() => handleStrategyToggle(strategy.id)}
                    className={`p-2 rounded-lg transition-colors ${
                      strategy.status === 'active'
                        ? 'bg-red-600 hover:bg-red-700 text-white'
                        : 'bg-green-600 hover:bg-green-700 text-white'
                    }`}
                  >
                    {strategy.status === 'active' ? (
                      <PauseIcon className="h-4 w-4" />
                    ) : (
                      <PlayIcon className="h-4 w-4" />
                    )}
                  </button>
                </div>

                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div>
                    <p className="text-gray-400">Trades</p>
                    <p className="text-white font-medium">{strategy.metrics.totalTrades}</p>
                  </div>
                  <div>
                    <p className="text-gray-400">Success Rate</p>
                    <p className="text-green-400 font-medium">{strategy.metrics.successRate.toFixed(1)}%</p>
                  </div>
                  <div>
                    <p className="text-gray-400">Profit</p>
                    <p className="text-yellow-400 font-medium">${strategy.metrics.totalProfit.toFixed(2)}</p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </motion.div>
      </div>

      {/* Recent Trades */}
      <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
        <h2 className="text-xl font-semibold text-white mb-6 flex items-center">
          <ChartBarIcon className="h-6 w-6 mr-2 text-blue-400" />
          Recent Trades
        </h2>

        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-[#2A2D3A]">
                <th className="text-left text-gray-400 text-sm font-medium py-3">Time</th>
                <th className="text-left text-gray-400 text-sm font-medium py-3">Strategy</th>
                <th className="text-left text-gray-400 text-sm font-medium py-3">Token</th>
                <th className="text-left text-gray-400 text-sm font-medium py-3">Type</th>
                <th className="text-left text-gray-400 text-sm font-medium py-3">Profit/Loss</th>
                <th className="text-left text-gray-400 text-sm font-medium py-3">Status</th>
              </tr>
            </thead>
            <tbody>
              {tradingHistory?.slice(0, 10).map((trade: any) => (
                <tr key={trade.id} className="border-b border-[#2A2D3A]/50">
                  <td className="py-3 text-gray-400 text-sm">
                    {new Date(trade.timestamp).toLocaleTimeString()}
                  </td>
                  <td className="py-3 text-white text-sm">{trade.strategy}</td>
                  <td className="py-3 text-white text-sm">{trade.token}</td>
                  <td className="py-3 text-gray-400 text-sm">{trade.type}</td>
                  <td className={`py-3 text-sm font-medium ${
                    trade.profit_sol >= 0 ? 'text-green-400' : 'text-red-400'
                  }`}>
                    {trade.profit_sol >= 0 ? '+' : ''}{trade.profit_sol.toFixed(6)} SOL
                  </td>
                  <td className="py-3">
                    {trade.status === 'completed' ? (
                      <CheckCircleIcon className="h-5 w-5 text-green-400" />
                    ) : (
                      <XCircleIcon className="h-5 w-5 text-red-400" />
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </motion.div>
    </motion.div>
  );
};

export default TradingPage;
