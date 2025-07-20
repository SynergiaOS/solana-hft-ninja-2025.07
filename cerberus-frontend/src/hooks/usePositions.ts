"use client";

import { useEffect, useState, useCallback } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";

export interface Position {
  mint: string;
  symbol?: string;
  entryPrice: number;
  currentPrice: number;
  positionSizeSol: number;
  pnl: number;
  pnlPercent: number;
  status: "open" | "closed" | "pending" | "failed";
  strategy: string;
  ageSeconds: number;
  takeProfitTarget: number;
  stopLossTarget: number;
  timeoutSeconds: number;
  walletAddress: string;
  createdAt: string;
  updatedAt: string;
}

export interface PositionsData {
  positions: Position[];
  totalValue: number;
  totalPnl: number;
  totalPnlPercent: number;
  activeCount: number;
  profitableCount: number;
}

export interface UsePositionsReturn {
  data: PositionsData | null;
  positions: Position[];
  isConnected: boolean;
  readyState: ReadyState;
  error: string | null;
  sendMessage: (message: string | ArrayBufferLike | Blob | ArrayBufferView) => void;
  refetch: () => Promise<void>;
}

const WEBSOCKET_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:8080/ws/positions";
const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080/api";

export function usePositions(): UsePositionsReturn {
  const [data, setData] = useState<PositionsData | null>(null);
  const [error, setError] = useState<string | null>(null);

  // WebSocket connection
  const { lastMessage, readyState, sendMessage } = useWebSocket(
    WEBSOCKET_URL,
    {
      shouldReconnect: () => true,
      reconnectAttempts: 10,
      reconnectInterval: (attemptNumber) => Math.min(Math.pow(2, attemptNumber) * 1000, 30000),
      onOpen: () => {
        console.log("WebSocket connected");
        setError(null);
      },
      onClose: () => {
        console.log("WebSocket disconnected");
      },
      onError: (event) => {
        console.error("WebSocket error:", event);
        setError("WebSocket connection failed");
      },
    }
  );

  // Process WebSocket messages
  useEffect(() => {
    if (lastMessage) {
      try {
        const message = JSON.parse(lastMessage.data);
        
        switch (message.type) {
          case "positions_update":
            setData(message.data);
            break;
          case "position_update":
            // Update single position
            setData(prev => {
              if (!prev) return prev;
              const updatedPositions = prev.positions.map(pos =>
                pos.mint === message.data.mint ? { ...pos, ...message.data } : pos
              );
              return calculatePositionsData(updatedPositions);
            });
            break;
          case "position_closed":
            // Remove closed position
            setData(prev => {
              if (!prev) return prev;
              const filteredPositions = prev.positions.filter(pos => pos.mint !== message.data.mint);
              return calculatePositionsData(filteredPositions);
            });
            break;
          case "error":
            setError(message.message);
            break;
          default:
            console.warn("Unknown message type:", message.type);
        }
      } catch (err) {
        console.error("Failed to parse WebSocket message:", err);
        setError("Failed to parse server message");
      }
    }
  }, [lastMessage]);

  // Fetch initial data via REST API
  const refetch = useCallback(async () => {
    try {
      setError(null);
      const response = await fetch(`${API_URL}/cerberus/positions`);
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const positions: Position[] = await response.json();
      setData(calculatePositionsData(positions));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to fetch positions";
      setError(errorMessage);
      console.error("Failed to fetch positions:", err);
      
      // Use mock data in development
      if (process.env.NODE_ENV === "development") {
        setData(getMockPositionsData());
      }
    }
  }, []);

  // Initial data fetch
  useEffect(() => {
    refetch();
  }, [refetch]);

  // Mock data for development
  useEffect(() => {
    if (readyState !== ReadyState.OPEN && process.env.NODE_ENV === "development") {
      const interval = setInterval(() => {
        setData(getMockPositionsData());
      }, 2000);

      return () => clearInterval(interval);
    }
  }, [readyState]);

  return {
    data,
    positions: data?.positions || [],
    isConnected: readyState === ReadyState.OPEN,
    readyState,
    error,
    sendMessage,
    refetch,
  };
}

// Helper function to calculate aggregated data
function calculatePositionsData(positions: Position[]): PositionsData {
  const totalValue = positions.reduce((sum, pos) => sum + pos.positionSizeSol, 0);
  const totalPnl = positions.reduce((sum, pos) => sum + pos.pnl, 0);
  const totalPnlPercent = totalValue > 0 ? (totalPnl / totalValue) * 100 : 0;
  const activeCount = positions.filter(pos => pos.status === "open").length;
  const profitableCount = positions.filter(pos => pos.pnl > 0).length;

  return {
    positions,
    totalValue,
    totalPnl,
    totalPnlPercent,
    activeCount,
    profitableCount,
  };
}

// Mock data for development
function getMockPositionsData(): PositionsData {
  const mockPositions: Position[] = [
    {
      mint: "So11111111111111111111111111111111111111112",
      symbol: "SOL",
      entryPrice: 23.45,
      currentPrice: 23.67,
      positionSizeSol: 0.1,
      pnl: 0.0094,
      pnlPercent: 0.94,
      status: "open",
      strategy: "sandwich_strategy",
      ageSeconds: 120 + Math.floor(Math.random() * 300),
      takeProfitTarget: 100.0,
      stopLossTarget: -25.0,
      timeoutSeconds: 600,
      walletAddress: "DSJXCqXuRckDhSX34oiFgEQChuezxvVgkEAyaA2MML8X",
      createdAt: new Date(Date.now() - 120000).toISOString(),
      updatedAt: new Date().toISOString(),
    },
    {
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      symbol: "USDC",
      entryPrice: 1.0,
      currentPrice: 0.998,
      positionSizeSol: 0.05,
      pnl: -0.0001,
      pnlPercent: -0.2,
      status: "open",
      strategy: "arbitrage_strategy",
      ageSeconds: 45 + Math.floor(Math.random() * 200),
      takeProfitTarget: 20.0,
      stopLossTarget: -10.0,
      timeoutSeconds: 300,
      walletAddress: "DSJXCqXuRckDhSX34oiFgEQChuezxvVgkEAyaA2MML8X",
      createdAt: new Date(Date.now() - 45000).toISOString(),
      updatedAt: new Date().toISOString(),
    },
    {
      mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
      symbol: "USDT",
      entryPrice: 1.001,
      currentPrice: 1.003,
      positionSizeSol: 0.08,
      pnl: 0.00016,
      pnlPercent: 0.2,
      status: "open",
      strategy: "market_making",
      ageSeconds: 300 + Math.floor(Math.random() * 400),
      takeProfitTarget: 50.0,
      stopLossTarget: -15.0,
      timeoutSeconds: 900,
      walletAddress: "DSJXCqXuRckDhSX34oiFgEQChuezxvVgkEAyaA2MML8X",
      createdAt: new Date(Date.now() - 300000).toISOString(),
      updatedAt: new Date().toISOString(),
    },
  ];

  // Add some randomness to mock data
  mockPositions.forEach(pos => {
    pos.currentPrice += (Math.random() - 0.5) * 0.01;
    pos.pnl = (pos.currentPrice - pos.entryPrice) * pos.positionSizeSol;
    pos.pnlPercent = ((pos.currentPrice - pos.entryPrice) / pos.entryPrice) * 100;
    pos.ageSeconds += 1;
  });

  return calculatePositionsData(mockPositions);
}
