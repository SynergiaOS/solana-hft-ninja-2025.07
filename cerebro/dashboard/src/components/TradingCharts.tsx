import React from 'react';
import { motion } from 'framer-motion';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell
} from 'recharts';

// Mock data for charts
const profitData = [
  { time: '09:00', profit: 0.12, cumulative: 0.12 },
  { time: '10:00', profit: 0.08, cumulative: 0.20 },
  { time: '11:00', profit: 0.15, cumulative: 0.35 },
  { time: '12:00', profit: 0.22, cumulative: 0.57 },
  { time: '13:00', profit: 0.18, cumulative: 0.75 },
  { time: '14:00', profit: 0.25, cumulative: 1.00 },
  { time: '15:00', profit: 0.19, cumulative: 1.19 },
];

const strategyData = [
  { name: 'Sandwich', trades: 12, profit: 0.847, color: '#F59E0B' },
  { name: 'Arbitrage', trades: 8, profit: 0.623, color: '#3B82F6' },
  { name: 'Liquidation', trades: 3, profit: 0.456, color: '#8B5CF6' },
  { name: 'Token Snipe', trades: 2, profit: 0.445, color: '#10B981' },
  { name: 'Jupiter Arb', trades: 6, profit: 0.234, color: '#EF4444' },
];

const executionTimeData = [
  { strategy: 'Sandwich', avgTime: 89, target: 100 },
  { strategy: 'Arbitrage', avgTime: 145, target: 150 },
  { strategy: 'Liquidation', avgTime: 78, target: 100 },
  { strategy: 'Token Snipe', avgTime: 52, target: 80 },
  { strategy: 'Jupiter Arb', avgTime: 118, target: 120 },
];

const volumeData = [
  { hour: '09', volume: 2.3, opportunities: 15 },
  { hour: '10', volume: 3.1, opportunities: 22 },
  { hour: '11', volume: 4.2, opportunities: 28 },
  { hour: '12', volume: 5.8, opportunities: 35 },
  { hour: '13', volume: 4.9, opportunities: 31 },
  { hour: '14', volume: 6.2, opportunities: 42 },
  { hour: '15', volume: 3.7, opportunities: 26 },
];

const TradingCharts: React.FC = () => {
  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-gray-800 border border-gray-600 rounded-lg p-3 shadow-lg">
          <p className="text-white font-medium">{label}</p>
          {payload.map((entry: any, index: number) => (
            <p key={index} style={{ color: entry.color }} className="text-sm">
              {entry.name}: {typeof entry.value === 'number' ? entry.value.toFixed(4) : entry.value}
              {entry.name.includes('profit') || entry.name.includes('Profit') ? ' SOL' : ''}
              {entry.name.includes('time') || entry.name.includes('Time') ? 'ms' : ''}
            </p>
          ))}
        </div>
      );
    }
    return null;
  };

  return (
    <div className="space-y-6">
      {/* Profit Over Time */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6"
      >
        <h3 className="text-xl font-semibold text-white mb-6">Profit Over Time</h3>
        <div className="h-80">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={profitData}>
              <defs>
                <linearGradient id="profitGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#10B981" stopOpacity={0.3}/>
                  <stop offset="95%" stopColor="#10B981" stopOpacity={0}/>
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="time" stroke="#9CA3AF" />
              <YAxis stroke="#9CA3AF" />
              <Tooltip content={<CustomTooltip />} />
              <Area
                type="monotone"
                dataKey="cumulative"
                stroke="#10B981"
                strokeWidth={2}
                fill="url(#profitGradient)"
              />
              <Line
                type="monotone"
                dataKey="profit"
                stroke="#F59E0B"
                strokeWidth={2}
                dot={{ fill: '#F59E0B', strokeWidth: 2, r: 4 }}
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </motion.div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Strategy Performance */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6"
        >
          <h3 className="text-xl font-semibold text-white mb-6">Strategy Distribution</h3>
          <div className="h-80">
            <ResponsiveContainer width="100%" height="100%">
              <PieChart>
                <Pie
                  data={strategyData}
                  cx="50%"
                  cy="50%"
                  outerRadius={100}
                  dataKey="profit"
                  label={({ name, value }) => `${name}: ${value.toFixed(3)} SOL`}
                  labelLine={false}
                >
                  {strategyData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip content={<CustomTooltip />} />
              </PieChart>
            </ResponsiveContainer>
          </div>
        </motion.div>

        {/* Execution Time Analysis */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6"
        >
          <h3 className="text-xl font-semibold text-white mb-6">Execution Time vs Target</h3>
          <div className="h-80">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={executionTimeData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="strategy" stroke="#9CA3AF" />
                <YAxis stroke="#9CA3AF" />
                <Tooltip content={<CustomTooltip />} />
                <Bar dataKey="avgTime" fill="#8B5CF6" name="Avg Time" />
                <Bar dataKey="target" fill="#374151" name="Target" />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </motion.div>
      </div>

      {/* Volume and Opportunities */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
        className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6"
      >
        <h3 className="text-xl font-semibold text-white mb-6">Trading Volume & Opportunities</h3>
        <div className="h-80">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={volumeData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="hour" stroke="#9CA3AF" />
              <YAxis yAxisId="left" stroke="#9CA3AF" />
              <YAxis yAxisId="right" orientation="right" stroke="#9CA3AF" />
              <Tooltip content={<CustomTooltip />} />
              <Line
                yAxisId="left"
                type="monotone"
                dataKey="volume"
                stroke="#3B82F6"
                strokeWidth={3}
                dot={{ fill: '#3B82F6', strokeWidth: 2, r: 5 }}
                name="Volume (SOL)"
              />
              <Line
                yAxisId="right"
                type="monotone"
                dataKey="opportunities"
                stroke="#F59E0B"
                strokeWidth={3}
                dot={{ fill: '#F59E0B', strokeWidth: 2, r: 5 }}
                name="Opportunities"
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </motion.div>

      {/* Performance Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="bg-gradient-to-br from-green-900/20 to-green-800/10 border border-green-700/30 rounded-xl p-6 text-center"
        >
          <div className="text-3xl font-bold text-green-400">94.2%</div>
          <div className="text-green-300 text-sm mt-1">Success Rate</div>
          <div className="text-xs text-gray-400 mt-2">↑ 2.1% from yesterday</div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.5 }}
          className="bg-gradient-to-br from-blue-900/20 to-blue-800/10 border border-blue-700/30 rounded-xl p-6 text-center"
        >
          <div className="text-3xl font-bold text-blue-400">89ms</div>
          <div className="text-blue-300 text-sm mt-1">Avg Execution</div>
          <div className="text-xs text-gray-400 mt-2">↓ 12ms from target</div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.6 }}
          className="bg-gradient-to-br from-purple-900/20 to-purple-800/10 border border-purple-700/30 rounded-xl p-6 text-center"
        >
          <div className="text-3xl font-bold text-purple-400">156</div>
          <div className="text-purple-300 text-sm mt-1">Opportunities</div>
          <div className="text-xs text-gray-400 mt-2">↑ 23 from last hour</div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.7 }}
          className="bg-gradient-to-br from-orange-900/20 to-orange-800/10 border border-orange-700/30 rounded-xl p-6 text-center"
        >
          <div className="text-3xl font-bold text-orange-400">12.5%</div>
          <div className="text-orange-300 text-sm mt-1">Daily ROI</div>
          <div className="text-xs text-gray-400 mt-2">Target: 5%</div>
        </motion.div>
      </div>
    </div>
  );
};

export default TradingCharts;
