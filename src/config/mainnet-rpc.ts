/**
 * 🌐 MAINNET RPC CONFIGURATION
 * Premium RPC providers with fallback and monitoring
 */

import { Connection, ConnectionConfig } from '@solana/web3.js';

export interface RPCProvider {
  name: string;
  endpoint: string;
  rateLimit: number; // requests per second
  cost: number; // USD per month
  features: string[];
  priority: number; // 1 = primary, 2 = secondary, etc.
}

export interface RPCConfig {
  providers: RPCProvider[];
  fallbackEnabled: boolean;
  healthCheckInterval: number; // milliseconds
  maxRetries: number;
  timeoutMs: number;
}

// Production RPC Configuration
export const MAINNET_RPC_CONFIG: RPCConfig = {
  providers: [
    {
      name: 'Helius',
      endpoint: `https://mainnet.helius-rpc.com/?api-key=${process.env.HELIUS_API_KEY}`,
      rateLimit: 1000, // 1000 req/sec
      cost: 99, // $99/month
      features: ['WebSocket', 'Enhanced APIs', 'Priority Access', 'DAS API'],
      priority: 1
    },
    {
      name: 'QuickNode',
      endpoint: process.env.QUICKNODE_ENDPOINT || '',
      rateLimit: 500, // 500 req/sec
      cost: 49, // $49/month
      features: ['Global Infrastructure', 'Analytics', 'Webhooks'],
      priority: 2
    },
    {
      name: 'Solana Labs',
      endpoint: 'https://api.mainnet-beta.solana.com',
      rateLimit: 100, // 100 req/sec (free tier)
      cost: 0,
      features: ['Basic RPC'],
      priority: 3
    }
  ],
  fallbackEnabled: true,
  healthCheckInterval: 30000, // 30 seconds
  maxRetries: 3,
  timeoutMs: 10000 // 10 seconds
};

// Connection configuration for optimal performance
export const CONNECTION_CONFIG: ConnectionConfig = {
  commitment: 'confirmed',
  confirmTransactionInitialTimeout: 60000, // 60 seconds
  disableRetryOnRateLimit: false,
  httpHeaders: {
    'User-Agent': 'Cerebro-HFT-Ninja/1.0',
    'Content-Type': 'application/json'
  }
};

// WebSocket configuration
export const WEBSOCKET_CONFIG = {
  endpoint: `wss://mainnet.helius-rpc.com/?api-key=${process.env.HELIUS_API_KEY}`,
  reconnectInterval: 5000, // 5 seconds
  maxReconnectAttempts: 10,
  pingInterval: 30000, // 30 seconds
  pongTimeout: 5000 // 5 seconds
};

/**
 * RPC Connection Manager with automatic failover
 */
export class MainnetRPCManager {
  private connections: Map<string, Connection> = new Map();
  private currentProvider: RPCProvider;
  private healthStatus: Map<string, boolean> = new Map();
  private lastHealthCheck: Map<string, number> = new Map();

  constructor(private config: RPCConfig = MAINNET_RPC_CONFIG) {
    this.currentProvider = this.config.providers[0];
    this.initializeConnections();
    this.startHealthChecks();
  }

  private initializeConnections(): void {
    for (const provider of this.config.providers) {
      if (provider.endpoint) {
        const connection = new Connection(provider.endpoint, CONNECTION_CONFIG);
        this.connections.set(provider.name, connection);
        this.healthStatus.set(provider.name, true);
        console.log(`🔗 Initialized connection to ${provider.name}`);
      }
    }
  }

  /**
   * Get the current active connection
   */
  public getConnection(): Connection {
    const connection = this.connections.get(this.currentProvider.name);
    if (!connection) {
      throw new Error(`No connection available for ${this.currentProvider.name}`);
    }
    return connection;
  }

  /**
   * Get connection for specific provider
   */
  public getProviderConnection(providerName: string): Connection | undefined {
    return this.connections.get(providerName);
  }

  /**
   * Switch to next available provider
   */
  public async switchToNextProvider(): Promise<boolean> {
    const availableProviders = this.config.providers
      .filter(p => this.healthStatus.get(p.name) && p.name !== this.currentProvider.name)
      .sort((a, b) => a.priority - b.priority);

    if (availableProviders.length === 0) {
      console.error('❌ No healthy RPC providers available!');
      return false;
    }

    const nextProvider = availableProviders[0];
    console.log(`🔄 Switching from ${this.currentProvider.name} to ${nextProvider.name}`);
    
    this.currentProvider = nextProvider;
    return true;
  }

  /**
   * Health check for all providers
   */
  private async checkProviderHealth(provider: RPCProvider): Promise<boolean> {
    try {
      const connection = this.connections.get(provider.name);
      if (!connection) return false;

      const start = Date.now();
      const slot = await connection.getSlot();
      const latency = Date.now() - start;

      if (latency > 5000) { // 5 second timeout
        console.warn(`⚠️ High latency for ${provider.name}: ${latency}ms`);
        return false;
      }

      if (slot > 0) {
        this.lastHealthCheck.set(provider.name, Date.now());
        return true;
      }

      return false;
    } catch (error) {
      console.error(`❌ Health check failed for ${provider.name}:`, error);
      return false;
    }
  }

  /**
   * Start periodic health checks
   */
  private startHealthChecks(): void {
    setInterval(async () => {
      for (const provider of this.config.providers) {
        if (provider.endpoint) {
          const isHealthy = await this.checkProviderHealth(provider);
          const wasHealthy = this.healthStatus.get(provider.name);
          
          this.healthStatus.set(provider.name, isHealthy);

          // Log status changes
          if (wasHealthy && !isHealthy) {
            console.error(`🔴 ${provider.name} went unhealthy`);
            
            // Switch provider if current one failed
            if (provider.name === this.currentProvider.name) {
              await this.switchToNextProvider();
            }
          } else if (!wasHealthy && isHealthy) {
            console.log(`🟢 ${provider.name} is healthy again`);
          }
        }
      }
    }, this.config.healthCheckInterval);
  }

  /**
   * Get current provider status
   */
  public getStatus() {
    return {
      currentProvider: this.currentProvider.name,
      healthStatus: Object.fromEntries(this.healthStatus),
      lastHealthCheck: Object.fromEntries(this.lastHealthCheck),
      totalProviders: this.config.providers.length,
      healthyProviders: Array.from(this.healthStatus.values()).filter(Boolean).length
    };
  }

  /**
   * Execute RPC call with automatic retry and failover
   */
  public async executeWithRetry<T>(
    operation: (connection: Connection) => Promise<T>,
    maxRetries: number = this.config.maxRetries
  ): Promise<T> {
    let lastError: Error | undefined;

    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        const connection = this.getConnection();
        return await operation(connection);
      } catch (error) {
        lastError = error as Error;
        console.warn(`⚠️ RPC call failed (attempt ${attempt}/${maxRetries}):`, error);

        // Try switching provider on error
        if (attempt < maxRetries) {
          const switched = await this.switchToNextProvider();
          if (!switched) {
            break; // No more providers available
          }
          
          // Wait before retry
          await new Promise(resolve => setTimeout(resolve, 1000 * attempt));
        }
      }
    }

    throw new Error(`RPC operation failed after ${maxRetries} attempts: ${lastError?.message}`);
  }
}

// Global RPC manager instance
export const rpcManager = new MainnetRPCManager();

// Export convenience functions
export const getMainnetConnection = () => rpcManager.getConnection();
export const executeRPCWithRetry = <T>(operation: (connection: Connection) => Promise<T>) => 
  rpcManager.executeWithRetry(operation);

// Rate limiting helper
export class RPCRateLimiter {
  private requestCounts: Map<string, number> = new Map();
  private resetTimers: Map<string, NodeJS.Timeout> = new Map();

  public async checkRateLimit(providerName: string, limit: number): Promise<boolean> {
    const current = this.requestCounts.get(providerName) || 0;
    
    if (current >= limit) {
      return false; // Rate limit exceeded
    }

    // Increment counter
    this.requestCounts.set(providerName, current + 1);

    // Set reset timer if not exists
    if (!this.resetTimers.has(providerName)) {
      const timer = setTimeout(() => {
        this.requestCounts.set(providerName, 0);
        this.resetTimers.delete(providerName);
      }, 1000); // Reset every second

      this.resetTimers.set(providerName, timer);
    }

    return true;
  }
}

export const rateLimiter = new RPCRateLimiter();
