import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  WifiIcon,
  ExclamationTriangleIcon,
  CheckCircleIcon,
  XCircleIcon,
  ArrowPathIcon,
} from '@heroicons/react/24/outline';
import { useWallet } from '@solana/wallet-adapter-react';
import { useWalletStore } from '@/stores/walletStore';
import { useBackendSync } from '@/hooks/useBackendSync';
import { webSocketService } from '@/services/websocket';

interface ConnectionStatusProps {
  className?: string;
  variant?: 'compact' | 'detailed';
}

const ConnectionStatus: React.FC<ConnectionStatusProps> = ({
  className = '',
  variant = 'compact',
}) => {
  const { connected: walletConnected } = useWallet();
  const {
    connected: backendConnected,
    wsConnected,
    error: backendError,
    syncInProgress,
    lastSync,
  } = useWalletStore();
  const { manualSync } = useBackendSync();
  const [isExpanded, setIsExpanded] = useState(false);

  // Determine overall connection status
  const getOverallStatus = () => {
    if (!walletConnected) return 'wallet_disconnected';
    if (!backendConnected) return 'backend_disconnected';
    if (!wsConnected) return 'websocket_disconnected';
    return 'connected';
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'connected':
        return 'text-green-400';
      case 'websocket_disconnected':
        return 'text-yellow-400';
      case 'backend_disconnected':
      case 'wallet_disconnected':
        return 'text-red-400';
      default:
        return 'text-gray-400';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'connected':
        return <CheckCircleIcon className="w-4 h-4" />;
      case 'websocket_disconnected':
        return <ExclamationTriangleIcon className="w-4 h-4" />;
      case 'backend_disconnected':
      case 'wallet_disconnected':
        return <XCircleIcon className="w-4 h-4" />;
      default:
        return <WifiIcon className="w-4 h-4" />;
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'connected':
        return 'All Systems Online';
      case 'websocket_disconnected':
        return 'Real-time Offline';
      case 'backend_disconnected':
        return 'Backend Offline';
      case 'wallet_disconnected':
        return 'Wallet Disconnected';
      default:
        return 'Unknown Status';
    }
  };

  const overallStatus = getOverallStatus();
  const statusColor = getStatusColor(overallStatus);
  const statusIcon = getStatusIcon(overallStatus);
  const statusText = getStatusText(overallStatus);

  const handleManualSync = async () => {
    try {
      await manualSync();
    } catch (error) {
      console.error('Manual sync failed:', error);
    }
  };

  if (variant === 'compact') {
    return (
      <div className={`flex items-center space-x-2 ${className}`}>
        <div className={`${statusColor}`}>
          {statusIcon}
        </div>
        <span className={`text-sm ${statusColor}`}>
          {overallStatus === 'connected' ? 'Online' : 'Offline'}
        </span>
        {overallStatus === 'connected' && wsConnected && (
          <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
        )}
      </div>
    );
  }

  return (
    <div className={`bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-4 ${className}`}>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-2">
          <div className={statusColor}>
            {statusIcon}
          </div>
          <h3 className="text-lg font-semibold text-white">Connection Status</h3>
        </div>
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="text-gray-400 hover:text-white transition-colors"
        >
          <motion.div
            animate={{ rotate: isExpanded ? 180 : 0 }}
            transition={{ duration: 0.2 }}
          >
            <ArrowPathIcon className="w-4 h-4" />
          </motion.div>
        </button>
      </div>

      {/* Overall Status */}
      <div className="mb-4">
        <div className={`text-lg font-medium ${statusColor}`}>
          {statusText}
        </div>
        {backendError && (
          <div className="text-sm text-red-400 mt-1">
            {backendError}
          </div>
        )}
      </div>

      {/* Detailed Status */}
      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.3 }}
            className="space-y-3"
          >
            {/* Wallet Connection */}
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-300">Wallet</span>
              <div className="flex items-center space-x-2">
                <div className={`w-2 h-2 rounded-full ${
                  walletConnected ? 'bg-green-400' : 'bg-red-400'
                }`}></div>
                <span className={`text-sm ${
                  walletConnected ? 'text-green-400' : 'text-red-400'
                }`}>
                  {walletConnected ? 'Connected' : 'Disconnected'}
                </span>
              </div>
            </div>

            {/* Backend API */}
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-300">Backend API</span>
              <div className="flex items-center space-x-2">
                <div className={`w-2 h-2 rounded-full ${
                  backendConnected ? 'bg-green-400' : 'bg-red-400'
                }`}></div>
                <span className={`text-sm ${
                  backendConnected ? 'text-green-400' : 'text-red-400'
                }`}>
                  {backendConnected ? 'Connected' : 'Disconnected'}
                </span>
              </div>
            </div>

            {/* WebSocket */}
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-300">Real-time Data</span>
              <div className="flex items-center space-x-2">
                <div className={`w-2 h-2 rounded-full ${
                  wsConnected ? 'bg-green-400 animate-pulse' : 'bg-gray-400'
                }`}></div>
                <span className={`text-sm ${
                  wsConnected ? 'text-green-400' : 'text-gray-400'
                }`}>
                  {wsConnected ? 'Live' : 'Offline'}
                </span>
              </div>
            </div>

            {/* Last Sync */}
            {lastSync && (
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-300">Last Sync</span>
                <span className="text-sm text-gray-400">
                  {new Date(lastSync).toLocaleTimeString()}
                </span>
              </div>
            )}

            {/* Manual Sync Button */}
            {backendConnected && (
              <div className="pt-2 border-t border-[#2A2D3A]">
                <button
                  onClick={handleManualSync}
                  disabled={syncInProgress}
                  className="w-full flex items-center justify-center space-x-2 px-3 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-purple-600/50 text-white rounded-lg transition-colors"
                >
                  <ArrowPathIcon className={`w-4 h-4 ${syncInProgress ? 'animate-spin' : ''}`} />
                  <span>{syncInProgress ? 'Syncing...' : 'Manual Sync'}</span>
                </button>
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default ConnectionStatus;
