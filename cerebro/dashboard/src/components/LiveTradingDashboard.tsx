import React, { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { motion, AnimatePresence } from 'framer-motion';
import { apiClient } from '@/services/api';
import { useMockWebSocket } from '@/hooks/useWebSocket';
import toast from 'react-hot-toast';
import TradingCharts from '@/components/TradingCharts';
import {
  TrendingUp,
  TrendingDown,
  Zap,
  Target,
  DollarSign,
  Clock,
  Activity,
  CheckCircle,
  AlertTriangle,
  BarChart3
} from 'lucide-react';

interface Trade {
  id: string;
  type: string;
  token_pair: string;
  profit_sol: number;
  profit_usd: number;
  execution_time_ms: number;
  timestamp: string;
  status: string;
  strategy: string;
  confidence: number;
  gas_fees: number;
  dex?: string;
  dex_from?: string;
  dex_to?: string;
  liquidation_bonus?: number;
  protocol?: string;
  entry_price?: number;
  exit_price?: number;
  tokens_bought?: number;
  route_hops?: number;
  impact?: number;
  price_diff?: number;
  slippage?: number;
}

interface TradingSummary {
  total_trades: number;
  successful_trades: number;
  total_profit_sol: number;
  total_profit_usd: number;
  success_rate: number;
  avg_execution_time_ms: number;
  total_gas_fees: number;
  net_profit_sol: number;
  roi_percentage: number;
}

interface Strategy {
  name: string;
  active: boolean;
  trades_today: number;
  success_rate: number;
  profit_sol: number;
  avg_execution_ms: number;
  risk_level: string;
  last_trade: string;
}

const LiveTradingDashboard: React.FC = () => {
  const [selectedTrade, setSelectedTrade] = useState<Trade | null>(null);
  const [liveEvents, setLiveEvents] = useState<any[]>([]);

  // WebSocket connection for live updates
  const { isConnected, lastMessage } = useMockWebSocket();

  // Fetch trading history with auto-refresh
  const { data: tradingData, isLoading: tradesLoading } = useQuery({
    queryKey: ['trading-history'],
    queryFn: () => apiClient.get('/api/trading/history'),
    refetchInterval: 5000, // Refresh every 5 seconds
  });

  // Fetch strategies
  const { data: strategiesData, isLoading: strategiesLoading } = useQuery({
    queryKey: ['strategies'],
    queryFn: () => apiClient.get('/api/strategies'),
    refetchInterval: 10000, // Refresh every 10 seconds
  });

  // Fetch system metrics
  const { data: metricsData } = useQuery({
    queryKey: ['system-metrics'],
    queryFn: () => apiClient.get('/api/system/metrics'),
    refetchInterval: 3000, // Refresh every 3 seconds
  });

  const trades: Trade[] = tradingData?.trades || [];
  const summary: TradingSummary = tradingData?.summary || {};
  const strategies: Strategy[] = strategiesData?.strategies || [];

  // Handle live WebSocket messages
  useEffect(() => {
    if (lastMessage) {
      setLiveEvents(prev => [lastMessage, ...prev.slice(0, 9)]); // Keep last 10 events

      // Show toast notifications for important events
      if (lastMessage.type === 'new_trade') {
        const trade = lastMessage.data;
        toast.success(
          `🎉 New ${trade.type} trade! +${trade.profit_sol.toFixed(4)} SOL`,
          {
            duration: 4000,
            style: {
              background: '#1A1D29',
              color: '#fff',
              border: '1px solid #10B981'
            }
          }
        );
      } else if (lastMessage.type === 'opportunity_detected') {
        const opp = lastMessage.data;
        toast(
          `👀 ${opp.type} opportunity detected: ${opp.token_pair}`,
          {
            duration: 3000,
            icon: '⚡',
            style: {
              background: '#1A1D29',
              color: '#fff',
              border: '1px solid #F59E0B'
            }
          }
        );
      }
    }
  }, [lastMessage]);

  const getTradeIcon = (type: string) => {
    switch (type) {
      case 'sandwich': return '🥪';
      case 'arbitrage': return '⚡';
      case 'liquidation': return '💧';
      case 'token_snipe': return '🎯';
      case 'jupiter_arbitrage': return '🪐';
      default: return '💰';
    }
  };

  const getTradeColor = (type: string) => {
    switch (type) {
      case 'sandwich': return 'from-orange-500 to-red-500';
      case 'arbitrage': return 'from-blue-500 to-cyan-500';
      case 'liquidation': return 'from-purple-500 to-pink-500';
      case 'token_snipe': return 'from-green-500 to-emerald-500';
      case 'jupiter_arbitrage': return 'from-indigo-500 to-purple-500';
      default: return 'from-gray-500 to-gray-600';
    }
  };

  const getRiskColor = (level: string) => {
    switch (level) {
      case 'low': return 'text-green-400 bg-green-900/20';
      case 'medium': return 'text-yellow-400 bg-yellow-900/20';
      case 'high': return 'text-red-400 bg-red-900/20';
      default: return 'text-gray-400 bg-gray-900/20';
    }
  };

  const formatTime = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString();
  };

  const formatCurrency = (amount: number, currency: string = 'USD') => {
    if (currency === 'SOL') {
      return `${amount.toFixed(4)} SOL`;
    }
    return `$${amount.toFixed(2)}`;
  };

  if (tradesLoading || strategiesLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white flex items-center gap-3">
            <Activity className="h-8 w-8 text-green-400" />
            Live Trading Dashboard
          </h1>
          <p className="text-gray-400 mt-2">
            Real-time MEV trading performance and analytics
          </p>
        </div>
        
        <div className="flex gap-2">
          <div className={`border rounded-lg px-3 py-2 ${isConnected ? 'bg-green-900/30 border-green-700/50' : 'bg-red-900/30 border-red-700/50'}`}>
            <div className={`flex items-center gap-2 ${isConnected ? 'text-green-300' : 'text-red-300'}`}>
              <div className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-400 animate-pulse' : 'bg-red-400'}`}></div>
              <span className="text-sm font-medium">{isConnected ? 'Live' : 'Offline'}</span>
            </div>
          </div>
          <div className="bg-blue-900/30 border border-blue-700/50 rounded-lg px-3 py-2">
            <div className="flex items-center gap-2 text-blue-300">
              <span className="text-sm font-medium">Events: {liveEvents.length}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-gradient-to-br from-green-900/20 to-green-800/10 border border-green-700/30 rounded-xl p-6"
        >
          <div className="flex items-center justify-between">
            <div>
              <p className="text-green-300 text-sm font-medium">Total Profit</p>
              <p className="text-2xl font-bold text-white">
                {formatCurrency(summary.total_profit_sol, 'SOL')}
              </p>
              <p className="text-green-400 text-sm">
                {formatCurrency(summary.total_profit_usd)}
              </p>
            </div>
            <DollarSign className="h-8 w-8 text-green-400" />
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="bg-gradient-to-br from-blue-900/20 to-blue-800/10 border border-blue-700/30 rounded-xl p-6"
        >
          <div className="flex items-center justify-between">
            <div>
              <p className="text-blue-300 text-sm font-medium">Success Rate</p>
              <p className="text-2xl font-bold text-white">
                {summary.success_rate?.toFixed(1)}%
              </p>
              <p className="text-blue-400 text-sm">
                {summary.successful_trades}/{summary.total_trades} trades
              </p>
            </div>
            <Target className="h-8 w-8 text-blue-400" />
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="bg-gradient-to-br from-purple-900/20 to-purple-800/10 border border-purple-700/30 rounded-xl p-6"
        >
          <div className="flex items-center justify-between">
            <div>
              <p className="text-purple-300 text-sm font-medium">Avg Execution</p>
              <p className="text-2xl font-bold text-white">
                {summary.avg_execution_time_ms?.toFixed(0)}ms
              </p>
              <p className="text-purple-400 text-sm">
                Sub-100ms target
              </p>
            </div>
            <Zap className="h-8 w-8 text-purple-400" />
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="bg-gradient-to-br from-orange-900/20 to-orange-800/10 border border-orange-700/30 rounded-xl p-6"
        >
          <div className="flex items-center justify-between">
            <div>
              <p className="text-orange-300 text-sm font-medium">ROI</p>
              <p className="text-2xl font-bold text-white">
                {summary.roi_percentage?.toFixed(1)}%
              </p>
              <p className="text-orange-400 text-sm">
                Daily return
              </p>
            </div>
            <TrendingUp className="h-8 w-8 text-orange-400" />
          </div>
        </motion.div>
      </div>

      {/* Recent Trades & Live Events */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-xl font-semibold text-white">Recent Successful Trades</h3>
            <div className="flex items-center gap-2 text-green-400">
              <CheckCircle className="h-4 w-4" />
              <span className="text-sm">All Profitable</span>
            </div>
          </div>
          
          <div className="space-y-4 max-h-96 overflow-y-auto">
            <AnimatePresence>
              {trades.map((trade, index) => (
                <motion.div
                  key={trade.id}
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: index * 0.1 }}
                  className="bg-gray-800/50 rounded-lg p-4 cursor-pointer hover:bg-gray-800/70 transition-colors"
                  onClick={() => setSelectedTrade(trade)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className={`w-10 h-10 rounded-lg bg-gradient-to-r ${getTradeColor(trade.type)} flex items-center justify-center text-lg`}>
                        {getTradeIcon(trade.type)}
                      </div>
                      <div>
                        <p className="font-medium text-white">{trade.token_pair}</p>
                        <p className="text-sm text-gray-400">{trade.strategy}</p>
                      </div>
                    </div>
                    <div className="text-right">
                      <p className="font-medium text-green-400">
                        +{formatCurrency(trade.profit_sol, 'SOL')}
                      </p>
                      <p className="text-sm text-gray-400">
                        {trade.execution_time_ms}ms
                      </p>
                    </div>
                  </div>
                </motion.div>
              ))}
            </AnimatePresence>
          </div>
        </div>

        {/* Strategy Performance */}
        <div className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-xl font-semibold text-white">Strategy Performance</h3>
            <BarChart3 className="h-5 w-5 text-gray-400" />
          </div>
          
          <div className="space-y-4">
            {strategies.map((strategy, index) => (
              <motion.div
                key={strategy.name}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: index * 0.1 }}
                className="bg-gray-800/50 rounded-lg p-4"
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-3">
                    <div className={`w-2 h-2 rounded-full ${strategy.active ? 'bg-green-400' : 'bg-gray-500'}`}></div>
                    <span className="font-medium text-white">{strategy.name}</span>
                  </div>
                  <span className={`px-2 py-1 rounded text-xs font-medium ${getRiskColor(strategy.risk_level)}`}>
                    {strategy.risk_level}
                  </span>
                </div>
                
                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div>
                    <p className="text-gray-400">Success Rate</p>
                    <p className="text-white font-medium">{strategy.success_rate.toFixed(1)}%</p>
                  </div>
                  <div>
                    <p className="text-gray-400">Profit</p>
                    <p className="text-green-400 font-medium">{strategy.profit_sol.toFixed(3)} SOL</p>
                  </div>
                  <div>
                    <p className="text-gray-400">Avg Time</p>
                    <p className="text-white font-medium">{strategy.avg_execution_ms}ms</p>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        </div>

        {/* Live Events Panel */}
        <div className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-xl font-semibold text-white">Live Events</h3>
            <div className="flex items-center gap-2">
              <div className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-400 animate-pulse' : 'bg-red-400'}`}></div>
              <span className="text-sm text-gray-400">Real-time</span>
            </div>
          </div>

          <div className="space-y-3 max-h-96 overflow-y-auto">
            <AnimatePresence>
              {liveEvents.map((event, index) => (
                <motion.div
                  key={`${event.timestamp}-${index}`}
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                  className="bg-gray-800/50 rounded-lg p-3 border-l-4 border-purple-500"
                >
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm font-medium text-white">
                      {event.type.replace('_', ' ').toUpperCase()}
                    </span>
                    <span className="text-xs text-gray-400">
                      {new Date(event.timestamp).toLocaleTimeString()}
                    </span>
                  </div>

                  {event.type === 'new_trade' && (
                    <div className="text-sm text-gray-300">
                      <p>{event.data.token_pair} • {event.data.type}</p>
                      <p className="text-green-400">+{event.data.profit_sol.toFixed(4)} SOL</p>
                    </div>
                  )}

                  {event.type === 'opportunity_detected' && (
                    <div className="text-sm text-gray-300">
                      <p>{event.data.token_pair} • {event.data.type}</p>
                      <p className="text-yellow-400">Potential: {event.data.potential_profit.toFixed(4)} SOL</p>
                    </div>
                  )}

                  {event.type === 'system_metrics' && (
                    <div className="text-sm text-gray-300">
                      <p>Processed: {event.data.transactions_processed}</p>
                      <p>Latency: {event.data.avg_latency_ms}ms</p>
                    </div>
                  )}
                </motion.div>
              ))}
            </AnimatePresence>

            {liveEvents.length === 0 && (
              <div className="text-center py-8 text-gray-400">
                <Activity className="h-8 w-8 mx-auto mb-2 opacity-50" />
                <p>Waiting for live events...</p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Trading Analytics Charts */}
      <div className="mt-8">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-white">Trading Analytics</h2>
          <div className="text-sm text-gray-400">
            Real-time performance metrics and insights
          </div>
        </div>
        <TradingCharts />
      </div>

      {/* Trade Details Modal */}
      <AnimatePresence>
        {selectedTrade && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
            onClick={() => setSelectedTrade(null)}
          >
            <motion.div
              initial={{ scale: 0.9, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.9, opacity: 0 }}
              className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6 max-w-md w-full"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-xl font-semibold text-white">Trade Details</h3>
                <button
                  onClick={() => setSelectedTrade(null)}
                  className="text-gray-400 hover:text-white"
                >
                  ✕
                </button>
              </div>
              
              <div className="space-y-4">
                <div className="flex items-center gap-3">
                  <div className={`w-12 h-12 rounded-lg bg-gradient-to-r ${getTradeColor(selectedTrade.type)} flex items-center justify-center text-xl`}>
                    {getTradeIcon(selectedTrade.type)}
                  </div>
                  <div>
                    <p className="font-medium text-white">{selectedTrade.token_pair}</p>
                    <p className="text-sm text-gray-400">{selectedTrade.strategy}</p>
                  </div>
                </div>
                
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <p className="text-gray-400">Profit</p>
                    <p className="text-green-400 font-medium">
                      {formatCurrency(selectedTrade.profit_sol, 'SOL')} ({formatCurrency(selectedTrade.profit_usd)})
                    </p>
                  </div>
                  <div>
                    <p className="text-gray-400">Execution Time</p>
                    <p className="text-white font-medium">{selectedTrade.execution_time_ms}ms</p>
                  </div>
                  <div>
                    <p className="text-gray-400">Confidence</p>
                    <p className="text-white font-medium">{(selectedTrade.confidence * 100).toFixed(1)}%</p>
                  </div>
                  <div>
                    <p className="text-gray-400">Gas Fees</p>
                    <p className="text-white font-medium">{selectedTrade.gas_fees.toFixed(4)} SOL</p>
                  </div>
                  {selectedTrade.dex && (
                    <div>
                      <p className="text-gray-400">DEX</p>
                      <p className="text-white font-medium">{selectedTrade.dex}</p>
                    </div>
                  )}
                  {selectedTrade.dex_from && selectedTrade.dex_to && (
                    <div>
                      <p className="text-gray-400">Route</p>
                      <p className="text-white font-medium">{selectedTrade.dex_from} → {selectedTrade.dex_to}</p>
                    </div>
                  )}
                </div>
                
                <div className="pt-4 border-t border-gray-700">
                  <p className="text-xs text-gray-400">
                    Executed at {formatTime(selectedTrade.timestamp)}
                  </p>
                </div>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default LiveTradingDashboard;
