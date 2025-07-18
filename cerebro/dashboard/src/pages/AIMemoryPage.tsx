import React, { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'react-hot-toast';
import { 
  MagnifyingGlassIcon, 
  BrainIcon, 
  ClockIcon,
  TagIcon,
  SparklesIcon,
  DocumentTextIcon,
  ChartBarIcon,
  ExclamationTriangleIcon
} from '@heroicons/react/24/outline';

// Components
import Card from '@/components/ui/Card';
import LoadingSpinner from '@/components/ui/LoadingSpinner';
import Badge from '@/components/ui/Badge';

// Services
import { api } from '@/services/api';

// Types
import type { ContextEntry, RAGSearchResult } from '@/types';

const AIMemoryPage: React.FC = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedContextTypes, setSelectedContextTypes] = useState<string[]>([]);
  const [searchResults, setSearchResults] = useState<RAGSearchResult[]>([]);
  const queryClient = useQueryClient();

  // Context types for filtering
  const contextTypes = [
    { value: 'MEV_OPPORTUNITY', label: 'MEV Opportunities', icon: SparklesIcon, color: 'bg-yellow-500' },
    { value: 'TRADE_OUTCOME_SUCCESS', label: 'Successful Trades', icon: ChartBarIcon, color: 'bg-green-500' },
    { value: 'TRADE_OUTCOME_FAILURE', label: 'Failed Trades', icon: ExclamationTriangleIcon, color: 'bg-red-500' },
    { value: 'RISK_ALERT', label: 'Risk Alerts', icon: ExclamationTriangleIcon, color: 'bg-orange-500' },
    { value: 'WALLET_ACTIVITY', label: 'Wallet Activity', icon: DocumentTextIcon, color: 'bg-blue-500' },
  ];

  // Fetch recent context entries
  const { data: recentEntries, isLoading: isLoadingRecent } = useQuery({
    queryKey: ['recent-context-entries'],
    queryFn: () => api.getRecentEvents(50),
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  // Search memory mutation
  const searchMemoryMutation = useMutation({
    mutationFn: ({ query, types }: { query: string; types?: string[] }) =>
      api.searchMemory(query, types),
    onSuccess: (data) => {
      setSearchResults(data);
      toast.success(`Found ${data.length} relevant memories`);
    },
    onError: (error) => {
      console.error('Memory search failed:', error);
      toast.error('Failed to search memory');
    },
  });

  // Store context mutation
  const storeContextMutation = useMutation({
    mutationFn: (context: Partial<ContextEntry>) => api.storeContext(context),
    onSuccess: () => {
      toast.success('Context stored successfully');
      queryClient.invalidateQueries({ queryKey: ['recent-context-entries'] });
    },
    onError: (error) => {
      console.error('Failed to store context:', error);
      toast.error('Failed to store context');
    },
  });

  // Handle search
  const handleSearch = () => {
    if (!searchQuery.trim()) {
      toast.error('Please enter a search query');
      return;
    }

    searchMemoryMutation.mutate({
      query: searchQuery,
      types: selectedContextTypes.length > 0 ? selectedContextTypes : undefined,
    });
  };

  // Handle context type toggle
  const toggleContextType = (type: string) => {
    setSelectedContextTypes(prev =>
      prev.includes(type)
        ? prev.filter(t => t !== type)
        : [...prev, type]
    );
  };

  // Format timestamp
  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  // Get context type info
  const getContextTypeInfo = (type: string) => {
    return contextTypes.find(ct => ct.value === type) || {
      label: type,
      icon: DocumentTextIcon,
      color: 'bg-gray-500'
    };
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <div className="p-2 bg-purple-500/20 rounded-lg">
            <BrainIcon className="h-6 w-6 text-purple-400" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">AI Memory & RAG Search</h1>
            <p className="text-gray-400">Search and explore the system's memory and learned patterns</p>
          </div>
        </div>
      </div>

      {/* Search Interface */}
      <Card className="p-6">
        <div className="space-y-4">
          <div className="flex items-center space-x-2">
            <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
            <h3 className="text-lg font-semibold text-white">Memory Search</h3>
          </div>

          {/* Search Input */}
          <div className="flex space-x-4">
            <div className="flex-1">
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
                placeholder="Search for patterns, trades, wallets, or insights..."
                className="w-full px-4 py-3 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              />
            </div>
            <button
              onClick={handleSearch}
              disabled={searchMemoryMutation.isPending}
              className="px-6 py-3 bg-purple-600 hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-colors flex items-center space-x-2"
            >
              {searchMemoryMutation.isPending ? (
                <LoadingSpinner size="sm" />
              ) : (
                <MagnifyingGlassIcon className="h-5 w-5" />
              )}
              <span>Search</span>
            </button>
          </div>

          {/* Context Type Filters */}
          <div className="space-y-2">
            <label className="text-sm font-medium text-gray-300">Filter by Context Type:</label>
            <div className="flex flex-wrap gap-2">
              {contextTypes.map((type) => {
                const isSelected = selectedContextTypes.includes(type.value);
                const Icon = type.icon;
                
                return (
                  <button
                    key={type.value}
                    onClick={() => toggleContextType(type.value)}
                    className={`flex items-center space-x-2 px-3 py-2 rounded-lg border transition-all ${
                      isSelected
                        ? 'bg-purple-600 border-purple-500 text-white'
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
        </div>
      </Card>

      {/* Search Results */}
      {searchResults.length > 0 && (
        <Card className="p-6">
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-white flex items-center space-x-2">
              <SparklesIcon className="h-5 w-5 text-purple-400" />
              <span>Search Results ({searchResults.length})</span>
            </h3>

            <div className="space-y-4">
              {searchResults.map((result, index) => {
                const typeInfo = getContextTypeInfo(result.context_entry.context_type);
                const Icon = typeInfo.icon;

                return (
                  <div
                    key={`${result.context_entry.context_id}-${index}`}
                    className="p-4 bg-gray-800/50 rounded-lg border border-gray-700 hover:border-gray-600 transition-colors"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1 space-y-2">
                        <div className="flex items-center space-x-3">
                          <div className={`p-1 rounded ${typeInfo.color}/20`}>
                            <Icon className={`h-4 w-4 text-${typeInfo.color.split('-')[1]}-400`} />
                          </div>
                          <Badge variant="secondary">{typeInfo.label}</Badge>
                          <Badge variant="outline">
                            Similarity: {(result.similarity_score * 100).toFixed(1)}%
                          </Badge>
                        </div>
                        
                        <p className="text-white">{result.context_entry.content}</p>
                        
                        {result.relevance_explanation && (
                          <p className="text-sm text-purple-300 italic">
                            💡 {result.relevance_explanation}
                          </p>
                        )}
                        
                        <div className="flex items-center space-x-4 text-sm text-gray-400">
                          <div className="flex items-center space-x-1">
                            <ClockIcon className="h-4 w-4" />
                            <span>{formatTimestamp(result.context_entry.timestamp)}</span>
                          </div>
                          <div className="flex items-center space-x-1">
                            <TagIcon className="h-4 w-4" />
                            <span>{result.context_entry.source}</span>
                          </div>
                          {result.context_entry.related_strategy && (
                            <Badge variant="outline" size="sm">
                              {result.context_entry.related_strategy}
                            </Badge>
                          )}
                        </div>
                      </div>
                      
                      <div className="text-right">
                        <div className="text-sm font-medium text-purple-400">
                          Confidence: {(result.context_entry.confidence * 100).toFixed(0)}%
                        </div>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </Card>
      )}

      {/* Recent Context Entries */}
      <Card className="p-6">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white flex items-center space-x-2">
              <ClockIcon className="h-5 w-5 text-blue-400" />
              <span>Recent Memory Entries</span>
            </h3>
            <Badge variant="secondary">
              {recentEntries?.length || 0} entries
            </Badge>
          </div>

          {isLoadingRecent ? (
            <div className="flex justify-center py-8">
              <LoadingSpinner />
            </div>
          ) : recentEntries && recentEntries.length > 0 ? (
            <div className="space-y-3 max-h-96 overflow-y-auto">
              {recentEntries.slice(0, 20).map((entry: any, index: number) => {
                const typeInfo = getContextTypeInfo(entry.context_type || entry.event_type);
                const Icon = typeInfo.icon;

                return (
                  <div
                    key={`${entry.context_id || entry.event_type}-${index}`}
                    className="p-3 bg-gray-800/30 rounded-lg border border-gray-700/50"
                  >
                    <div className="flex items-start space-x-3">
                      <div className={`p-1 rounded ${typeInfo.color}/20 flex-shrink-0`}>
                        <Icon className={`h-4 w-4 text-${typeInfo.color.split('-')[1]}-400`} />
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="text-sm text-white truncate">
                          {entry.content || entry.description || 'No description available'}
                        </p>
                        <div className="flex items-center space-x-2 mt-1">
                          <Badge variant="outline" size="sm">{typeInfo.label}</Badge>
                          <span className="text-xs text-gray-400">
                            {formatTimestamp(entry.timestamp)}
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="text-center py-8 text-gray-400">
              <BrainIcon className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>No recent memory entries found</p>
            </div>
          )}
        </div>
      </Card>
    </div>
  );
};

export default AIMemoryPage;
