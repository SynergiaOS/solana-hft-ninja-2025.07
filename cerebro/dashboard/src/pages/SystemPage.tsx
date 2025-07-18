import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { motion } from 'framer-motion';
import { apiClient } from '@/services/api';

// Icons
import {
  CpuChipIcon,
  CircleStackIcon,
  WifiIcon,
  ClockIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon
} from '@heroicons/react/24/outline';

const SystemPage: React.FC = () => {
  // Fetch system metrics
  const { data: systemMetrics, isLoading } = useQuery({
    queryKey: ['system-metrics'],
    queryFn: () => apiClient.get('/api/system/metrics'),
    refetchInterval: 5000, // Refresh every 5 seconds
  });

  // Fetch HFT metrics
  const { data: hftMetrics } = useQuery({
    queryKey: ['hft-metrics'],
    queryFn: () => apiClient.get('/api/hft/metrics'),
    refetchInterval: 5000,
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

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold text-white">System</h1>
          <p className="text-gray-400 mt-1">System health and monitoring</p>
        </div>
        <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-8 text-center">
          <p className="text-gray-400">Loading system metrics...</p>
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
        <h1 className="text-3xl font-bold text-white">System</h1>
        <p className="text-gray-400 mt-1">System health and monitoring</p>
      </motion.div>

      {/* System Status Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {/* CPU Usage */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">CPU Usage</p>
              <p className="text-2xl font-bold text-white">
                {systemMetrics?.system?.cpu_usage?.toFixed(1) || '0'}%
              </p>
            </div>
            <CpuChipIcon className="h-8 w-8 text-blue-400" />
          </div>
          <div className="mt-4 bg-gray-700 rounded-full h-2">
            <div
              className="bg-blue-400 h-2 rounded-full transition-all duration-300"
              style={{ width: `${systemMetrics?.system?.cpu_usage || 0}%` }}
            />
          </div>
        </motion.div>

        {/* Memory Usage */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Memory Usage</p>
              <p className="text-2xl font-bold text-white">
                {systemMetrics?.system?.memory_usage?.toFixed(1) || '0'}%
              </p>
            </div>
            <CircleStackIcon className="h-8 w-8 text-green-400" />
          </div>
          <div className="mt-4 bg-gray-700 rounded-full h-2">
            <div
              className="bg-green-400 h-2 rounded-full transition-all duration-300"
              style={{ width: `${systemMetrics?.system?.memory_usage || 0}%` }}
            />
          </div>
        </motion.div>

        {/* Network Latency */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Network Latency</p>
              <p className="text-2xl font-bold text-white">
                {systemMetrics?.system?.network_latency_ms || '0'}ms
              </p>
            </div>
            <WifiIcon className="h-8 w-8 text-purple-400" />
          </div>
          <div className="mt-2">
            <span className={`text-xs px-2 py-1 rounded-full ${
              (systemMetrics?.system?.network_latency_ms || 0) < 50
                ? 'bg-green-900 text-green-300'
                : 'bg-yellow-900 text-yellow-300'
            }`}>
              {(systemMetrics?.system?.network_latency_ms || 0) < 50 ? 'Excellent' : 'Good'}
            </span>
          </div>
        </motion.div>

        {/* Uptime */}
        <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Uptime</p>
              <p className="text-2xl font-bold text-white">
                {Math.floor((systemMetrics?.trading?.uptime_seconds || 0) / 3600)}h
              </p>
            </div>
            <ClockIcon className="h-8 w-8 text-orange-400" />
          </div>
          <div className="mt-2">
            <span className="text-xs px-2 py-1 rounded-full bg-green-900 text-green-300">
              Running
            </span>
          </div>
        </motion.div>
      </div>

      {/* Service Status */}
      <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
        <h2 className="text-xl font-semibold text-white mb-6">Service Status</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* HFT Engine */}
          <div className="flex items-center space-x-3">
            {hftMetrics?.status === 'connected' ? (
              <CheckCircleIcon className="h-5 w-5 text-green-400" />
            ) : (
              <ExclamationTriangleIcon className="h-5 w-5 text-red-400" />
            )}
            <div>
              <p className="text-white font-medium">HFT Engine</p>
              <p className="text-gray-400 text-sm">
                {hftMetrics?.status === 'connected' ? 'Connected' : 'Disconnected'}
              </p>
            </div>
          </div>

          {/* WebSocket */}
          <div className="flex items-center space-x-3">
            {systemMetrics?.system?.websocket_connected ? (
              <CheckCircleIcon className="h-5 w-5 text-green-400" />
            ) : (
              <ExclamationTriangleIcon className="h-5 w-5 text-red-400" />
            )}
            <div>
              <p className="text-white font-medium">WebSocket</p>
              <p className="text-gray-400 text-sm">
                {systemMetrics?.system?.websocket_connected ? 'Connected' : 'Disconnected'}
              </p>
            </div>
          </div>

          {/* Database */}
          <div className="flex items-center space-x-3">
            <CheckCircleIcon className="h-5 w-5 text-green-400" />
            <div>
              <p className="text-white font-medium">DragonflyDB</p>
              <p className="text-gray-400 text-sm">Connected</p>
            </div>
          </div>
        </div>
      </motion.div>

      {/* Trading Metrics */}
      <motion.div variants={itemVariants} className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
        <h2 className="text-xl font-semibold text-white mb-6">Trading Performance</h2>
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div>
            <p className="text-gray-400 text-sm">Total Trades Today</p>
            <p className="text-2xl font-bold text-white">
              {systemMetrics?.trading?.total_trades_today || 0}
            </p>
          </div>
          <div>
            <p className="text-gray-400 text-sm">Success Rate</p>
            <p className="text-2xl font-bold text-green-400">
              {systemMetrics?.trading?.success_rate?.toFixed(1) || '0'}%
            </p>
          </div>
          <div>
            <p className="text-gray-400 text-sm">Avg Latency</p>
            <p className="text-2xl font-bold text-blue-400">
              {systemMetrics?.trading?.avg_latency_ms || 0}ms
            </p>
          </div>
          <div>
            <p className="text-gray-400 text-sm">Total Profit</p>
            <p className="text-2xl font-bold text-yellow-400">
              {systemMetrics?.trading?.total_profit_sol?.toFixed(3) || '0'} SOL
            </p>
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
};

export default SystemPage;
