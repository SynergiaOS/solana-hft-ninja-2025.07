import { useQuery } from '@tanstack/react-query';
import { TradingMetrics } from '@/types';

// Mock API function - replace with real API call
const fetchTradingMetrics = async (): Promise<TradingMetrics> => {
  // Simulate API delay
  await new Promise(resolve => setTimeout(resolve, 1000));
  
  return {
    totalPnL: 47892.34,
    totalTrades: 1247,
    winRate: 87.3,
    avgProfit: 234.56,
    avgLoss: -89.23,
    sharpeRatio: 2.34,
    maxDrawdown: -5.67,
    currentBalance: 47892.34,
    dailyPnL: 1247.89,
    weeklyPnL: 3456.78,
    monthlyPnL: 12345.67,
  };
};

export const useTradingMetrics = () => {
  return useQuery({
    queryKey: ['trading-metrics'],
    queryFn: fetchTradingMetrics,
    refetchInterval: 30000, // Refetch every 30 seconds
    staleTime: 10000, // Consider data stale after 10 seconds
  });
};
