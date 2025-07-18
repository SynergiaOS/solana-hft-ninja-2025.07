import { useQuery } from '@tanstack/react-query';
import { Strategy } from '@/types';
import { apiClient } from '@/services/api';

// Real API function
const fetchStrategies = async (): Promise<Strategy[]> => {
  try {
    const strategiesData = await apiClient.get('/api/strategies');

    // Transform API data to Strategy format
    return strategiesData.map((strategy: any) => ({
      id: strategy.id,
      name: strategy.name,
      type: strategy.type,
      status: strategy.status,
      config: {
        enabled: strategy.status === 'active',
        maxPositionSize: strategy.config.max_position_size || 1000,
        minProfitThreshold: strategy.config.min_profit_bps / 100 || 0.5,
        maxSlippage: strategy.config.max_slippage_bps / 10000 || 0.1,
        gasPrice: 50, // Default
        timeoutMs: 5000, // Default
      },
      metrics: {
        totalTrades: strategy.metrics.totalTrades,
        successfulTrades: Math.round(strategy.metrics.totalTrades * strategy.metrics.successRate / 100),
        successRate: strategy.metrics.successRate,
        totalProfit: strategy.metrics.totalProfit * 150, // Convert SOL to USD
        avgLatency: strategy.metrics.avgLatency,
      },
      lastUpdate: strategy.updatedAt,
    }));
  } catch (error) {
    console.error('Failed to fetch strategies:', error);
    // Return empty array on error
    return [];
  }
};

export const useStrategies = () => {
  return useQuery({
    queryKey: ['strategies'],
    queryFn: fetchStrategies,
    refetchInterval: 60000, // Refetch every minute
    staleTime: 30000, // Consider data stale after 30 seconds
  });
};
