import React from 'react';
import { motion } from 'framer-motion';
import { ArrowUpIcon, ArrowDownIcon, ClockIcon } from '@heroicons/react/24/outline';

interface Trade {
  id: string;
  strategy: string;
  type: 'buy' | 'sell';
  token: string;
  amount: number;
  price: number;
  profit: number;
  timestamp: string;
  status: 'completed' | 'pending' | 'failed';
}

const mockTrades: Trade[] = [
  {
    id: '1',
    strategy: 'Sandwich',
    type: 'buy',
    token: 'SOL',
    amount: 10.5,
    price: 98.45,
    profit: 234.56,
    timestamp: '2 min ago',
    status: 'completed',
  },
  {
    id: '2',
    strategy: 'Arbitrage',
    type: 'sell',
    token: 'USDC',
    amount: 1000,
    price: 1.001,
    profit: 89.23,
    timestamp: '5 min ago',
    status: 'completed',
  },
  {
    id: '3',
    strategy: 'Market Making',
    type: 'buy',
    token: 'RAY',
    amount: 50,
    price: 2.34,
    profit: -12.45,
    timestamp: '8 min ago',
    status: 'completed',
  },
  {
    id: '4',
    strategy: 'Liquidation',
    type: 'sell',
    token: 'ORCA',
    amount: 25,
    price: 4.56,
    profit: 156.78,
    timestamp: '12 min ago',
    status: 'completed',
  },
];

const RecentTrades: React.FC = () => {
  return (
    <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-white">Recent Trades</h2>
        <button className="text-purple-400 hover:text-purple-300 text-sm font-medium">
          View All →
        </button>
      </div>

      <div className="space-y-4">
        {mockTrades.map((trade, index) => (
          <motion.div
            key={trade.id}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: index * 0.1 }}
            className="flex items-center justify-between p-4 bg-[#0F1419] rounded-lg border border-[#2A2D3A] hover:border-[#3A3D4A] transition-colors"
          >
            <div className="flex items-center space-x-4">
              {/* Trade Type Icon */}
              <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${
                trade.type === 'buy' 
                  ? 'bg-green-500/20 text-green-400' 
                  : 'bg-red-500/20 text-red-400'
              }`}>
                {trade.type === 'buy' ? (
                  <ArrowUpIcon className="w-5 h-5" />
                ) : (
                  <ArrowDownIcon className="w-5 h-5" />
                )}
              </div>

              {/* Trade Details */}
              <div>
                <div className="flex items-center space-x-2">
                  <span className="font-medium text-white">
                    {trade.type.toUpperCase()} {trade.token}
                  </span>
                  <span className="text-xs px-2 py-1 bg-purple-500/20 text-purple-400 rounded">
                    {trade.strategy}
                  </span>
                </div>
                <div className="flex items-center space-x-4 text-sm text-gray-400 mt-1">
                  <span>{trade.amount} {trade.token}</span>
                  <span>@${trade.price}</span>
                  <div className="flex items-center space-x-1">
                    <ClockIcon className="w-3 h-3" />
                    <span>{trade.timestamp}</span>
                  </div>
                </div>
              </div>
            </div>

            {/* Profit/Loss */}
            <div className="text-right">
              <div className={`font-semibold ${
                trade.profit >= 0 ? 'text-green-400' : 'text-red-400'
              }`}>
                {trade.profit >= 0 ? '+' : ''}${trade.profit.toFixed(2)}
              </div>
              <div className={`text-xs px-2 py-1 rounded-full ${
                trade.status === 'completed' 
                  ? 'bg-green-500/20 text-green-400'
                  : trade.status === 'pending'
                  ? 'bg-yellow-500/20 text-yellow-400'
                  : 'bg-red-500/20 text-red-400'
              }`}>
                {trade.status}
              </div>
            </div>
          </motion.div>
        ))}
      </div>

      {/* Summary */}
      <div className="mt-6 pt-4 border-t border-[#2A2D3A]">
        <div className="grid grid-cols-3 gap-4 text-center">
          <div>
            <p className="text-2xl font-bold text-green-400">+$467.12</p>
            <p className="text-xs text-gray-400">Total Profit</p>
          </div>
          <div>
            <p className="text-2xl font-bold text-white">4</p>
            <p className="text-xs text-gray-400">Trades Today</p>
          </div>
          <div>
            <p className="text-2xl font-bold text-purple-400">75%</p>
            <p className="text-xs text-gray-400">Success Rate</p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default RecentTrades;
