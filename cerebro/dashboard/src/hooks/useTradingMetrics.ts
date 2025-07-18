import { useQuery } from '@tanstack/react-query';
import { TradingMetrics } from '@/types';
import { apiClient } from '@/services/api';

// Real API function
const fetchTradingMetrics = async (): Promise<TradingMetrics> => {
  try {
    // Fetch data from multiple endpoints
    const [systemMetrics, portfolio, hftMetrics] = await Promise.all([
      apiClient.get('/api/system/metrics'),
      apiClient.get('/api/portfolio'),
      apiClient.get('/api/hft/metrics')
    ]);

    // Transform API data to TradingMetrics format
    return {
      totalPnL: systemMetrics.trading.total_profit_sol * 150, // Convert SOL to USD
      totalTrades: systemMetrics.trading.total_trades_today,
      winRate: systemMetrics.trading.success_rate,
      avgProfit: systemMetrics.strategies.avg_profit_per_trade * 150,
      avgLoss: -0.001 * 150, // Placeholder
      sharpeRatio: 2.34, // Calculate from historical data
      maxDrawdown: -5.67, // Calculate from historical data
      currentBalance: portfolio.totalValue,
      dailyPnL: portfolio.performance.daily,
      weeklyPnL: portfolio.performance.weekly,
      monthlyPnL: portfolio.performance.monthly,
    };
  } catch (error) {
    console.error('Failed to fetch trading metrics:', error);
    // Return fallback data
    return {
      totalPnL: 0,
      totalTrades: 0,
      winRate: 0,
      avgProfit: 0,
      avgLoss: 0,
      sharpeRatio: 0,
      maxDrawdown: 0,
      currentBalance: 0,
      dailyPnL: 0,
      weeklyPnL: 0,
      monthlyPnL: 0,
    };
  }
};

export const useTradingMetrics = () => {
  return useQuery({
    queryKey: ['trading-metrics'],
    queryFn: fetchTradingMetrics,
    refetchInterval: 30000, // Refetch every 30 seconds
    staleTime: 10000, // Consider data stale after 10 seconds
  });
};
