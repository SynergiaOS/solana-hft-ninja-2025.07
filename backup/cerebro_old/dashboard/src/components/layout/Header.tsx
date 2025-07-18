import React, { useState } from 'react';
import { motion } from 'framer-motion';
import {
  Bars3Icon,
  BellIcon,
  MagnifyingGlassIcon,
  Cog6ToothIcon,
  UserCircleIcon,
  ChevronDownIcon,
  SunIcon,
  MoonIcon,
} from '@heroicons/react/24/outline';
import WalletConnectButton from '@/web3/components/WalletConnectButton';

interface HeaderProps {
  onMenuClick: () => void;
  onNotificationClick: () => void;
  notificationCount: number;
}

const Header: React.FC<HeaderProps> = ({
  onMenuClick,
  onNotificationClick,
  notificationCount,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [userMenuOpen, setUserMenuOpen] = useState(false);

  return (
    <header className="bg-[#1A1D29] border-b border-[#2A2D3A] px-6 py-4">
      <div className="flex items-center justify-between">
        {/* Left Section */}
        <div className="flex items-center space-x-4">
          {/* Menu Button */}
          <button
            onClick={onMenuClick}
            className="p-2 rounded-lg hover:bg-[#2A2D3A] transition-colors lg:hidden"
          >
            <Bars3Icon className="w-5 h-5 text-gray-400" />
          </button>

          {/* Search Bar */}
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
            </div>
            <input
              type="text"
              placeholder="Search strategies, trades, or ask Cerebro..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="block w-80 pl-10 pr-3 py-2 border border-[#2A2D3A] rounded-lg bg-[#0F1419] text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent"
            />
          </div>
        </div>

        {/* Right Section */}
        <div className="flex items-center space-x-4">
          {/* Portfolio Value */}
          <div className="hidden md:flex items-center space-x-4 px-4 py-2 bg-[#0F1419] rounded-lg border border-[#2A2D3A]">
            <div className="text-right">
              <p className="text-xs text-gray-400">Portfolio Value</p>
              <p className="text-sm font-semibold text-white">$47,892.34</p>
            </div>
            <div className="flex items-center space-x-1">
              <div className="w-2 h-2 bg-green-400 rounded-full"></div>
              <span className="text-xs text-green-400">+2.34%</span>
            </div>
          </div>

          {/* Quick Stats */}
          <div className="hidden lg:flex items-center space-x-6">
            <div className="text-center">
              <p className="text-xs text-gray-400">24h PnL</p>
              <p className="text-sm font-semibold text-green-400">+$1,247.89</p>
            </div>
            <div className="text-center">
              <p className="text-xs text-gray-400">Active Trades</p>
              <p className="text-sm font-semibold text-white">12</p>
            </div>
            <div className="text-center">
              <p className="text-xs text-gray-400">Success Rate</p>
              <p className="text-sm font-semibold text-purple-400">87.3%</p>
            </div>
          </div>

          {/* Notifications */}
          <button
            onClick={onNotificationClick}
            className="relative p-2 rounded-lg hover:bg-[#2A2D3A] transition-colors"
          >
            <BellIcon className="w-5 h-5 text-gray-400" />
            {notificationCount > 0 && (
              <motion.span
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                className="absolute -top-1 -right-1 w-5 h-5 bg-red-500 text-white text-xs rounded-full flex items-center justify-center"
              >
                {notificationCount > 9 ? '9+' : notificationCount}
              </motion.span>
            )}
          </button>

          {/* Wallet Connect */}
          <WalletConnectButton variant="compact" showBalance={false} />

          {/* Settings */}
          <button className="p-2 rounded-lg hover:bg-[#2A2D3A] transition-colors">
            <Cog6ToothIcon className="w-5 h-5 text-gray-400" />
          </button>

          {/* User Menu */}
          <div className="relative">
            <button
              onClick={() => setUserMenuOpen(!userMenuOpen)}
              className="flex items-center space-x-2 p-2 rounded-lg hover:bg-[#2A2D3A] transition-colors"
            >
              <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
                <span className="text-sm font-semibold text-white">BC</span>
              </div>
              <ChevronDownIcon className="w-4 h-4 text-gray-400" />
            </button>

            {/* User Dropdown */}
            {userMenuOpen && (
              <motion.div
                initial={{ opacity: 0, y: -10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                className="absolute right-0 mt-2 w-48 bg-[#1A1D29] border border-[#2A2D3A] rounded-lg shadow-lg z-50"
              >
                <div className="py-2">
                  <div className="px-4 py-2 border-b border-[#2A2D3A]">
                    <p className="text-sm font-medium text-white">Bryan Crawford</p>
                    <p className="text-xs text-gray-400">bryan@cerebro.ai</p>
                  </div>
                  <button className="w-full text-left px-4 py-2 text-sm text-gray-300 hover:text-white hover:bg-[#2A2D3A] transition-colors">
                    <UserCircleIcon className="w-4 h-4 inline mr-2" />
                    Profile
                  </button>
                  <button className="w-full text-left px-4 py-2 text-sm text-gray-300 hover:text-white hover:bg-[#2A2D3A] transition-colors">
                    <Cog6ToothIcon className="w-4 h-4 inline mr-2" />
                    Settings
                  </button>
                  <button className="w-full text-left px-4 py-2 text-sm text-gray-300 hover:text-white hover:bg-[#2A2D3A] transition-colors">
                    <MoonIcon className="w-4 h-4 inline mr-2" />
                    Dark Mode
                  </button>
                  <div className="border-t border-[#2A2D3A] mt-2 pt-2">
                    <button className="w-full text-left px-4 py-2 text-sm text-red-400 hover:text-red-300 hover:bg-[#2A2D3A] transition-colors">
                      Sign Out
                    </button>
                  </div>
                </div>
              </motion.div>
            )}
          </div>
        </div>
      </div>
    </header>
  );
};

export default Header;
