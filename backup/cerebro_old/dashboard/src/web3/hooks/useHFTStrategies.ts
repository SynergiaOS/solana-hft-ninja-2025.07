import { useState, useEffect, useCallback } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { hftTradingProgram, StrategyAccount, StrategyConfig } from '@/blockchain/contracts/HFTTradingProgram';
import { solanaService } from '@/web3/services/SolanaService';

interface UseHFTStrategiesReturn {
  strategies: StrategyAccount[];
  loading: boolean;
  error: string | null;
  createStrategy: (config: StrategyConfig) => Promise<string>;
  updateStrategy: (strategyAccount: PublicKey, config: StrategyConfig) => Promise<string>;
  toggleStrategy: (strategyAccount: PublicKey, enabled: boolean) => Promise<string>;
  closeStrategy: (strategyAccount: PublicKey) => Promise<string>;
  refresh: () => Promise<void>;
}

export const useHFTStrategies = (): UseHFTStrategiesReturn => {
  const { publicKey, sendTransaction, wallet } = useWallet();
  const [strategies, setStrategies] = useState<StrategyAccount[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStrategies = useCallback(async () => {
    if (!publicKey) {
      setStrategies([]);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const walletStrategies = await hftTradingProgram.getWalletStrategies(publicKey);
      setStrategies(walletStrategies);
    } catch (err) {
      console.error('Error fetching HFT strategies:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch strategies');
      setStrategies([]);
    } finally {
      setLoading(false);
    }
  }, [publicKey]);

  // Fetch strategies on mount and when wallet changes
  useEffect(() => {
    fetchStrategies();
  }, [fetchStrategies]);

  // Create new strategy
  const createStrategy = useCallback(async (config: StrategyConfig): Promise<string> => {
    if (!wallet || !publicKey || !sendTransaction) {
      throw new Error('Wallet not connected');
    }

    try {
      const { transaction, strategyAccount } = await hftTradingProgram.createStrategy(wallet, config);
      
      // Get recent blockhash
      const { blockhash } = await solanaService.getConnection().getLatestBlockhash();
      transaction.recentBlockhash = blockhash;
      transaction.feePayer = publicKey;

      // Send transaction
      const signature = await sendTransaction(transaction, solanaService.getConnection());
      
      // Wait for confirmation
      const confirmed = await solanaService.confirmTransaction(signature);
      if (!confirmed) {
        throw new Error('Transaction failed to confirm');
      }

      // Refresh strategies
      await fetchStrategies();

      return signature;
    } catch (err) {
      console.error('Error creating strategy:', err);
      throw err;
    }
  }, [wallet, publicKey, sendTransaction, fetchStrategies]);

  // Update strategy configuration
  const updateStrategy = useCallback(async (
    strategyAccount: PublicKey,
    config: StrategyConfig
  ): Promise<string> => {
    if (!wallet || !publicKey || !sendTransaction) {
      throw new Error('Wallet not connected');
    }

    try {
      const transaction = await hftTradingProgram.updateStrategy(wallet, strategyAccount, config);
      
      // Get recent blockhash
      const { blockhash } = await solanaService.getConnection().getLatestBlockhash();
      transaction.recentBlockhash = blockhash;
      transaction.feePayer = publicKey;

      // Send transaction
      const signature = await sendTransaction(transaction, solanaService.getConnection());
      
      // Wait for confirmation
      const confirmed = await solanaService.confirmTransaction(signature);
      if (!confirmed) {
        throw new Error('Transaction failed to confirm');
      }

      // Refresh strategies
      await fetchStrategies();

      return signature;
    } catch (err) {
      console.error('Error updating strategy:', err);
      throw err;
    }
  }, [wallet, publicKey, sendTransaction, fetchStrategies]);

  // Toggle strategy (pause/resume)
  const toggleStrategy = useCallback(async (
    strategyAccount: PublicKey,
    enabled: boolean
  ): Promise<string> => {
    if (!wallet || !publicKey || !sendTransaction) {
      throw new Error('Wallet not connected');
    }

    try {
      const transaction = await hftTradingProgram.toggleStrategy(wallet, strategyAccount, enabled);
      
      // Get recent blockhash
      const { blockhash } = await solanaService.getConnection().getLatestBlockhash();
      transaction.recentBlockhash = blockhash;
      transaction.feePayer = publicKey;

      // Send transaction
      const signature = await sendTransaction(transaction, solanaService.getConnection());
      
      // Wait for confirmation
      const confirmed = await solanaService.confirmTransaction(signature);
      if (!confirmed) {
        throw new Error('Transaction failed to confirm');
      }

      // Refresh strategies
      await fetchStrategies();

      return signature;
    } catch (err) {
      console.error('Error toggling strategy:', err);
      throw err;
    }
  }, [wallet, publicKey, sendTransaction, fetchStrategies]);

  // Close strategy
  const closeStrategy = useCallback(async (strategyAccount: PublicKey): Promise<string> => {
    if (!wallet || !publicKey || !sendTransaction) {
      throw new Error('Wallet not connected');
    }

    try {
      const transaction = await hftTradingProgram.closeStrategy(wallet, strategyAccount);
      
      // Get recent blockhash
      const { blockhash } = await solanaService.getConnection().getLatestBlockhash();
      transaction.recentBlockhash = blockhash;
      transaction.feePayer = publicKey;

      // Send transaction
      const signature = await sendTransaction(transaction, solanaService.getConnection());
      
      // Wait for confirmation
      const confirmed = await solanaService.confirmTransaction(signature);
      if (!confirmed) {
        throw new Error('Transaction failed to confirm');
      }

      // Refresh strategies
      await fetchStrategies();

      return signature;
    } catch (err) {
      console.error('Error closing strategy:', err);
      throw err;
    }
  }, [wallet, publicKey, sendTransaction, fetchStrategies]);

  return {
    strategies,
    loading,
    error,
    createStrategy,
    updateStrategy,
    toggleStrategy,
    closeStrategy,
    refresh: fetchStrategies,
  };
};
