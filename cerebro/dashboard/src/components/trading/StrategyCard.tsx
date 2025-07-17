import React from 'react';
import { motion } from 'framer-motion';
import { 
  PlayIcon, 
  PauseIcon, 
  StopIcon,
  ChartBarIcon,
  Cog6ToothIcon 
} from '@heroicons/react/24/outline';

interface StrategyCardProps {
  name: string;
  type: 'sandwich' | 'arbitrage' | 'liquidation' | 'sniping' | 'market_making';
  performance: string;
  trades: number;
  status: 'active' | 'inactive' | 'paused' | 'error';
  loading?: boolean;
}

const StrategyCard: React.FC<StrategyCardProps> = ({
  name,
  type,
  performance,
  trades,
  status,
  loading = false,
}) => {
  const getStatusColor = () => {
    switch (status) {
      case 'active':
        return 'text-green-400';
      case 'paused':
        return 'text-yellow-400';
      case 'error':
        return 'text-red-400';
      default:
        return 'text-gray-400';
    }
  };

  const getStatusIcon = () => {
    switch (status) {
      case 'active':
        return <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>;
      case 'paused':
        return <div className="w-2 h-2 bg-yellow-400 rounded-full"></div>;
      case 'error':
        return <div className="w-2 h-2 bg-red-400 rounded-full"></div>;
      default:
        return <div className="w-2 h-2 bg-gray-400 rounded-full"></div>;
    }
  };

  const getTypeIcon = () => {
    switch (type) {
      case 'sandwich':
        return '🥪';
      case 'arbitrage':
        return '⚖️';
      case 'liquidation':
        return '💧';
      case 'sniping':
        return '🎯';
      case 'market_making':
        return '🏪';
      default:
        return '⚡';
    }
  };

  if (loading) {
    return (
      <div className="bg-[#0F1419] border border-[#2A2D3A] rounded-lg p-4">
        <div className="animate-pulse">
          <div className="flex items-center justify-between mb-3">
            <div className="h-4 bg-[#2A2D3A] rounded w-24"></div>
            <div className="h-6 w-6 bg-[#2A2D3A] rounded"></div>
          </div>
          <div className="h-6 bg-[#2A2D3A] rounded w-16 mb-2"></div>
          <div className="h-4 bg-[#2A2D3A] rounded w-20"></div>
        </div>
      </div>
    );
  }

  return (
    <motion.div
      whileHover={{ scale: 1.02 }}
      transition={{ type: "spring", stiffness: 300, damping: 30 }}
      className="bg-[#0F1419] border border-[#2A2D3A] rounded-lg p-4 hover:border-[#3A3D4A] transition-all duration-200 group"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center space-x-2">
          <span className="text-lg">{getTypeIcon()}</span>
          <h4 className="font-medium text-white text-sm">{name}</h4>
        </div>
        <div className="flex items-center space-x-2">
          {getStatusIcon()}
          <span className={`text-xs font-medium capitalize ${getStatusColor()}`}>
            {status}
          </span>
        </div>
      </div>

      {/* Performance */}
      <div className="mb-3">
        <div className="flex items-center justify-between">
          <span className="text-xs text-gray-400">Performance</span>
          <span className={`text-sm font-semibold ${
            performance.startsWith('+') ? 'text-green-400' : 'text-red-400'
          }`}>
            {performance}
          </span>
        </div>
      </div>

      {/* Trades */}
      <div className="mb-4">
        <div className="flex items-center justify-between">
          <span className="text-xs text-gray-400">Total Trades</span>
          <span className="text-sm font-medium text-white">{trades}</span>
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center justify-between pt-3 border-t border-[#2A2D3A]">
        <div className="flex items-center space-x-2">
          {status === 'active' ? (
            <button className="p-1.5 rounded-md hover:bg-[#2A2D3A] transition-colors">
              <PauseIcon className="w-4 h-4 text-yellow-400" />
            </button>
          ) : (
            <button className="p-1.5 rounded-md hover:bg-[#2A2D3A] transition-colors">
              <PlayIcon className="w-4 h-4 text-green-400" />
            </button>
          )}
          <button className="p-1.5 rounded-md hover:bg-[#2A2D3A] transition-colors">
            <StopIcon className="w-4 h-4 text-red-400" />
          </button>
        </div>
        
        <div className="flex items-center space-x-2">
          <button className="p-1.5 rounded-md hover:bg-[#2A2D3A] transition-colors">
            <ChartBarIcon className="w-4 h-4 text-gray-400 hover:text-white" />
          </button>
          <button className="p-1.5 rounded-md hover:bg-[#2A2D3A] transition-colors">
            <Cog6ToothIcon className="w-4 h-4 text-gray-400 hover:text-white" />
          </button>
        </div>
      </div>
    </motion.div>
  );
};

export default StrategyCard;
