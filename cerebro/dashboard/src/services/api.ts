import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import { PublicKey } from '@solana/web3.js';

// API Configuration
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8002';
const API_TIMEOUT = 30000; // 30 seconds

// Request/Response Types
export interface ApiResponse<T = any> {
  success: boolean;
  data: T;
  message?: string;
  error?: string;
}

export interface WalletProfile {
  address: string;
  network: string;
  balance: number;
  tokenAccounts: any[];
  strategies: any[];
  lastSync: string;
  createdAt: string;
  updatedAt: string;
}

export interface PortfolioData {
  totalValue: number;
  solBalance: number;
  tokenBalances: Array<{
    mint: string;
    symbol: string;
    amount: number;
    value: number;
  }>;
  performance: {
    daily: number;
    weekly: number;
    monthly: number;
  };
}

export interface StrategyData {
  id: string;
  name: string;
  type: string;
  status: string;
  config: any;
  metrics: {
    totalTrades: number;
    successRate: number;
    totalProfit: number;
    avgLatency: number;
  };
  createdAt: string;
  updatedAt: string;
}

// API Client Class
class ApiClient {
  private client: AxiosInstance;
  private authToken: string | null = null;

  constructor() {
    this.client = axios.create({
      baseURL: API_BASE_URL,
      timeout: API_TIMEOUT,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    this.setupInterceptors();
  }

  private setupInterceptors() {
    // Request interceptor
    this.client.interceptors.request.use(
      (config) => {
        // Add auth token if available
        if (this.authToken) {
          config.headers.Authorization = `Bearer ${this.authToken}`;
        }

        // Add wallet address if available
        const walletAddress = this.getStoredWalletAddress();
        if (walletAddress) {
          config.headers['X-Wallet-Address'] = walletAddress;
        }

        console.log(`API Request: ${config.method?.toUpperCase()} ${config.url}`);
        return config;
      },
      (error) => {
        console.error('API Request Error:', error);
        return Promise.reject(error);
      }
    );

    // Response interceptor
    this.client.interceptors.response.use(
      (response: AxiosResponse<ApiResponse>) => {
        console.log(`API Response: ${response.status} ${response.config.url}`);
        return response;
      },
      (error) => {
        console.error('API Response Error:', error.response?.data || error.message);
        
        // Handle specific error cases
        if (error.response?.status === 401) {
          this.handleAuthError();
        }
        
        return Promise.reject(error);
      }
    );
  }

  private getStoredWalletAddress(): string | null {
    return localStorage.getItem('cerebro_wallet_address');
  }

  private handleAuthError() {
    // Clear auth token and redirect to login
    this.authToken = null;
    localStorage.removeItem('cerebro_auth_token');
    // Could emit event for app to handle
    window.dispatchEvent(new CustomEvent('auth_error'));
  }

  // Auth Methods
  setAuthToken(token: string) {
    this.authToken = token;
    localStorage.setItem('cerebro_auth_token', token);
  }

  clearAuthToken() {
    this.authToken = null;
    localStorage.removeItem('cerebro_auth_token');
  }

  loadAuthToken() {
    const token = localStorage.getItem('cerebro_auth_token');
    if (token) {
      this.authToken = token;
    }
  }

  // Generic API Methods
  async get<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.get<ApiResponse<T>>(url, config);
    return response.data.data;
  }

  async post<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.post<ApiResponse<T>>(url, data, config);
    return response.data.data;
  }

  async put<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.put<ApiResponse<T>>(url, data, config);
    return response.data.data;
  }

  async delete<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.delete<ApiResponse<T>>(url, config);
    return response.data.data;
  }

  // Wallet API Methods
  async connectWallet(walletAddress: string, signature: string, message: string): Promise<{ token: string; profile: WalletProfile }> {
    const response = await this.post<{ token: string; profile: WalletProfile }>('/api/wallet/connect', {
      address: walletAddress,
      signature,
      message,
      network: 'devnet', // or get from config
    });

    // Store auth token
    this.setAuthToken(response.token);
    localStorage.setItem('cerebro_wallet_address', walletAddress);

    return response;
  }

  async getWalletProfile(): Promise<WalletProfile> {
    return this.get<WalletProfile>('/api/wallet/profile');
  }

  async syncWallet(): Promise<WalletProfile> {
    return this.post<WalletProfile>('/api/wallet/sync');
  }

  async disconnectWallet(): Promise<void> {
    await this.delete('/api/wallet/disconnect');
    this.clearAuthToken();
    localStorage.removeItem('cerebro_wallet_address');
  }

  // Portfolio API Methods
  async getPortfolio(): Promise<PortfolioData> {
    return this.get<PortfolioData>('/api/portfolio');
  }

  async getBalances(): Promise<any[]> {
    return this.get<any[]>('/api/portfolio/balances');
  }

  async getTokens(): Promise<any[]> {
    return this.get<any[]>('/api/portfolio/tokens');
  }

  async getTransactionHistory(limit: number = 50): Promise<any[]> {
    return this.get<any[]>(`/api/portfolio/history?limit=${limit}`);
  }

  async syncPortfolio(): Promise<PortfolioData> {
    return this.post<PortfolioData>('/api/portfolio/sync');
  }

  // Strategy API Methods
  async getStrategies(): Promise<StrategyData[]> {
    return this.get<StrategyData[]>('/api/strategies');
  }

  async getStrategy(id: string): Promise<StrategyData> {
    return this.get<StrategyData>(`/api/strategies/${id}`);
  }

  async createStrategy(strategy: Partial<StrategyData>): Promise<StrategyData> {
    return this.post<StrategyData>('/api/strategies', strategy);
  }

  async updateStrategy(id: string, strategy: Partial<StrategyData>): Promise<StrategyData> {
    return this.put<StrategyData>(`/api/strategies/${id}`, strategy);
  }

  async deleteStrategy(id: string): Promise<void> {
    return this.delete(`/api/strategies/${id}`);
  }

  async toggleStrategy(id: string, enabled: boolean): Promise<StrategyData> {
    return this.put<StrategyData>(`/api/strategies/${id}/toggle`, { enabled });
  }

  // Trading API Methods
  async executeTrade(tradeData: any): Promise<{ signature: string }> {
    return this.post<{ signature: string }>('/api/trading/execute', tradeData);
  }

  async getTradeHistory(limit: number = 50): Promise<any[]> {
    return this.get<any[]>(`/api/trading/history?limit=${limit}`);
  }

  // System API Methods
  async getSystemHealth(): Promise<any> {
    return this.get('/api/system/health');
  }

  async getSystemMetrics(): Promise<any> {
    return this.get('/api/system/metrics');
  }

  // WebSocket connection info
  getWebSocketUrl(): string {
    const wsProtocol = API_BASE_URL.startsWith('https') ? 'wss' : 'ws';
    const wsUrl = API_BASE_URL.replace(/^https?/, wsProtocol);
    return `${wsUrl}/ws`;
  }
}

// Export singleton instance
export const apiClient = new ApiClient();

// Initialize auth token on app start
apiClient.loadAuthToken();
