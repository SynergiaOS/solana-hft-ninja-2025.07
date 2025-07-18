import React, { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { 
  BoltIcon,
  ChartBarIcon,
  ExclamationTriangleIcon,
  WalletIcon,
  SparklesIcon,
  ClockIcon,
  FunnelIcon,
  ArrowPathIcon
} from '@heroicons/react/24/outline';

// Components
import Card from '@/components/ui/Card';
import LoadingSpinner from '@/components/ui/LoadingSpinner';
import Badge from '@/components/ui/Badge';

// Services
import { api } from '@/services/api';

// Types
import type { OpportunityEvent, ExecutionEvent, RiskEvent, WalletEvent } from '@/types';

const WebhookEventsPage: React.FC = () => {
  const [selectedEventTypes, setSelectedEventTypes] = useState<string[]>([]);
  const [autoRefresh, setAutoRefresh] = useState(true);

  // Event types for filtering
  const eventTypes = [
    { value: 'opportunity', label: 'Opportunities', icon: SparklesIcon, color: 'bg-yellow-500' },
    { value: 'execution', label: 'Executions', icon: BoltIcon, color: 'bg-blue-500' },
    { value: 'risk', label: 'Risk Events', icon: ExclamationTriangleIcon, color: 'bg-red-500' },
    { value: 'wallet', label: 'Wallet Events', icon: WalletIcon, color: 'bg-green-500' },
  ];

  // Fetch recent events
  const { data: recentEvents, isLoading, refetch } = useQuery({
    queryKey: ['recent-webhook-events'],
    queryFn: () => api.getRecentEvents(100),
    refetchInterval: autoRefresh ? 5000 : false, // Refresh every 5 seconds if auto-refresh is on
  });

  // Fetch opportunity events
  const { data: opportunityEvents } = useQuery({
    queryKey: ['opportunity-events'],
    queryFn: () => api.getOpportunityEvents(20),
    refetchInterval: autoRefresh ? 10000 : false,
  });

  // Fetch execution events
  const { data: executionEvents } = useQuery({
    queryKey: ['execution-events'],
    queryFn: () => api.getExecutionEvents(20),
    refetchInterval: autoRefresh ? 10000 : false,
  });

  // Fetch risk events
  const { data: riskEvents } = useQuery({
    queryKey: ['risk-events'],
    queryFn: () => api.getRiskEvents(10),
    refetchInterval: autoRefresh ? 15000 : false,
  });

  // Fetch wallet events
  const { data: walletEvents } = useQuery({
    queryKey: ['wallet-events'],
    queryFn: () => api.getWalletEvents(undefined, 20),
    refetchInterval: autoRefresh ? 10000 : false,
  });

  // Filter events based on selected types
  const filteredEvents = recentEvents?.filter((event: any) => {
    if (selectedEventTypes.length === 0) return true;
    
    const eventType = event.event_type?.split('_')[0] || 'unknown';
    return selectedEventTypes.includes(eventType);
  }) || [];

  // Handle event type toggle
  const toggleEventType = (type: string) => {
    setSelectedEventTypes(prev =>
      prev.includes(type)
        ? prev.filter(t => t !== type)
        : [...prev, type]
    );
  };

  // Format timestamp
  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  // Get event type info
  const getEventTypeInfo = (eventType: string) => {
    const type = eventType?.split('_')[0] || 'unknown';
    return eventTypes.find(et => et.value === type) || {
      label: eventType,
      icon: BoltIcon,
      color: 'bg-gray-500'
    };
  };

  // Get event status color
  const getEventStatusColor = (event: any) => {
    if (event.outcome === 'success') return 'text-green-400';
    if (event.outcome === 'failure') return 'text-red-400';
    if (event.severity === 'critical') return 'text-red-400';
    if (event.severity === 'high') return 'text-orange-400';
    if (event.severity === 'medium') return 'text-yellow-400';
    return 'text-blue-400';
  };

  // Calculate event statistics
  const eventStats = {
    total: recentEvents?.length || 0,
    opportunities: opportunityEvents?.length || 0,
    executions: executionEvents?.length || 0,
    risks: riskEvents?.length || 0,
    wallets: walletEvents?.length || 0,
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <div className="p-2 bg-blue-500/20 rounded-lg">
            <BoltIcon className="h-6 w-6 text-blue-400" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">Webhook Events</h1>
            <p className="text-gray-400">Real-time events from the HFT Ninja system</p>
          </div>
        </div>
        
        <div className="flex items-center space-x-4">
          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={`flex items-center space-x-2 px-4 py-2 rounded-lg border transition-colors ${
              autoRefresh
                ? 'bg-green-600 border-green-500 text-white'
                : 'bg-gray-800 border-gray-700 text-gray-300 hover:border-gray-600'
            }`}
          >
            <ArrowPathIcon className={`h-4 w-4 ${autoRefresh ? 'animate-spin' : ''}`} />
            <span>Auto Refresh</span>
          </button>
          
          <button
            onClick={() => refetch()}
            className="flex items-center space-x-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
          >
            <ArrowPathIcon className="h-4 w-4" />
            <span>Refresh</span>
          </button>
        </div>
      </div>

      {/* Event Statistics */}
      <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
        <Card className="p-4">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-gray-500/20 rounded-lg">
              <ChartBarIcon className="h-5 w-5 text-gray-400" />
            </div>
            <div>
              <p className="text-sm text-gray-400">Total Events</p>
              <p className="text-xl font-bold text-white">{eventStats.total}</p>
            </div>
          </div>
        </Card>

        <Card className="p-4">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-yellow-500/20 rounded-lg">
              <SparklesIcon className="h-5 w-5 text-yellow-400" />
            </div>
            <div>
              <p className="text-sm text-gray-400">Opportunities</p>
              <p className="text-xl font-bold text-white">{eventStats.opportunities}</p>
            </div>
          </div>
        </Card>

        <Card className="p-4">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-blue-500/20 rounded-lg">
              <BoltIcon className="h-5 w-5 text-blue-400" />
            </div>
            <div>
              <p className="text-sm text-gray-400">Executions</p>
              <p className="text-xl font-bold text-white">{eventStats.executions}</p>
            </div>
          </div>
        </Card>

        <Card className="p-4">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-red-500/20 rounded-lg">
              <ExclamationTriangleIcon className="h-5 w-5 text-red-400" />
            </div>
            <div>
              <p className="text-sm text-gray-400">Risk Events</p>
              <p className="text-xl font-bold text-white">{eventStats.risks}</p>
            </div>
          </div>
        </Card>

        <Card className="p-4">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-green-500/20 rounded-lg">
              <WalletIcon className="h-5 w-5 text-green-400" />
            </div>
            <div>
              <p className="text-sm text-gray-400">Wallet Events</p>
              <p className="text-xl font-bold text-white">{eventStats.wallets}</p>
            </div>
          </div>
        </Card>
      </div>

      {/* Event Filters */}
      <Card className="p-6">
        <div className="space-y-4">
          <div className="flex items-center space-x-2">
            <FunnelIcon className="h-5 w-5 text-gray-400" />
            <h3 className="text-lg font-semibold text-white">Event Filters</h3>
          </div>

          <div className="flex flex-wrap gap-2">
            {eventTypes.map((type) => {
              const isSelected = selectedEventTypes.includes(type.value);
              const Icon = type.icon;
              
              return (
                <button
                  key={type.value}
                  onClick={() => toggleEventType(type.value)}
                  className={`flex items-center space-x-2 px-3 py-2 rounded-lg border transition-all ${
                    isSelected
                      ? 'bg-blue-600 border-blue-500 text-white'
                      : 'bg-gray-800 border-gray-700 text-gray-300 hover:border-gray-600'
                  }`}
                >
                  <div className={`w-2 h-2 rounded-full ${type.color}`} />
                  <Icon className="h-4 w-4" />
                  <span className="text-sm">{type.label}</span>
                </button>
              );
            })}
          </div>
        </div>
      </Card>

      {/* Event Stream */}
      <Card className="p-6">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white flex items-center space-x-2">
              <BoltIcon className="h-5 w-5 text-blue-400" />
              <span>Live Event Stream</span>
            </h3>
            <Badge variant="secondary">
              {filteredEvents.length} events
            </Badge>
          </div>

          {isLoading ? (
            <div className="flex justify-center py-8">
              <LoadingSpinner />
            </div>
          ) : filteredEvents.length > 0 ? (
            <div className="space-y-3 max-h-96 overflow-y-auto">
              {filteredEvents.map((event: any, index: number) => {
                const typeInfo = getEventTypeInfo(event.event_type);
                const Icon = typeInfo.icon;
                const statusColor = getEventStatusColor(event);

                return (
                  <div
                    key={`${event.event_type}-${event.timestamp}-${index}`}
                    className="p-4 bg-gray-800/50 rounded-lg border border-gray-700 hover:border-gray-600 transition-colors"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-start space-x-3">
                        <div className={`p-2 rounded ${typeInfo.color}/20 flex-shrink-0`}>
                          <Icon className={`h-5 w-5 text-${typeInfo.color.split('-')[1]}-400`} />
                        </div>
                        <div className="flex-1">
                          <div className="flex items-center space-x-2 mb-2">
                            <Badge variant="secondary">{typeInfo.label}</Badge>
                            {event.outcome && (
                              <Badge variant={event.outcome === 'success' ? 'success' : 'destructive'}>
                                {event.outcome}
                              </Badge>
                            )}
                            {event.severity && (
                              <Badge variant="outline">{event.severity}</Badge>
                            )}
                          </div>
                          
                          <p className="text-white mb-2">
                            {event.description || event.opportunity_type || event.strategy || 'Event occurred'}
                          </p>
                          
                          {event.token_address && (
                            <p className="text-sm text-gray-400 mb-1">
                              Token: <span className="font-mono">{event.token_address}</span>
                            </p>
                          )}
                          
                          {event.wallet_address && (
                            <p className="text-sm text-gray-400 mb-1">
                              Wallet: <span className="font-mono">{event.wallet_address}</span>
                            </p>
                          )}
                          
                          {event.pnl_sol !== undefined && (
                            <p className={`text-sm font-medium ${event.pnl_sol >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                              P&L: {event.pnl_sol >= 0 ? '+' : ''}{event.pnl_sol.toFixed(4)} SOL
                            </p>
                          )}
                          
                          <div className="flex items-center space-x-4 mt-2 text-xs text-gray-400">
                            <div className="flex items-center space-x-1">
                              <ClockIcon className="h-3 w-3" />
                              <span>{formatTimestamp(event.timestamp)}</span>
                            </div>
                            {event.confidence && (
                              <span>Confidence: {(event.confidence * 100).toFixed(0)}%</span>
                            )}
                            {event.execution_time_ms && (
                              <span>Execution: {event.execution_time_ms}ms</span>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="text-center py-8 text-gray-400">
              <BoltIcon className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>No events found</p>
              <p className="text-sm">Events will appear here as they occur</p>
            </div>
          )}
        </div>
      </Card>
    </div>
  );
};

export default WebhookEventsPage;
