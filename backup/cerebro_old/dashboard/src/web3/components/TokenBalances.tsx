import React from 'react';
import { motion } from 'framer-motion';
import { useWallet } from '@solana/wallet-adapter-react';
import { ArrowPathIcon } from '@heroicons/react/24/outline';
import { useTokenAccounts } from '@/web3/hooks/useTokenAccounts';

interface TokenBalancesProps {
  className?: string;
}

const TokenBalances: React.FC<TokenBalancesProps> = ({ className = '' }) => {
  const { publicKey } = useWallet();
  const { tokenBalances, loading, error, refresh } = useTokenAccounts(publicKey);

  if (!publicKey) {
    return (
      <div className={`bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6 ${className}`}>
        <h3 className="text-lg font-semibold text-white mb-4">Token Balances</h3>
        <div className="text-center py-8">
          <p className="text-gray-400">Connect your wallet to view token balances</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-6 ${className}`}>
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-semibold text-white">Token Balances</h3>
        <button
          onClick={refresh}
          disabled={loading}
          className="p-2 rounded-lg hover:bg-[#2A2D3A] transition-colors disabled:opacity-50"
        >
          <ArrowPathIcon className={`w-4 h-4 text-gray-400 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg">
          <p className="text-red-400 text-sm">{error}</p>
        </div>
      )}

      {loading ? (
        <div className="space-y-3">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="animate-pulse">
              <div className="flex items-center space-x-3 p-3 bg-[#0F1419] rounded-lg">
                <div className="w-8 h-8 bg-[#2A2D3A] rounded-full"></div>
                <div className="flex-1">
                  <div className="h-4 bg-[#2A2D3A] rounded w-16 mb-1"></div>
                  <div className="h-3 bg-[#2A2D3A] rounded w-24"></div>
                </div>
                <div className="text-right">
                  <div className="h-4 bg-[#2A2D3A] rounded w-20 mb-1"></div>
                  <div className="h-3 bg-[#2A2D3A] rounded w-16"></div>
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : tokenBalances.length === 0 ? (
        <div className="text-center py-8">
          <p className="text-gray-400">No token balances found</p>
          <p className="text-gray-500 text-sm mt-1">
            Your wallet doesn't hold any SPL tokens
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {tokenBalances.map((token, index) => (
            <motion.div
              key={token.mint}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.1 }}
              className="flex items-center space-x-3 p-3 bg-[#0F1419] rounded-lg border border-[#2A2D3A] hover:border-[#3A3D4A] transition-colors"
            >
              {/* Token Icon */}
              <div className="w-8 h-8 rounded-full overflow-hidden bg-[#2A2D3A] flex items-center justify-center">
                {token.logoURI ? (
                  <img
                    src={token.logoURI}
                    alt={token.symbol}
                    className="w-full h-full object-cover"
                    onError={(e) => {
                      const target = e.target as HTMLImageElement;
                      target.style.display = 'none';
                      target.nextElementSibling?.classList.remove('hidden');
                    }}
                  />
                ) : null}
                <div className={`text-xs font-semibold text-gray-400 ${token.logoURI ? 'hidden' : ''}`}>
                  {token.symbol.slice(0, 2)}
                </div>
              </div>

              {/* Token Info */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center space-x-2">
                  <p className="font-medium text-white truncate">{token.symbol}</p>
                  {token.name !== token.symbol && (
                    <p className="text-xs text-gray-400 truncate">{token.name}</p>
                  )}
                </div>
                <p className="text-xs text-gray-500 truncate">
                  {token.mint.slice(0, 8)}...{token.mint.slice(-8)}
                </p>
              </div>

              {/* Balance */}
              <div className="text-right">
                <p className="font-semibold text-white">
                  {token.uiAmount.toLocaleString(undefined, {
                    minimumFractionDigits: 0,
                    maximumFractionDigits: 6,
                  })}
                </p>
                {token.value && token.value > 0 && (
                  <p className="text-xs text-gray-400">
                    ${token.value.toFixed(2)}
                  </p>
                )}
              </div>
            </motion.div>
          ))}
        </div>
      )}

      {/* Summary */}
      {tokenBalances.length > 0 && (
        <div className="mt-6 pt-4 border-t border-[#2A2D3A]">
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Total Tokens</span>
            <span className="text-white font-medium">{tokenBalances.length}</span>
          </div>
          {/* TODO: Add total USD value when price data is available */}
        </div>
      )}
    </div>
  );
};

export default TokenBalances;
