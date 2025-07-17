import { Connection, clusterApiUrl, Commitment } from '@solana/web3.js';

// Network configuration
export type Network = 'devnet' | 'testnet' | 'mainnet-beta';

export const NETWORKS = {
  devnet: 'devnet',
  testnet: 'testnet',
  'mainnet-beta': 'mainnet-beta',
} as const;

// RPC endpoints
export const RPC_ENDPOINTS = {
  devnet: clusterApiUrl('devnet'),
  testnet: clusterApiUrl('testnet'),
  'mainnet-beta': clusterApiUrl('mainnet-beta'),
  // Custom RPC endpoints for better performance
  helius: 'https://rpc.helius.xyz/?api-key=your-api-key',
  quicknode: 'https://your-endpoint.solana-mainnet.quiknode.pro/your-api-key/',
  alchemy: 'https://solana-mainnet.g.alchemy.com/v2/your-api-key',
} as const;

// Default network for development
export const DEFAULT_NETWORK: Network = 'devnet';

// Connection configuration
export const CONNECTION_CONFIG = {
  commitment: 'confirmed' as Commitment,
  wsEndpoint: undefined,
  disableRetryOnRateLimit: false,
  confirmTransactionInitialTimeout: 60000,
};

// Create connection instance
export const createConnection = (network: Network = DEFAULT_NETWORK): Connection => {
  const endpoint = RPC_ENDPOINTS[network];
  return new Connection(endpoint, CONNECTION_CONFIG);
};

// Default connection instance
export const connection = createConnection(DEFAULT_NETWORK);

// WebSocket connection for real-time updates
export const createWebSocketConnection = (network: Network = DEFAULT_NETWORK): Connection => {
  const endpoint = RPC_ENDPOINTS[network];
  const wsEndpoint = endpoint.replace('https://', 'wss://').replace('http://', 'ws://');
  
  return new Connection(endpoint, {
    ...CONNECTION_CONFIG,
    wsEndpoint,
  });
};

// Connection health check
export const checkConnectionHealth = async (conn: Connection): Promise<boolean> => {
  try {
    const version = await conn.getVersion();
    return !!version;
  } catch (error) {
    console.error('Connection health check failed:', error);
    return false;
  }
};

// Get network from connection
export const getNetworkFromConnection = (conn: Connection): Network => {
  const endpoint = conn.rpcEndpoint;
  
  if (endpoint.includes('devnet')) return 'devnet';
  if (endpoint.includes('testnet')) return 'testnet';
  if (endpoint.includes('mainnet')) return 'mainnet-beta';
  
  return DEFAULT_NETWORK;
};

// Connection utilities
export const connectionUtils = {
  createConnection,
  createWebSocketConnection,
  checkConnectionHealth,
  getNetworkFromConnection,
};
