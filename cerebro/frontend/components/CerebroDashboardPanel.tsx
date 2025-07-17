import React, { useState, useEffect } from 'react';
import { 
  Brain, 
  TrendingUp, 
  AlertTriangle, 
  CheckCircle, 
  Clock, 
  Zap,
  BarChart3,
  MessageSquare,
  Settings,
  RefreshCw,
  Activity
} from 'lucide-react';

interface CerebroStatus {
  status: 'active' | 'inactive' | 'error';
  session_id: string;
  metrics: {
    total_queries: number;
    successful_responses: number;
    average_response_time: number;
    total_actions_executed: number;
    memory_entries_created: number;
  };
  components: {
    memory_manager: string;
    llm_router: string;
    tools_count: number;
    langgraph_flow: string;
  };
  conversation_length: number;
  timestamp: string;
}

interface Recommendation {
  type: string;
  title: string;
  description: string;
  priority: 'low' | 'medium' | 'high';
  action: string;
}

interface RecentAnalysis {
  id: string;
  type: string;
  title: string;
  summary: string;
  timestamp: string;
  status: 'completed' | 'running' | 'failed';
  metrics?: {
    execution_time?: number;
    actions_executed?: number;
    insights_generated?: number;
  };
}

interface CerebroDashboardPanelProps {
  apiUrl?: string;
  className?: string;
  onChatOpen?: () => void;
}

const CerebroDashboardPanel: React.FC<CerebroDashboardPanelProps> = ({
  apiUrl = 'http://localhost:8000',
  className = '',
  onChatOpen
}) => {
  const [status, setStatus] = useState<CerebroStatus | null>(null);
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [recentAnalyses, setRecentAnalyses] = useState<RecentAnalysis[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  // Fetch Cerebro status
  const fetchStatus = async () => {
    try {
      const response = await fetch(`${apiUrl}/health`);
      if (!response.ok) throw new Error('Failed to fetch status');
      
      const data = await response.json();
      
      // Transform health data to status format
      const cerebroStatus: CerebroStatus = {
        status: data.services?.dragonflydb === 'healthy' ? 'active' : 'error',
        session_id: 'dashboard_session',
        metrics: {
          total_queries: 0,
          successful_responses: 0,
          average_response_time: 0,
          total_actions_executed: 0,
          memory_entries_created: 0
        },
        components: {
          memory_manager: data.services?.dragonflydb || 'unknown',
          llm_router: 'active',
          tools_count: 5,
          langgraph_flow: 'active'
        },
        conversation_length: 0,
        timestamp: data.timestamp
      };
      
      setStatus(cerebroStatus);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      setStatus(null);
    }
  };

  // Fetch recommendations
  const fetchRecommendations = async () => {
    try {
      // Mock recommendations - in production, this would call the actual API
      const mockRecommendations: Recommendation[] = [
        {
          type: 'performance_optimization',
          title: 'Optimize Sandwich Strategy',
          description: 'Success rate has dropped to 82%. Consider adjusting timing parameters.',
          priority: 'high',
          action: 'Review strategy settings'
        },
        {
          type: 'market_opportunity',
          title: 'High Volume Detected',
          description: 'SOL/USDC pair showing 300% volume increase. Monitor for arbitrage opportunities.',
          priority: 'medium',
          action: 'Enable aggressive mode'
        },
        {
          type: 'system_maintenance',
          title: 'Memory Cleanup Recommended',
          description: 'DragonflyDB memory usage at 75%. Consider running cleanup routine.',
          priority: 'low',
          action: 'Schedule maintenance'
        }
      ];
      
      setRecommendations(mockRecommendations);
    } catch (err) {
      console.error('Failed to fetch recommendations:', err);
    }
  };

  // Fetch recent analyses
  const fetchRecentAnalyses = async () => {
    try {
      // Mock recent analyses - in production, this would call the actual API
      const mockAnalyses: RecentAnalysis[] = [
        {
          id: '1',
          type: 'hourly_performance',
          title: 'Hourly Performance Review',
          summary: 'Generated 0.15 SOL profit with 89% success rate',
          timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
          status: 'completed',
          metrics: {
            execution_time: 2500,
            actions_executed: 8,
            insights_generated: 3
          }
        },
        {
          id: '2',
          type: 'market_sentiment',
          title: 'Market Sentiment Analysis',
          summary: 'Bullish sentiment detected (0.72/1.0) with high social volume',
          timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
          status: 'completed',
          metrics: {
            execution_time: 4200,
            actions_executed: 12,
            insights_generated: 5
          }
        },
        {
          id: '3',
          type: 'strategy_optimization',
          title: 'Strategy Optimization',
          summary: 'Analyzing arbitrage strategy performance...',
          timestamp: new Date().toISOString(),
          status: 'running'
        }
      ];
      
      setRecentAnalyses(mockAnalyses);
    } catch (err) {
      console.error('Failed to fetch recent analyses:', err);
    }
  };

  // Initial load and periodic refresh
  useEffect(() => {
    const loadData = async () => {
      setIsLoading(true);
      await Promise.all([
        fetchStatus(),
        fetchRecommendations(),
        fetchRecentAnalyses()
      ]);
      setIsLoading(false);
      setLastUpdate(new Date());
    };

    loadData();
    
    // Refresh every 30 seconds
    const interval = setInterval(loadData, 30000);
    return () => clearInterval(interval);
  }, [apiUrl]);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'text-green-600 bg-green-100';
      case 'inactive': return 'text-yellow-600 bg-yellow-100';
      case 'error': return 'text-red-600 bg-red-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high': return 'text-red-600 bg-red-100';
      case 'medium': return 'text-yellow-600 bg-yellow-100';
      case 'low': return 'text-blue-600 bg-blue-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`;
    return date.toLocaleDateString();
  };

  if (isLoading) {
    return (
      <div className={`bg-white rounded-lg shadow-lg p-6 ${className}`}>
        <div className="flex items-center justify-center h-64">
          <RefreshCw className="w-8 h-8 animate-spin text-purple-600" />
        </div>
      </div>
    );
  }

  return (
    <div className={`bg-white rounded-lg shadow-lg ${className}`}>
      {/* Header */}
      <div className="p-6 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <Brain className="w-8 h-8 text-purple-600" />
            <div>
              <h2 className="text-xl font-bold text-gray-900">Cerebro AI Assistant</h2>
              <p className="text-sm text-gray-500">Intelligent Trading Analysis</p>
            </div>
          </div>
          
          <div className="flex items-center space-x-2">
            {status && (
              <span className={`px-3 py-1 rounded-full text-xs font-medium ${getStatusColor(status.status)}`}>
                {status.status.toUpperCase()}
              </span>
            )}
            <button
              onClick={onChatOpen}
              className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 flex items-center space-x-2"
            >
              <MessageSquare className="w-4 h-4" />
              <span>Chat</span>
            </button>
          </div>
        </div>
      </div>

      <div className="p-6 space-y-6">
        {/* Status Overview */}
        {status && (
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="bg-blue-50 p-4 rounded-lg">
              <div className="flex items-center space-x-2">
                <BarChart3 className="w-5 h-5 text-blue-600" />
                <span className="text-sm font-medium text-blue-900">Queries</span>
              </div>
              <p className="text-2xl font-bold text-blue-900 mt-1">
                {status.metrics.total_queries}
              </p>
            </div>
            
            <div className="bg-green-50 p-4 rounded-lg">
              <div className="flex items-center space-x-2">
                <CheckCircle className="w-5 h-5 text-green-600" />
                <span className="text-sm font-medium text-green-900">Success Rate</span>
              </div>
              <p className="text-2xl font-bold text-green-900 mt-1">
                {status.metrics.total_queries > 0 
                  ? Math.round((status.metrics.successful_responses / status.metrics.total_queries) * 100)
                  : 0}%
              </p>
            </div>
            
            <div className="bg-purple-50 p-4 rounded-lg">
              <div className="flex items-center space-x-2">
                <Zap className="w-5 h-5 text-purple-600" />
                <span className="text-sm font-medium text-purple-900">Avg Response</span>
              </div>
              <p className="text-2xl font-bold text-purple-900 mt-1">
                {Math.round(status.metrics.average_response_time)}ms
              </p>
            </div>
            
            <div className="bg-orange-50 p-4 rounded-lg">
              <div className="flex items-center space-x-2">
                <Activity className="w-5 h-5 text-orange-600" />
                <span className="text-sm font-medium text-orange-900">Actions</span>
              </div>
              <p className="text-2xl font-bold text-orange-900 mt-1">
                {status.metrics.total_actions_executed}
              </p>
            </div>
          </div>
        )}

        {/* Recommendations */}
        <div>
          <h3 className="text-lg font-semibold text-gray-900 mb-3 flex items-center space-x-2">
            <TrendingUp className="w-5 h-5" />
            <span>AI Recommendations</span>
          </h3>
          
          <div className="space-y-3">
            {recommendations.map((rec, index) => (
              <div key={index} className="border border-gray-200 rounded-lg p-4 hover:bg-gray-50">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2 mb-1">
                      <h4 className="font-medium text-gray-900">{rec.title}</h4>
                      <span className={`px-2 py-1 rounded text-xs font-medium ${getPriorityColor(rec.priority)}`}>
                        {rec.priority.toUpperCase()}
                      </span>
                    </div>
                    <p className="text-sm text-gray-600 mb-2">{rec.description}</p>
                    <button className="text-sm text-purple-600 hover:text-purple-800 font-medium">
                      {rec.action} →
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Recent Analyses */}
        <div>
          <h3 className="text-lg font-semibold text-gray-900 mb-3 flex items-center space-x-2">
            <Clock className="w-5 h-5" />
            <span>Recent Analyses</span>
          </h3>
          
          <div className="space-y-3">
            {recentAnalyses.map((analysis) => (
              <div key={analysis.id} className="border border-gray-200 rounded-lg p-4">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2 mb-1">
                      <h4 className="font-medium text-gray-900">{analysis.title}</h4>
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        analysis.status === 'completed' ? 'text-green-600 bg-green-100' :
                        analysis.status === 'running' ? 'text-blue-600 bg-blue-100' :
                        'text-red-600 bg-red-100'
                      }`}>
                        {analysis.status.toUpperCase()}
                      </span>
                    </div>
                    <p className="text-sm text-gray-600 mb-2">{analysis.summary}</p>
                    
                    {analysis.metrics && (
                      <div className="flex items-center space-x-4 text-xs text-gray-500">
                        <span>⚡ {analysis.metrics.execution_time}ms</span>
                        <span>🔧 {analysis.metrics.actions_executed} actions</span>
                        {analysis.metrics.insights_generated && (
                          <span>💡 {analysis.metrics.insights_generated} insights</span>
                        )}
                      </div>
                    )}
                  </div>
                  
                  <div className="text-xs text-gray-500 ml-4">
                    {formatTimestamp(analysis.timestamp)}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Footer */}
        <div className="text-center text-xs text-gray-500 pt-4 border-t border-gray-200">
          Last updated: {lastUpdate.toLocaleTimeString()}
          {error && (
            <div className="mt-2 text-red-600 flex items-center justify-center space-x-1">
              <AlertTriangle className="w-4 h-4" />
              <span>{error}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default CerebroDashboardPanel;
