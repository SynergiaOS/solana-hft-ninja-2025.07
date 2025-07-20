"use client";

import { useEffect, useState } from "react";
import useWebSocket from "react-use-websocket";

interface TickerData {
  price: number;
  pnl: number;
  volume: number;
  change24h: number;
  timestamp: number;
}

export default function LiveTicker() {
  const [data, setData] = useState<TickerData | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  // WebSocket connection to Cerberus API
  const { lastMessage, readyState } = useWebSocket(
    process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:8080/ws/positions",
    {
      shouldReconnect: () => true,
      reconnectAttempts: 10,
      reconnectInterval: 3000,
    }
  );

  useEffect(() => {
    setIsConnected(readyState === 1); // 1 = OPEN
  }, [readyState]);

  useEffect(() => {
    if (lastMessage) {
      try {
        const parsed = JSON.parse(lastMessage.data);
        setData(parsed);
      } catch (error) {
        console.error("Failed to parse WebSocket message:", error);
      }
    }
  }, [lastMessage]);

  // Mock data for development
  useEffect(() => {
    if (!isConnected) {
      const interval = setInterval(() => {
        setData({
          price: 23.45 + (Math.random() - 0.5) * 2,
          pnl: (Math.random() - 0.5) * 10,
          volume: 1250000 + Math.random() * 500000,
          change24h: (Math.random() - 0.5) * 20,
          timestamp: Date.now(),
        });
      }, 1000);

      return () => clearInterval(interval);
    }
  }, [isConnected]);

  if (!data) {
    return (
      <div className="flex items-center gap-4 p-4 animate-pulse">
        <div className="h-8 w-20 bg-gray-300 dark:bg-gray-700 rounded"></div>
        <div className="h-6 w-16 bg-gray-300 dark:bg-gray-700 rounded"></div>
      </div>
    );
  }

  return (
    <div className="flex items-center justify-between p-4 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800">
      {/* Connection Status */}
      <div className="flex items-center gap-2">
        <div
          className={`w-2 h-2 rounded-full ${
            isConnected ? "bg-green-500" : "bg-red-500"
          }`}
        />
        <span className="text-xs text-gray-500 dark:text-gray-400">
          {isConnected ? "Live" : "Mock"}
        </span>
      </div>

      {/* Price Data */}
      <div className="flex items-center gap-6">
        {/* Current Price */}
        <div className="text-center">
          <div className="text-xs text-gray-500 dark:text-gray-400">SOL/USD</div>
          <div className="font-mono text-xl font-bold text-gray-900 dark:text-white">
            ${data.price.toFixed(2)}
          </div>
        </div>

        {/* 24h Change */}
        <div className="text-center">
          <div className="text-xs text-gray-500 dark:text-gray-400">24h</div>
          <div
            className={`font-mono text-sm font-semibold ${
              data.change24h >= 0
                ? "text-green-500 dark:text-green-400"
                : "text-red-500 dark:text-red-400"
            }`}
          >
            {data.change24h >= 0 ? "+" : ""}
            {data.change24h.toFixed(2)}%
          </div>
        </div>

        {/* P&L */}
        <div className="text-center">
          <div className="text-xs text-gray-500 dark:text-gray-400">P&L</div>
          <div
            className={`font-mono text-lg font-bold ${
              data.pnl >= 0
                ? "text-green-500 dark:text-green-400"
                : "text-red-500 dark:text-red-400"
            }`}
          >
            {data.pnl >= 0 ? "+" : ""}
            {data.pnl.toFixed(2)}%
          </div>
        </div>

        {/* Volume */}
        <div className="text-center hidden md:block">
          <div className="text-xs text-gray-500 dark:text-gray-400">Volume</div>
          <div className="font-mono text-sm text-gray-900 dark:text-white">
            ${(data.volume / 1000000).toFixed(1)}M
          </div>
        </div>
      </div>

      {/* Last Update */}
      <div className="text-xs text-gray-500 dark:text-gray-400">
        {new Date(data.timestamp).toLocaleTimeString()}
      </div>
    </div>
  );
}
