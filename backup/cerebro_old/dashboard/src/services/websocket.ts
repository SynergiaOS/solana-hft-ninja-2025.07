import { apiClient } from './api';

// WebSocket Event Types
export interface WebSocketEvent {
  type: string;
  data: any;
  timestamp: number;
}

export interface WalletBalanceEvent {
  type: 'wallet.balance_updated';
  data: {
    address: string;
    balance: number;
    change: number;
  };
}

export interface PortfolioUpdateEvent {
  type: 'portfolio.updated';
  data: {
    totalValue: number;
    tokens: Array<{
      mint: string;
      symbol: string;
      amount: number;
      value: number;
    }>;
  };
}

export interface StrategyStatusEvent {
  type: 'strategy.status_changed';
  data: {
    id: string;
    status: string;
    metrics: any;
  };
}

export interface TradingExecutionEvent {
  type: 'trading.execution_completed';
  data: {
    strategyId: string;
    signature: string;
    profit: number;
    status: 'success' | 'failed';
  };
}

export type CerebroWebSocketEvent = 
  | WalletBalanceEvent 
  | PortfolioUpdateEvent 
  | StrategyStatusEvent 
  | TradingExecutionEvent;

// Event Listener Type
export type EventListener<T = any> = (event: T) => void;

// WebSocket Service Class
class WebSocketService {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000; // Start with 1 second
  private maxReconnectDelay = 30000; // Max 30 seconds
  private listeners: Map<string, Set<EventListener>> = new Map();
  private isConnecting = false;
  private shouldReconnect = true;
  private heartbeatInterval: NodeJS.Timeout | null = null;
  private connectionTimeout: NodeJS.Timeout | null = null;

  constructor() {
    // Auto-connect when service is created
    this.connect();
  }

  // Connection Management
  async connect(): Promise<void> {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return;
    }

    this.isConnecting = true;
    
    try {
      const wsUrl = apiClient.getWebSocketUrl();
      const authToken = localStorage.getItem('cerebro_auth_token');
      const walletAddress = localStorage.getItem('cerebro_wallet_address');

      // Add auth params to WebSocket URL
      const url = new URL(wsUrl);
      if (authToken) url.searchParams.set('token', authToken);
      if (walletAddress) url.searchParams.set('wallet', walletAddress);

      console.log('Connecting to WebSocket:', url.toString());
      
      this.ws = new WebSocket(url.toString());
      this.setupEventHandlers();

      // Connection timeout
      this.connectionTimeout = setTimeout(() => {
        if (this.ws && this.ws.readyState === WebSocket.CONNECTING) {
          console.warn('WebSocket connection timeout');
          this.ws.close();
        }
      }, 10000);

    } catch (error) {
      console.error('WebSocket connection error:', error);
      this.isConnecting = false;
      this.scheduleReconnect();
    }
  }

  private setupEventHandlers(): void {
    if (!this.ws) return;

    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.isConnecting = false;
      this.reconnectAttempts = 0;
      this.reconnectDelay = 1000;
      
      if (this.connectionTimeout) {
        clearTimeout(this.connectionTimeout);
        this.connectionTimeout = null;
      }

      this.startHeartbeat();
      this.emit('connection.opened', {});
    };

    this.ws.onmessage = (event) => {
      try {
        const message: WebSocketEvent = JSON.parse(event.data);
        console.log('WebSocket message:', message);
        this.handleMessage(message);
      } catch (error) {
        console.error('Error parsing WebSocket message:', error);
      }
    };

    this.ws.onclose = (event) => {
      console.log('WebSocket closed:', event.code, event.reason);
      this.isConnecting = false;
      this.stopHeartbeat();
      
      if (this.connectionTimeout) {
        clearTimeout(this.connectionTimeout);
        this.connectionTimeout = null;
      }

      this.emit('connection.closed', { code: event.code, reason: event.reason });

      if (this.shouldReconnect && event.code !== 1000) {
        this.scheduleReconnect();
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      this.emit('connection.error', { error });
    };
  }

  private handleMessage(message: WebSocketEvent): void {
    // Handle specific message types
    switch (message.type) {
      case 'wallet.balance_updated':
        this.emit('wallet.balance_updated', message.data);
        break;
      case 'portfolio.updated':
        this.emit('portfolio.updated', message.data);
        break;
      case 'strategy.status_changed':
        this.emit('strategy.status_changed', message.data);
        break;
      case 'trading.execution_completed':
        this.emit('trading.execution_completed', message.data);
        break;
      case 'pong':
        // Heartbeat response
        break;
      default:
        console.warn('Unknown WebSocket message type:', message.type);
    }

    // Emit generic message event
    this.emit('message', message);
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached');
      this.emit('connection.failed', {});
      return;
    }

    this.reconnectAttempts++;
    const delay = Math.min(this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1), this.maxReconnectDelay);
    
    console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
    
    setTimeout(() => {
      if (this.shouldReconnect) {
        this.connect();
      }
    }, delay);
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.send('ping', {});
      }
    }, 30000); // Send ping every 30 seconds
  }

  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }

  // Message Sending
  send(type: string, data: any): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const message = {
        type,
        data,
        timestamp: Date.now(),
      };
      this.ws.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket not connected, cannot send message:', type);
    }
  }

  // Event Subscription
  on<T = any>(eventType: string, listener: EventListener<T>): () => void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set());
    }
    
    this.listeners.get(eventType)!.add(listener);

    // Return unsubscribe function
    return () => {
      const listeners = this.listeners.get(eventType);
      if (listeners) {
        listeners.delete(listener);
        if (listeners.size === 0) {
          this.listeners.delete(eventType);
        }
      }
    };
  }

  // Event Emission
  private emit(eventType: string, data: any): void {
    const listeners = this.listeners.get(eventType);
    if (listeners) {
      listeners.forEach(listener => {
        try {
          listener(data);
        } catch (error) {
          console.error('Error in WebSocket event listener:', error);
        }
      });
    }
  }

  // Connection Status
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  getConnectionState(): string {
    if (!this.ws) return 'disconnected';
    
    switch (this.ws.readyState) {
      case WebSocket.CONNECTING: return 'connecting';
      case WebSocket.OPEN: return 'connected';
      case WebSocket.CLOSING: return 'closing';
      case WebSocket.CLOSED: return 'disconnected';
      default: return 'unknown';
    }
  }

  // Cleanup
  disconnect(): void {
    this.shouldReconnect = false;
    this.stopHeartbeat();
    
    if (this.connectionTimeout) {
      clearTimeout(this.connectionTimeout);
      this.connectionTimeout = null;
    }

    if (this.ws) {
      this.ws.close(1000, 'Manual disconnect');
      this.ws = null;
    }

    this.listeners.clear();
  }

  // Subscription Helpers
  onWalletBalanceUpdate(listener: EventListener<WalletBalanceEvent['data']>): () => void {
    return this.on('wallet.balance_updated', listener);
  }

  onPortfolioUpdate(listener: EventListener<PortfolioUpdateEvent['data']>): () => void {
    return this.on('portfolio.updated', listener);
  }

  onStrategyStatusChange(listener: EventListener<StrategyStatusEvent['data']>): () => void {
    return this.on('strategy.status_changed', listener);
  }

  onTradingExecution(listener: EventListener<TradingExecutionEvent['data']>): () => void {
    return this.on('trading.execution_completed', listener);
  }
}

// Export singleton instance
export const webSocketService = new WebSocketService();

// Cleanup on page unload
window.addEventListener('beforeunload', () => {
  webSocketService.disconnect();
});
