import React, { useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { motion, AnimatePresence } from 'framer-motion';
import {
  WalletIcon,
  ChevronDownIcon,
  ArrowRightOnRectangleIcon,
  DocumentDuplicateIcon,
  CheckIcon,
  ExclamationTriangleIcon,
} from '@heroicons/react/24/outline';
import { useBalance } from '@/web3/hooks/useBalance';
import { useWalletStore } from '@/stores/walletStore';

interface WalletConnectButtonProps {
  className?: string;
  showBalance?: boolean;
  variant?: 'default' | 'compact' | 'icon';
}

const WalletConnectButton: React.FC<WalletConnectButtonProps> = ({
  className = '',
  showBalance = true,
  variant = 'default',
}) => {
  const { connected, connecting, publicKey, disconnect, wallet } = useWallet();
  const { balance, loading: balanceLoading } = useBalance(publicKey);
  const {
    connected: backendConnected,
    connecting: backendConnecting,
    autoConnecting,
    profile,
    error: backendError,
    wsConnected,
    connectToBackend,
    disconnectFromBackend,
  } = useWalletStore();
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleCopyAddress = async () => {
    if (publicKey) {
      await navigator.clipboard.writeText(publicKey.toString());
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const formatAddress = (address: string) => {
    return `${address.slice(0, 4)}...${address.slice(-4)}`;
  };

  const formatBalance = (balance: number) => {
    return balance.toFixed(4);
  };

  if (!connected) {
    return (
      <div className={`relative ${className}`}>
        <WalletMultiButton className="!bg-purple-600 hover:!bg-purple-700 !text-white !rounded-lg !font-medium !transition-colors !border-none !h-10 !px-4">
          {connecting || autoConnecting ? (
            <div className="flex items-center space-x-2">
              <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              <span>{autoConnecting ? 'Auto-connecting...' : 'Connecting...'}</span>
            </div>
          ) : (
            <div className="flex items-center space-x-2">
              <WalletIcon className="w-4 h-4" />
              <span>Connect Wallet</span>
            </div>
          )}
        </WalletMultiButton>

        {backendError && (
          <div className="absolute top-full left-0 mt-2 p-2 bg-red-500/10 border border-red-500/20 rounded-lg text-xs text-red-400 whitespace-nowrap z-50">
            <ExclamationTriangleIcon className="w-3 h-3 inline mr-1" />
            {backendError}
          </div>
        )}
      </div>
    );
  }

  if (variant === 'icon') {
    return (
      <button
        onClick={() => setDropdownOpen(!dropdownOpen)}
        className={`relative p-2 rounded-lg bg-[#1A1D29] border border-[#2A2D3A] hover:border-[#3A3D4A] transition-colors ${className}`}
      >
        <div className="w-6 h-6 bg-gradient-to-br from-purple-500 to-purple-700 rounded-full flex items-center justify-center">
          <WalletIcon className="w-4 h-4 text-white" />
        </div>
        {dropdownOpen && (
          <WalletDropdown
            publicKey={publicKey}
            balance={balance}
            balanceLoading={balanceLoading}
            showBalance={showBalance}
            onCopyAddress={handleCopyAddress}
            onDisconnect={disconnect}
            copied={copied}
            onClose={() => setDropdownOpen(false)}
          />
        )}
      </button>
    );
  }

  if (variant === 'compact') {
    return (
      <div className={`relative ${className}`}>
        <button
          onClick={() => setDropdownOpen(!dropdownOpen)}
          className="flex items-center space-x-2 px-3 py-2 bg-[#1A1D29] border border-[#2A2D3A] rounded-lg hover:border-[#3A3D4A] transition-colors"
        >
          <div className="w-6 h-6 bg-gradient-to-br from-purple-500 to-purple-700 rounded-full flex items-center justify-center">
            <WalletIcon className="w-3 h-3 text-white" />
          </div>
          <span className="text-sm text-white font-medium">
            {formatAddress(publicKey?.toString() || '')}
          </span>
          <ChevronDownIcon className="w-4 h-4 text-gray-400" />
        </button>

        {dropdownOpen && (
          <WalletDropdown
            publicKey={publicKey}
            balance={balance}
            balanceLoading={balanceLoading}
            showBalance={showBalance}
            onCopyAddress={handleCopyAddress}
            onDisconnect={disconnect}
            copied={copied}
            onClose={() => setDropdownOpen(false)}
          />
        )}
      </div>
    );
  }

  // Default variant
  return (
    <div className={`relative ${className}`}>
      <button
        onClick={() => setDropdownOpen(!dropdownOpen)}
        className="flex items-center space-x-3 px-4 py-2 bg-[#1A1D29] border border-[#2A2D3A] rounded-lg hover:border-[#3A3D4A] transition-colors"
      >
        <div className="flex items-center space-x-2">
          <div className="w-8 h-8 bg-gradient-to-br from-purple-500 to-purple-700 rounded-full flex items-center justify-center">
            <WalletIcon className="w-4 h-4 text-white" />
          </div>
          <div className="text-left">
            <p className="text-sm font-medium text-white">
              {wallet?.adapter.name || 'Wallet'}
            </p>
            <p className="text-xs text-gray-400">
              {formatAddress(publicKey?.toString() || '')}
            </p>
          </div>
        </div>

        {showBalance && (
          <div className="text-right">
            <p className="text-sm font-medium text-white">
              {balanceLoading ? (
                <div className="w-16 h-4 bg-[#2A2D3A] rounded animate-pulse"></div>
              ) : (
                `${formatBalance(balance)} SOL`
              )}
            </p>
            <p className="text-xs text-gray-400">Balance</p>
          </div>
        )}

        <ChevronDownIcon className="w-4 h-4 text-gray-400" />
      </button>

      {dropdownOpen && (
        <WalletDropdown
          publicKey={publicKey}
          balance={balance}
          balanceLoading={balanceLoading}
          showBalance={showBalance}
          onCopyAddress={handleCopyAddress}
          onDisconnect={disconnect}
          copied={copied}
          onClose={() => setDropdownOpen(false)}
        />
      )}
    </div>
  );
};

interface WalletDropdownProps {
  publicKey: any;
  balance: number;
  balanceLoading: boolean;
  showBalance: boolean;
  onCopyAddress: () => void;
  onDisconnect: () => void;
  copied: boolean;
  onClose: () => void;
}

const WalletDropdown: React.FC<WalletDropdownProps> = ({
  publicKey,
  balance,
  balanceLoading,
  showBalance,
  onCopyAddress,
  onDisconnect,
  copied,
  onClose,
}) => {
  return (
    <>
      {/* Overlay */}
      <div
        className="fixed inset-0 z-40"
        onClick={onClose}
      />
      
      {/* Dropdown */}
      <motion.div
        initial={{ opacity: 0, y: -10 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -10 }}
        className="absolute right-0 mt-2 w-64 bg-[#1A1D29] border border-[#2A2D3A] rounded-lg shadow-lg z-50"
      >
        <div className="p-4">
          {/* Address */}
          <div className="mb-4">
            <p className="text-xs text-gray-400 mb-1">Wallet Address</p>
            <div className="flex items-center justify-between">
              <p className="text-sm font-mono text-white">
                {publicKey?.toString().slice(0, 20)}...
              </p>
              <button
                onClick={onCopyAddress}
                className="p-1 rounded hover:bg-[#2A2D3A] transition-colors"
              >
                {copied ? (
                  <CheckIcon className="w-4 h-4 text-green-400" />
                ) : (
                  <DocumentDuplicateIcon className="w-4 h-4 text-gray-400" />
                )}
              </button>
            </div>
          </div>

          {/* Balance */}
          {showBalance && (
            <div className="mb-4">
              <p className="text-xs text-gray-400 mb-1">Balance</p>
              {balanceLoading ? (
                <div className="w-24 h-5 bg-[#2A2D3A] rounded animate-pulse"></div>
              ) : (
                <p className="text-lg font-semibold text-white">
                  {profile?.balance?.toFixed(4) || balance.toFixed(4)} SOL
                </p>
              )}
            </div>
          )}

          {/* Backend Status */}
          <div className="mb-4">
            <p className="text-xs text-gray-400 mb-2">Backend Status</p>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs text-gray-300">API Connection</span>
                <div className="flex items-center space-x-1">
                  <div className={`w-2 h-2 rounded-full ${
                    backendConnected ? 'bg-green-400' : 'bg-red-400'
                  }`}></div>
                  <span className={`text-xs ${
                    backendConnected ? 'text-green-400' : 'text-red-400'
                  }`}>
                    {backendConnected ? 'Connected' : 'Disconnected'}
                  </span>
                </div>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs text-gray-300">WebSocket</span>
                <div className="flex items-center space-x-1">
                  <div className={`w-2 h-2 rounded-full ${
                    wsConnected ? 'bg-green-400 animate-pulse' : 'bg-gray-400'
                  }`}></div>
                  <span className={`text-xs ${
                    wsConnected ? 'text-green-400' : 'text-gray-400'
                  }`}>
                    {wsConnected ? 'Live' : 'Offline'}
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Actions */}
          <div className="space-y-2">
            <button
              onClick={onDisconnect}
              className="w-full flex items-center justify-center space-x-2 px-3 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
            >
              <ArrowRightOnRectangleIcon className="w-4 h-4" />
              <span>Disconnect</span>
            </button>
          </div>
        </div>
      </motion.div>
    </>
  );
};

export default WalletConnectButton;
