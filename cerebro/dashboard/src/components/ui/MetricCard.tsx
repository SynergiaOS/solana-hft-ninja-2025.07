import React from 'react';
import { motion } from 'framer-motion';
import { ArrowUpIcon, ArrowDownIcon } from '@heroicons/react/24/outline';

interface MetricCardProps {
  title: string;
  value: string;
  change?: string;
  changeType?: 'positive' | 'negative' | 'neutral';
  icon?: string;
  loading?: boolean;
  className?: string;
}

const MetricCard: React.FC<MetricCardProps> = ({
  title,
  value,
  change,
  changeType = 'neutral',
  icon,
  loading = false,
  className = '',
}) => {
  const getChangeColor = () => {
    switch (changeType) {
      case 'positive':
        return 'text-green-400';
      case 'negative':
        return 'text-red-400';
      default:
        return 'text-gray-400';
    }
  };

  const getChangeIcon = () => {
    switch (changeType) {
      case 'positive':
        return <ArrowUpIcon className="w-3 h-3" />;
      case 'negative':
        return <ArrowDownIcon className="w-3 h-3" />;
      default:
        return null;
    }
  };

  if (loading) {
    return (
      <div className={`bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6 ${className}`}>
        <div className="animate-pulse">
          <div className="flex items-center justify-between mb-4">
            <div className="h-4 bg-[#2A2D3A] rounded w-20"></div>
            <div className="h-6 w-6 bg-[#2A2D3A] rounded"></div>
          </div>
          <div className="h-8 bg-[#2A2D3A] rounded w-32 mb-2"></div>
          <div className="h-4 bg-[#2A2D3A] rounded w-16"></div>
        </div>
      </div>
    );
  }

  return (
    <motion.div
      whileHover={{ scale: 1.02 }}
      transition={{ type: "spring", stiffness: 300, damping: 30 }}
      className={`bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6 hover:border-[#3A3D4A] transition-all duration-200 ${className}`}
    >
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-medium text-gray-400">{title}</h3>
        {icon && (
          <div className="text-2xl opacity-80">
            {icon}
          </div>
        )}
      </div>
      
      <div className="flex items-end justify-between">
        <div>
          <p className="text-2xl font-bold text-white mb-1">{value}</p>
          {change && (
            <div className={`flex items-center space-x-1 ${getChangeColor()}`}>
              {getChangeIcon()}
              <span className="text-sm font-medium">{change}</span>
            </div>
          )}
        </div>
        
        {/* Optional trend indicator */}
        {changeType !== 'neutral' && (
          <div className={`w-12 h-8 rounded ${
            changeType === 'positive' 
              ? 'bg-green-400/10' 
              : 'bg-red-400/10'
          } flex items-center justify-center`}>
            <div className={`w-2 h-2 rounded-full ${
              changeType === 'positive' 
                ? 'bg-green-400' 
                : 'bg-red-400'
            } animate-pulse`}></div>
          </div>
        )}
      </div>
    </motion.div>
  );
};

export default MetricCard;
