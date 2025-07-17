import { PublicKey, Transaction, VersionedTransaction, Connection } from '@solana/web3.js';
import { WalletAdapter } from '@solana/wallet-adapter-base';

// Wallet Types
export interface WalletState {
  connected: boolean;
  connecting: boolean;
  disconnecting: boolean;
  publicKey: PublicKey | null;
  wallet: WalletAdapter | null;
  balance: number;
  network: string;
}

export interface WalletContextType extends WalletState {
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  sendTransaction: (transaction: Transaction | VersionedTransaction) => Promise<string>;
  signTransaction: (transaction: Transaction | VersionedTransaction) => Promise<Transaction | VersionedTransaction>;
  signAllTransactions: (transactions: (Transaction | VersionedTransaction)[]) => Promise<(Transaction | VersionedTransaction)[]>;
}

// Token Types
export interface TokenAccount {
  pubkey: PublicKey;
  mint: PublicKey;
  owner: PublicKey;
  amount: number;
  decimals: number;
  uiAmount: number;
  symbol?: string;
  name?: string;
  logoURI?: string;
}

export interface TokenBalance {
  mint: string;
  amount: string;
  decimals: number;
  uiAmount: number;
  symbol: string;
  name: string;
  logoURI?: string;
  value?: number; // USD value
}

export interface TokenMetadata {
  mint: string;
  symbol: string;
  name: string;
  decimals: number;
  logoURI?: string;
  description?: string;
  website?: string;
  twitter?: string;
  coingeckoId?: string;
}

// Transaction Types
export interface TransactionDetails {
  signature: string;
  slot: number;
  blockTime: number | null;
  confirmationStatus: 'processed' | 'confirmed' | 'finalized';
  err: any;
  memo?: string;
  fee: number;
  preBalances: number[];
  postBalances: number[];
  preTokenBalances: any[];
  postTokenBalances: any[];
  logMessages: string[];
  computeUnitsConsumed?: number;
}

export interface TransactionInstruction {
  programId: PublicKey;
  accounts: {
    pubkey: PublicKey;
    isSigner: boolean;
    isWritable: boolean;
  }[];
  data: Buffer;
}

export interface ParsedTransaction {
  signature: string;
  blockTime: number;
  slot: number;
  fee: number;
  status: 'success' | 'failed';
  type: 'transfer' | 'swap' | 'stake' | 'vote' | 'unknown';
  amount?: number;
  token?: string;
  from?: string;
  to?: string;
  program?: string;
  instructions: TransactionInstruction[];
}

// Smart Contract Types
export interface ProgramAccount {
  pubkey: PublicKey;
  account: {
    data: Buffer;
    executable: boolean;
    lamports: number;
    owner: PublicKey;
    rentEpoch: number;
  };
}

export interface ProgramInterface {
  programId: PublicKey;
  idl?: any; // Anchor IDL
  methods: Record<string, (...args: any[]) => any>;
}

// DEX Types
export interface DexMarket {
  address: PublicKey;
  name: string;
  baseMint: PublicKey;
  quoteMint: PublicKey;
  baseSymbol: string;
  quoteSymbol: string;
  tickSize: number;
  minOrderSize: number;
}

export interface OrderBook {
  market: PublicKey;
  bids: Array<{
    price: number;
    size: number;
  }>;
  asks: Array<{
    price: number;
    size: number;
  }>;
}

export interface SwapQuote {
  inputMint: string;
  outputMint: string;
  inputAmount: number;
  outputAmount: number;
  priceImpact: number;
  fee: number;
  route: Array<{
    dex: string;
    inputMint: string;
    outputMint: string;
    inputAmount: number;
    outputAmount: number;
  }>;
}

// Trading Types
export interface TradingPosition {
  id: string;
  market: string;
  side: 'long' | 'short';
  size: number;
  entryPrice: number;
  currentPrice: number;
  pnl: number;
  pnlPercent: number;
  openTime: number;
  status: 'open' | 'closed' | 'liquidated';
}

export interface TradingOrder {
  id: string;
  market: string;
  side: 'buy' | 'sell';
  type: 'market' | 'limit' | 'stop';
  size: number;
  price?: number;
  stopPrice?: number;
  status: 'pending' | 'filled' | 'cancelled' | 'failed';
  createdAt: number;
  filledAt?: number;
  signature?: string;
}

// Strategy Types
export interface StrategyConfig {
  id: string;
  name: string;
  type: 'sandwich' | 'arbitrage' | 'liquidation' | 'market_making';
  enabled: boolean;
  parameters: Record<string, any>;
  riskLimits: {
    maxPositionSize: number;
    maxSlippage: number;
    stopLoss?: number;
    takeProfit?: number;
  };
}

export interface StrategyExecution {
  id: string;
  strategyId: string;
  status: 'pending' | 'executing' | 'completed' | 'failed';
  startTime: number;
  endTime?: number;
  transactions: string[];
  profit: number;
  gas: number;
  error?: string;
}

// Pool Types (for AMM interactions)
export interface LiquidityPool {
  address: PublicKey;
  tokenA: TokenMetadata;
  tokenB: TokenMetadata;
  reserveA: number;
  reserveB: number;
  lpTokenSupply: number;
  fee: number;
  apy?: number;
  volume24h?: number;
  tvl?: number;
}

export interface PoolPosition {
  pool: PublicKey;
  lpTokens: number;
  sharePercent: number;
  tokenAAmount: number;
  tokenBAmount: number;
  value: number;
  rewards?: Array<{
    mint: string;
    amount: number;
    value: number;
  }>;
}

// Network Types
export type Network = 'devnet' | 'testnet' | 'mainnet-beta';

export interface NetworkConfig {
  name: Network;
  rpcUrl: string;
  wsUrl?: string;
  explorerUrl: string;
  faucetUrl?: string;
}

// Error Types
export interface Web3Error {
  code: string;
  message: string;
  details?: any;
  transaction?: string;
}

// Event Types
export interface AccountChangeEvent {
  accountId: PublicKey;
  data: Buffer;
  lamports: number;
}

export interface TransactionEvent {
  signature: string;
  status: 'confirmed' | 'finalized' | 'failed';
  slot: number;
  blockTime: number;
}

// Subscription Types
export interface Subscription {
  id: number;
  type: 'account' | 'program' | 'logs' | 'signature';
  callback: (data: any) => void;
  unsubscribe: () => void;
}

// Utility Types
export type Address = string | PublicKey;
export type TokenAmount = {
  amount: string;
  decimals: number;
  uiAmount: number;
  uiAmountString: string;
};

// Export all types
export * from '@solana/web3.js';
export * from '@solana/wallet-adapter-base';
