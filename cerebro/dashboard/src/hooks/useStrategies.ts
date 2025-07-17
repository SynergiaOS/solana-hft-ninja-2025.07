import { useQuery } from '@tanstack/react-query';
import { Strategy } from '@/types';

// Mock API function - replace with real API call
const fetchStrategies = async (): Promise<Strategy[]> => {
  // Simulate API delay
  await new Promise(resolve => setTimeout(resolve, 800));
  
  return [
    {
      id: '1',
      name: 'Sandwich Strategy',
      type: 'sandwich',
      status: 'active',
      config: {
        enabled: true,
        maxPositionSize: 1000,
        minProfitThreshold: 0.5,
        maxSlippage: 0.1,
        gasPrice: 50,
        timeoutMs: 5000,
      },
      metrics: {
        totalTrades: 156,
        successfulTrades: 136,
        successRate: 87.2,
        totalProfit: 2345.67,
        avgLatency: 89,
      },
      lastUpdate: new Date().toISOString(),
    },
    {
      id: '2',
      name: 'Arbitrage Bot',
      type: 'arbitrage',
      status: 'active',
      config: {
        enabled: true,
        maxPositionSize: 2000,
        minProfitThreshold: 0.3,
        maxSlippage: 0.05,
        gasPrice: 45,
        timeoutMs: 3000,
      },
      metrics: {
        totalTrades: 89,
        successfulTrades: 81,
        successRate: 91.0,
        totalProfit: 1892.34,
        avgLatency: 67,
      },
      lastUpdate: new Date().toISOString(),
    },
  ];
};

export const useStrategies = () => {
  return useQuery({
    queryKey: ['strategies'],
    queryFn: fetchStrategies,
    refetchInterval: 60000, // Refetch every minute
    staleTime: 30000, // Consider data stale after 30 seconds
  });
};
