import React from 'react';
import { NavLink, useLocation } from 'react-router-dom';
import { motion } from 'framer-motion';
import {
  HomeIcon,
  ChartBarIcon,
  CpuChipIcon,
  Cog6ToothIcon,
  BoltIcon,
  ChartPieIcon,
  ServerStackIcon,
  BeakerIcon,
  XMarkIcon,
  // 🆕 New AI & Memory Icons
  BrainIcon,
  SparklesIcon,
  WalletIcon,
} from '@heroicons/react/24/outline';
import { Brain } from 'lucide-react';
import ConnectionStatus from '@/components/ui/ConnectionStatus';

interface SidebarProps {
  onClose: () => void;
}

const navigation = [
  { name: 'Dashboard', href: '/overview', icon: HomeIcon },
  { name: 'Trading', href: '/trading', icon: ChartBarIcon },
  { name: 'FinGPT AI', href: '/fingpt', icon: CpuChipIcon, badge: 'AI' },
  { name: 'Enhanced AI', href: '/enhanced', icon: Brain, badge: 'NEW' },
  // 🆕 New AI & Memory Navigation
  { name: 'AI Memory', href: '/ai-memory', icon: BrainIcon, badge: 'BETA' },
  { name: 'Predictions', href: '/predictions', icon: SparklesIcon, badge: 'AI' },
  { name: 'Webhook Events', href: '/webhook-events', icon: WalletIcon, badge: 'LIVE' },
  { name: 'Strategies', href: '/strategies', icon: BoltIcon },
  { name: 'Analytics', href: '/analytics', icon: ChartPieIcon },
  { name: 'System', href: '/system', icon: ServerStackIcon },
  { name: 'Settings', href: '/settings', icon: Cog6ToothIcon },
];

const Sidebar: React.FC<SidebarProps> = ({ onClose }) => {
  const location = useLocation();

  return (
    <div className="flex flex-col w-72 h-full bg-[#1A1D29] border-r border-[#2A2D3A]">
      {/* Header */}
      <div className="flex items-center justify-between p-6 border-b border-[#2A2D3A]">
        <div className="flex items-center space-x-3">
          {/* Cerebro Logo */}
          <div className="w-10 h-10 bg-gradient-to-br from-purple-500 to-purple-700 rounded-xl flex items-center justify-center">
            <BeakerIcon className="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-white">Cerebro</h1>
            <p className="text-xs text-gray-400">AI Trading Intelligence</p>
          </div>
        </div>
        
        {/* Close button for mobile */}
        <button
          onClick={onClose}
          className="lg:hidden p-2 rounded-lg hover:bg-[#2A2D3A] transition-colors"
        >
          <XMarkIcon className="w-5 h-5 text-gray-400" />
        </button>
      </div>

      {/* User Profile */}
      <div className="p-6 border-b border-[#2A2D3A]">
        <div className="flex items-center space-x-3">
          <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
            <span className="text-sm font-semibold text-white">BC</span>
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-white truncate">Bryan Crawford</p>
            <p className="text-xs text-gray-400 truncate">HFT Trader</p>
          </div>
          <div className="flex items-center space-x-1">
            <div className="w-2 h-2 bg-green-400 rounded-full"></div>
            <span className="text-xs text-gray-400">Online</span>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-4 py-6 space-y-2">
        {navigation.map((item) => {
          const isActive = location.pathname === item.href;
          
          return (
            <NavLink
              key={item.name}
              to={item.href}
              className={({ isActive }) =>
                `group flex items-center px-4 py-3 text-sm font-medium rounded-xl transition-all duration-200 ${
                  isActive
                    ? 'bg-gradient-to-r from-purple-600 to-purple-700 text-white shadow-lg'
                    : 'text-gray-300 hover:text-white hover:bg-[#2A2D3A]'
                }`
              }
            >
              {({ isActive }) => (
                <>
                  <item.icon
                    className={`mr-3 h-5 w-5 transition-colors ${
                      isActive ? 'text-white' : 'text-gray-400 group-hover:text-white'
                    }`}
                  />
                  <span className="flex-1">{item.name}</span>
                  {item.badge && (
                    <motion.span
                      initial={{ scale: 0 }}
                      animate={{ scale: 1 }}
                      className="ml-2 px-2 py-1 text-xs font-semibold bg-purple-500 text-white rounded-full"
                    >
                      {item.badge}
                    </motion.span>
                  )}
                  {isActive && (
                    <motion.div
                      layoutId="activeTab"
                      className="absolute left-0 w-1 h-8 bg-white rounded-r-full"
                      transition={{ type: "spring", bounce: 0.2, duration: 0.6 }}
                    />
                  )}
                </>
              )}
            </NavLink>
          );
        })}
      </nav>

      {/* Bottom Section */}
      <div className="p-6 border-t border-[#2A2D3A]">
        {/* Connection Status */}
        <div className="mb-4">
          <ConnectionStatus variant="detailed" />
        </div>

        {/* Quick Actions */}
        <div className="space-y-2">
          <button className="w-full flex items-center justify-center px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white text-sm font-medium rounded-lg transition-colors">
            <BoltIcon className="w-4 h-4 mr-2" />
            Quick Trade
          </button>
          <button className="w-full flex items-center justify-center px-4 py-2 bg-[#2A2D3A] hover:bg-[#3A3D4A] text-gray-300 hover:text-white text-sm font-medium rounded-lg transition-colors">
            <CpuChipIcon className="w-4 h-4 mr-2" />
            Ask Cerebro AI
          </button>
        </div>
      </div>
    </div>
  );
};

export default Sidebar;
