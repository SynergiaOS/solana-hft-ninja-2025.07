import { useState, useEffect, useCallback } from 'react';
import { PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';

interface UseBalanceReturn {
  balance: number;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export const useBalance = (publicKey: PublicKey | null): UseBalanceReturn => {
  const { connection } = useConnection();
  const [balance, setBalance] = useState<number>(0);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const fetchBalance = useCallback(async () => {
    if (!publicKey || !connection) {
      setBalance(0);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const lamports = await connection.getBalance(publicKey);
      const solBalance = lamports / LAMPORTS_PER_SOL;
      setBalance(solBalance);
    } catch (err) {
      console.error('Error fetching balance:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch balance');
      setBalance(0);
    } finally {
      setLoading(false);
    }
  }, [publicKey, connection]);

  // Fetch balance on mount and when dependencies change
  useEffect(() => {
    fetchBalance();
  }, [fetchBalance]);

  // Set up account change subscription for real-time updates
  useEffect(() => {
    if (!publicKey || !connection) return;

    let subscriptionId: number | null = null;

    const setupSubscription = async () => {
      try {
        subscriptionId = connection.onAccountChange(
          publicKey,
          (accountInfo) => {
            const solBalance = accountInfo.lamports / LAMPORTS_PER_SOL;
            setBalance(solBalance);
          },
          'confirmed'
        );
      } catch (err) {
        console.error('Error setting up balance subscription:', err);
      }
    };

    setupSubscription();

    return () => {
      if (subscriptionId !== null) {
        connection.removeAccountChangeListener(subscriptionId);
      }
    };
  }, [publicKey, connection]);

  return {
    balance,
    loading,
    error,
    refresh: fetchBalance,
  };
};
