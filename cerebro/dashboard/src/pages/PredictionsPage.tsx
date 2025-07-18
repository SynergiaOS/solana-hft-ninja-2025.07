import React, { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { toast } from 'react-hot-toast';
import { 
  SparklesIcon,
  TrendingUpIcon,
  TrendingDownIcon,
  ExclamationTriangleIcon,
  EyeIcon,
  ChartBarIcon,
  ClockIcon,
  CpuChipIcon,
  LightBulbIcon
} from '@heroicons/react/24/outline';

// Components
import Card from '@/components/ui/Card';
import LoadingSpinner from '@/components/ui/LoadingSpinner';
import Badge from '@/components/ui/Badge';

// Services
import { api } from '@/services/api';

// Types
import type { TradingPrediction, MarketAnalysisAI } from '@/types';

const PredictionsPage: React.FC = () => {
  const [selectedToken, setSelectedToken] = useState<string>('');
  const [analysisPrompt, setAnalysisPrompt] = useState<string>('');

  // Prediction types with icons and colors
  const predictionTypes = {
    'PriceIncrease': { icon: TrendingUpIcon, color: 'text-green-400', bg: 'bg-green-500/20' },
    'PriceDecrease': { icon: TrendingDownIcon, color: 'text-red-400', bg: 'bg-red-500/20' },
    'HighVolatility': { icon: ChartBarIcon, color: 'text-yellow-400', bg: 'bg-yellow-500/20' },
    'RugPull': { icon: ExclamationTriangleIcon, color: 'text-red-500', bg: 'bg-red-600/20' },
    'WhaleActivity': { icon: EyeIcon, color: 'text-blue-400', bg: 'bg-blue-500/20' },
  };

  // Fetch trading predictions
  const { data: predictions, isLoading: isLoadingPredictions, refetch: refetchPredictions } = useQuery({
    queryKey: ['trading-predictions', selectedToken],
    queryFn: () => api.getTradingPredictions(selectedToken || undefined),
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  // Fetch market analysis
  const { data: marketAnalysis, isLoading: isLoadingAnalysis } = useQuery({
    queryKey: ['market-analysis'],
    queryFn: () => api.getMarketAnalysis(),
    refetchInterval: 60000, // Refresh every minute
  });

  // AI analysis mutation
  const aiAnalysisMutation = useMutation({
    mutationFn: ({ prompt, context }: { prompt: string; context?: any }) =>
      api.requestAIAnalysis(prompt, context),
    onSuccess: (data) => {
      toast.success('AI analysis completed');
    },
    onError: (error) => {
      console.error('AI analysis failed:', error);
      toast.error('Failed to get AI analysis');
    },
  });

  // Handle AI analysis request
  const handleAIAnalysis = () => {
    if (!analysisPrompt.trim()) {
      toast.error('Please enter an analysis prompt');
      return;
    }

    aiAnalysisMutation.mutate({
      prompt: analysisPrompt,
      context: {
        selectedToken,
        currentPredictions: predictions,
        marketAnalysis,
      },
    });
  };

  // Format timestamp
  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  // Get prediction type info
  const getPredictionTypeInfo = (type: string) => {
    return predictionTypes[type as keyof typeof predictionTypes] || {
      icon: SparklesIcon,
      color: 'text-gray-400',
      bg: 'bg-gray-500/20'
    };
  };

  // Get confidence color
  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'text-green-400';
    if (confidence >= 0.6) return 'text-yellow-400';
    return 'text-red-400';
  };

  // Get risk level color
  const getRiskLevelColor = (level: string) => {
    switch (level.toLowerCase()) {
      case 'low': return 'text-green-400';
      case 'medium': return 'text-yellow-400';
      case 'high': return 'text-orange-400';
      case 'critical': return 'text-red-400';
      default: return 'text-gray-400';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <div className="p-2 bg-purple-500/20 rounded-lg">
            <CpuChipIcon className="h-6 w-6 text-purple-400" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">AI Predictions & Analysis</h1>
            <p className="text-gray-400">Advanced AI-powered trading predictions and market analysis</p>
          </div>
        </div>
        
        <button
          onClick={() => refetchPredictions()}
          className="flex items-center space-x-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg transition-colors"
        >
          <SparklesIcon className="h-4 w-4" />
          <span>Refresh Predictions</span>
        </button>
      </div>

      {/* Market Analysis Overview */}
      {marketAnalysis && (
        <Card className="p-6">
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-white flex items-center space-x-2">
              <ChartBarIcon className="h-5 w-5 text-blue-400" />
              <span>Market Analysis Overview</span>
            </h3>

            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
              <div className="p-4 bg-gray-800/50 rounded-lg">
                <p className="text-sm text-gray-400 mb-1">Market Trend</p>
                <p className="text-lg font-semibold text-white">{marketAnalysis.market_trend}</p>
              </div>
              
              <div className="p-4 bg-gray-800/50 rounded-lg">
                <p className="text-sm text-gray-400 mb-1">Sentiment</p>
                <p className={`text-lg font-semibold ${marketAnalysis.overall_sentiment >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                  {(marketAnalysis.overall_sentiment * 100).toFixed(1)}%
                </p>
              </div>
              
              <div className="p-4 bg-gray-800/50 rounded-lg">
                <p className="text-sm text-gray-400 mb-1">Volatility</p>
                <p className="text-lg font-semibold text-yellow-400">
                  {(marketAnalysis.volatility_index * 100).toFixed(1)}%
                </p>
              </div>
              
              <div className="p-4 bg-gray-800/50 rounded-lg">
                <p className="text-sm text-gray-400 mb-1">Risk Level</p>
                <p className={`text-lg font-semibold ${getRiskLevelColor(marketAnalysis.risk_level)}`}>
                  {marketAnalysis.risk_level}
                </p>
              </div>
            </div>

            {marketAnalysis.key_insights && marketAnalysis.key_insights.length > 0 && (
              <div className="space-y-2">
                <h4 className="text-md font-medium text-white">Key Insights:</h4>
                <div className="space-y-1">
                  {marketAnalysis.key_insights.map((insight, index) => (
                    <div key={index} className="flex items-start space-x-2">
                      <LightBulbIcon className="h-4 w-4 text-yellow-400 mt-0.5 flex-shrink-0" />
                      <p className="text-sm text-gray-300">{insight}</p>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </Card>
      )}

      {/* Token Filter */}
      <Card className="p-6">
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-white">Filter Predictions</h3>
          <div className="flex space-x-4">
            <div className="flex-1">
              <input
                type="text"
                value={selectedToken}
                onChange={(e) => setSelectedToken(e.target.value)}
                placeholder="Enter token address to filter predictions..."
                className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              />
            </div>
            <button
              onClick={() => setSelectedToken('')}
              className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors"
            >
              Clear
            </button>
          </div>
        </div>
      </Card>

      {/* AI Analysis Interface */}
      <Card className="p-6">
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-white flex items-center space-x-2">
            <CpuChipIcon className="h-5 w-5 text-purple-400" />
            <span>Custom AI Analysis</span>
          </h3>

          <div className="space-y-4">
            <textarea
              value={analysisPrompt}
              onChange={(e) => setAnalysisPrompt(e.target.value)}
              placeholder="Ask the AI to analyze market conditions, specific tokens, or trading strategies..."
              rows={3}
              className="w-full px-4 py-3 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent resize-none"
            />
            
            <button
              onClick={handleAIAnalysis}
              disabled={aiAnalysisMutation.isPending}
              className="flex items-center space-x-2 px-6 py-3 bg-purple-600 hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-colors"
            >
              {aiAnalysisMutation.isPending ? (
                <LoadingSpinner size="sm" />
              ) : (
                <SparklesIcon className="h-5 w-5" />
              )}
              <span>Analyze</span>
            </button>
          </div>

          {aiAnalysisMutation.data && (
            <div className="p-4 bg-purple-900/20 border border-purple-700/50 rounded-lg">
              <h4 className="text-md font-medium text-purple-300 mb-2">AI Analysis Result:</h4>
              <p className="text-white whitespace-pre-wrap">{JSON.stringify(aiAnalysisMutation.data, null, 2)}</p>
            </div>
          )}
        </div>
      </Card>

      {/* Trading Predictions */}
      <Card className="p-6">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white flex items-center space-x-2">
              <SparklesIcon className="h-5 w-5 text-purple-400" />
              <span>Trading Predictions</span>
            </h3>
            <Badge variant="secondary">
              {predictions?.length || 0} predictions
            </Badge>
          </div>

          {isLoadingPredictions ? (
            <div className="flex justify-center py-8">
              <LoadingSpinner />
            </div>
          ) : predictions && predictions.length > 0 ? (
            <div className="space-y-4">
              {predictions.map((prediction: TradingPrediction, index: number) => {
                const typeInfo = getPredictionTypeInfo(prediction.prediction_type);
                const Icon = typeInfo.icon;

                return (
                  <div
                    key={`${prediction.token_address}-${index}`}
                    className="p-4 bg-gray-800/50 rounded-lg border border-gray-700 hover:border-gray-600 transition-colors"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-start space-x-3">
                        <div className={`p-2 rounded ${typeInfo.bg}`}>
                          <Icon className={`h-5 w-5 ${typeInfo.color}`} />
                        </div>
                        <div className="flex-1">
                          <div className="flex items-center space-x-2 mb-2">
                            <Badge variant="secondary">{prediction.prediction_type}</Badge>
                            <Badge variant="outline" className={getConfidenceColor(prediction.confidence)}>
                              {(prediction.confidence * 100).toFixed(0)}% confidence
                            </Badge>
                          </div>
                          
                          <p className="text-white mb-2">
                            Token: <span className="font-mono text-blue-300">{prediction.token_address}</span>
                          </p>
                          
                          {prediction.price_target && (
                            <p className="text-sm text-gray-300 mb-1">
                              Price Target: <span className="text-green-400">${prediction.price_target.toFixed(6)}</span>
                            </p>
                          )}
                          
                          <p className="text-sm text-gray-300 mb-2">
                            Time Horizon: {prediction.time_horizon_minutes} minutes
                          </p>
                          
                          <p className="text-sm text-gray-300 mb-3">{prediction.reasoning}</p>
                          
                          <div className="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs">
                            <div>
                              <span className="text-gray-400">Risk Score:</span>
                              <span className={`ml-1 font-medium ${getConfidenceColor(1 - prediction.risk_score)}`}>
                                {(prediction.risk_score * 100).toFixed(0)}%
                              </span>
                            </div>
                            <div>
                              <span className="text-gray-400">Sentiment:</span>
                              <span className={`ml-1 font-medium ${prediction.sentiment_score >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                                {(prediction.sentiment_score * 100).toFixed(0)}%
                              </span>
                            </div>
                            {Object.entries(prediction.technical_indicators).slice(0, 2).map(([key, value]) => (
                              <div key={key}>
                                <span className="text-gray-400">{key}:</span>
                                <span className="ml-1 font-medium text-blue-400">
                                  {typeof value === 'number' ? value.toFixed(2) : value}
                                </span>
                              </div>
                            ))}
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
              <SparklesIcon className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>No predictions available</p>
              <p className="text-sm">AI predictions will appear here as they are generated</p>
            </div>
          )}
        </div>
      </Card>
    </div>
  );
};

export default PredictionsPage;
