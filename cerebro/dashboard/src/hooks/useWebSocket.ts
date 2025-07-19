import { useState, useEffect, useRef, useCallback } from 'react';

interface WebSocketMessage {
  type: string;
  data: any;
  timestamp: string;
}

interface UseWebSocketOptions {
  url: string;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  onMessage?: (message: WebSocketMessage) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (error: Event) => void;
}

interface UseWebSocketReturn {
  isConnected: boolean;
  lastMessage: WebSocketMessage | null;
  sendMessage: (message: any) => void;
  connect: () => void;
  disconnect: () => void;
  reconnectAttempts: number;
}

export const useWebSocket = (options: UseWebSocketOptions): UseWebSocketReturn => {
  const {
    url,
    reconnectInterval = 3000,
    maxReconnectAttempts = 5,
    onMessage,
    onConnect,
    onDisconnect,
    onError
  } = options;

  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);
  const [reconnectAttempts, setReconnectAttempts] = useState(0);

  const ws = useRef<WebSocket | null>(null);
  const reconnectTimeoutId = useRef<NodeJS.Timeout | null>(null);
  const shouldReconnect = useRef(true);

  const connect = useCallback(() => {
    try {
      ws.current = new WebSocket(url);

      ws.current.onopen = () => {
        console.log('WebSocket connected');
        setIsConnected(true);
        setReconnectAttempts(0);
        onConnect?.();
      };

      ws.current.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          setLastMessage(message);
          onMessage?.(message);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      ws.current.onclose = () => {
        console.log('WebSocket disconnected');
        setIsConnected(false);
        onDisconnect?.();

        // Attempt to reconnect if enabled and under max attempts
        if (shouldReconnect.current && reconnectAttempts < maxReconnectAttempts) {
          reconnectTimeoutId.current = setTimeout(() => {
            setReconnectAttempts(prev => prev + 1);
            connect();
          }, reconnectInterval);
        }
      };

      ws.current.onerror = (error) => {
        console.error('WebSocket error:', error);
        onError?.(error);
      };

    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
    }
  }, [url, reconnectInterval, maxReconnectAttempts, reconnectAttempts, onMessage, onConnect, onDisconnect, onError]);

  const disconnect = useCallback(() => {
    shouldReconnect.current = false;
    
    if (reconnectTimeoutId.current) {
      clearTimeout(reconnectTimeoutId.current);
      reconnectTimeoutId.current = null;
    }

    if (ws.current) {
      ws.current.close();
      ws.current = null;
    }
  }, []);

  const sendMessage = useCallback((message: any) => {
    if (ws.current && ws.current.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket is not connected. Cannot send message.');
    }
  }, []);

  useEffect(() => {
    connect();

    return () => {
      shouldReconnect.current = false;
      disconnect();
    };
  }, [connect, disconnect]);

  return {
    isConnected,
    lastMessage,
    sendMessage,
    connect,
    disconnect,
    reconnectAttempts
  };
};

// Mock WebSocket for demo purposes
export const useMockWebSocket = (): UseWebSocketReturn => {
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);
  const [reconnectAttempts] = useState(0);

  // Mock data generator
  useEffect(() => {
    setIsConnected(true);
    
    const interval = setInterval(() => {
      const mockMessages = [
        {
          type: 'new_trade',
          data: {
            id: `tx_${Date.now()}`,
            type: ['sandwich', 'arbitrage', 'liquidation'][Math.floor(Math.random() * 3)],
            token_pair: ['SOL/USDC', 'BONK/SOL', 'RAY/USDC'][Math.floor(Math.random() * 3)],
            profit_sol: Math.random() * 0.5,
            profit_usd: Math.random() * 50,
            execution_time_ms: Math.floor(Math.random() * 200) + 50,
            timestamp: new Date().toISOString(),
            status: 'completed',
            confidence: 0.8 + Math.random() * 0.2
          },
          timestamp: new Date().toISOString()
        },
        {
          type: 'opportunity_detected',
          data: {
            type: 'arbitrage',
            token_pair: 'BONK/SOL',
            potential_profit: Math.random() * 0.1,
            confidence: 0.7 + Math.random() * 0.3,
            dex_from: 'Orca',
            dex_to: 'Raydium'
          },
          timestamp: new Date().toISOString()
        },
        {
          type: 'system_metrics',
          data: {
            transactions_processed: Math.floor(Math.random() * 100) + 1200,
            opportunities_detected: Math.floor(Math.random() * 20) + 150,
            avg_latency_ms: Math.floor(Math.random() * 50) + 80,
            memory_usage_mb: Math.floor(Math.random() * 100) + 200
          },
          timestamp: new Date().toISOString()
        }
      ];

      const randomMessage = mockMessages[Math.floor(Math.random() * mockMessages.length)];
      setLastMessage(randomMessage);
    }, 3000 + Math.random() * 2000); // Random interval between 3-5 seconds

    return () => clearInterval(interval);
  }, []);

  const sendMessage = useCallback(() => {
    console.log('Mock WebSocket: Message sent');
  }, []);

  const connect = useCallback(() => {
    setIsConnected(true);
  }, []);

  const disconnect = useCallback(() => {
    setIsConnected(false);
  }, []);

  return {
    isConnected,
    lastMessage,
    sendMessage,
    connect,
    disconnect,
    reconnectAttempts
  };
};
