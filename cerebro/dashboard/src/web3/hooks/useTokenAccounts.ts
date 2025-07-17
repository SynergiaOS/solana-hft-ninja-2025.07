import { useState, useEffect, useCallback } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from '@solana/spl-token';
import { TokenAccount, TokenBalance } from '@/web3/types';

interface UseTokenAccountsReturn {
  tokenAccounts: TokenAccount[];
  tokenBalances: TokenBalance[];
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

// Common Solana tokens for devnet/mainnet
const KNOWN_TOKENS: Record<string, { symbol: string; name: string; decimals: number; logoURI?: string }> = {
  // Devnet tokens
  'So11111111111111111111111111111111111111112': {
    symbol: 'SOL',
    name: 'Solana',
    decimals: 9,
    logoURI: 'https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png'
  },
  'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v': {
    symbol: 'USDC',
    name: 'USD Coin',
    decimals: 6,
    logoURI: 'https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png'
  },
  'Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB': {
    symbol: 'USDT',
    name: 'Tether USD',
    decimals: 6,
    logoURI: 'https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB/logo.png'
  },
  '4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R': {
    symbol: 'RAY',
    name: 'Raydium',
    decimals: 6,
    logoURI: 'https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R/logo.png'
  },
  'orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE': {
    symbol: 'ORCA',
    name: 'Orca',
    decimals: 6,
    logoURI: 'https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE/logo.png'
  },
};

export const useTokenAccounts = (publicKey: PublicKey | null): UseTokenAccountsReturn => {
  const { connection } = useConnection();
  const [tokenAccounts, setTokenAccounts] = useState<TokenAccount[]>([]);
  const [tokenBalances, setTokenBalances] = useState<TokenBalance[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const fetchTokenAccounts = useCallback(async () => {
    if (!publicKey || !connection) {
      setTokenAccounts([]);
      setTokenBalances([]);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // Get all token accounts for the wallet
      const tokenAccountsResponse = await connection.getParsedTokenAccountsByOwner(
        publicKey,
        { programId: TOKEN_PROGRAM_ID }
      );

      const accounts: TokenAccount[] = [];
      const balances: TokenBalance[] = [];

      for (const accountInfo of tokenAccountsResponse.value) {
        const parsedInfo = accountInfo.account.data.parsed.info;
        const mint = new PublicKey(parsedInfo.mint);
        const amount = parseInt(parsedInfo.tokenAmount.amount);
        const decimals = parsedInfo.tokenAmount.decimals;
        const uiAmount = parsedInfo.tokenAmount.uiAmount || 0;

        // Skip accounts with zero balance
        if (amount === 0) continue;

        // Get token metadata
        const tokenInfo = KNOWN_TOKENS[mint.toString()] || {
          symbol: mint.toString().slice(0, 8),
          name: 'Unknown Token',
          decimals: decimals,
        };

        const tokenAccount: TokenAccount = {
          pubkey: accountInfo.pubkey,
          mint,
          owner: publicKey,
          amount,
          decimals,
          uiAmount,
          symbol: tokenInfo.symbol,
          name: tokenInfo.name,
          logoURI: tokenInfo.logoURI,
        };

        const tokenBalance: TokenBalance = {
          mint: mint.toString(),
          amount: amount.toString(),
          decimals,
          uiAmount,
          symbol: tokenInfo.symbol,
          name: tokenInfo.name,
          logoURI: tokenInfo.logoURI,
          // TODO: Add USD value calculation using price APIs
          value: 0,
        };

        accounts.push(tokenAccount);
        balances.push(tokenBalance);
      }

      setTokenAccounts(accounts);
      setTokenBalances(balances);
    } catch (err) {
      console.error('Error fetching token accounts:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch token accounts');
      setTokenAccounts([]);
      setTokenBalances([]);
    } finally {
      setLoading(false);
    }
  }, [publicKey, connection]);

  // Fetch token accounts on mount and when dependencies change
  useEffect(() => {
    fetchTokenAccounts();
  }, [fetchTokenAccounts]);

  return {
    tokenAccounts,
    tokenBalances,
    loading,
    error,
    refresh: fetchTokenAccounts,
  };
};
