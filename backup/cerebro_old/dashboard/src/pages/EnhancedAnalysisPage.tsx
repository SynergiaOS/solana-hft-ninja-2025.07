import React from 'react';
import { Brain, Zap, Users, Shield } from 'lucide-react';
import EnhancedAnalysisPanel from '@/components/EnhancedAnalysisPanel';

const EnhancedAnalysisPage: React.FC = () => {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white flex items-center gap-3">
            <Brain className="h-8 w-8 text-purple-400" />
            Enhanced Analysis
          </h1>
          <p className="text-gray-400 mt-2">
            TensorZero-inspired multi-agent analysis with human-in-the-loop approval
          </p>
        </div>
        
        <div className="flex gap-2">
          <div className="bg-purple-900/30 border border-purple-700/50 rounded-lg px-3 py-2">
            <div className="flex items-center gap-2 text-purple-300">
              <Zap className="h-4 w-4" />
              <span className="text-sm font-medium">AI Enhanced</span>
            </div>
          </div>
          <div className="bg-blue-900/30 border border-blue-700/50 rounded-lg px-3 py-2">
            <div className="flex items-center gap-2 text-blue-300">
              <Users className="h-4 w-4" />
              <span className="text-sm font-medium">Multi-Agent</span>
            </div>
          </div>
          <div className="bg-green-900/30 border border-green-700/50 rounded-lg px-3 py-2">
            <div className="flex items-center gap-2 text-green-300">
              <Shield className="h-4 w-4" />
              <span className="text-sm font-medium">Human Oversight</span>
            </div>
          </div>
        </div>
      </div>

      {/* Features Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-gradient-to-br from-purple-900/20 to-purple-800/10 border border-purple-700/30 rounded-xl p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-purple-600/20 rounded-lg">
              <Users className="h-6 w-6 text-purple-400" />
            </div>
            <h3 className="text-lg font-semibold text-white">Multi-Agent Collaboration</h3>
          </div>
          <p className="text-gray-300 text-sm leading-relaxed">
            Specialized agents work together: Sentiment Analyzer, Technical Analyst, 
            Risk Assessor, and Strategy Coordinator provide comprehensive analysis.
          </p>
          <div className="mt-4 space-y-2">
            <div className="flex items-center gap-2 text-xs text-purple-300">
              <div className="w-2 h-2 bg-purple-400 rounded-full"></div>
              Sentiment Analysis Agent
            </div>
            <div className="flex items-center gap-2 text-xs text-purple-300">
              <div className="w-2 h-2 bg-purple-400 rounded-full"></div>
              Technical Analysis Agent
            </div>
            <div className="flex items-center gap-2 text-xs text-purple-300">
              <div className="w-2 h-2 bg-purple-400 rounded-full"></div>
              Risk Assessment Agent
            </div>
          </div>
        </div>

        <div className="bg-gradient-to-br from-green-900/20 to-green-800/10 border border-green-700/30 rounded-xl p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-green-600/20 rounded-lg">
              <Shield className="h-6 w-6 text-green-400" />
            </div>
            <h3 className="text-lg font-semibold text-white">Human-in-the-Loop</h3>
          </div>
          <p className="text-gray-300 text-sm leading-relaxed">
            Confidence-based approval system routes high-risk or low-confidence 
            decisions to human operators for oversight and approval.
          </p>
          <div className="mt-4 space-y-2">
            <div className="flex items-center gap-2 text-xs text-green-300">
              <div className="w-2 h-2 bg-green-400 rounded-full"></div>
              Auto-approval for high confidence
            </div>
            <div className="flex items-center gap-2 text-xs text-green-300">
              <div className="w-2 h-2 bg-green-400 rounded-full"></div>
              Human review for risky trades
            </div>
            <div className="flex items-center gap-2 text-xs text-green-300">
              <div className="w-2 h-2 bg-green-400 rounded-full"></div>
              Real-time notifications
            </div>
          </div>
        </div>

        <div className="bg-gradient-to-br from-blue-900/20 to-blue-800/10 border border-blue-700/30 rounded-xl p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-blue-600/20 rounded-lg">
              <Brain className="h-6 w-6 text-blue-400" />
            </div>
            <h3 className="text-lg font-semibold text-white">Advanced AI</h3>
          </div>
          <p className="text-gray-300 text-sm leading-relaxed">
            Enhanced confidence scoring, risk assessment, and decision synthesis 
            using state-of-the-art AI models and collaborative intelligence.
          </p>
          <div className="mt-4 space-y-2">
            <div className="flex items-center gap-2 text-xs text-blue-300">
              <div className="w-2 h-2 bg-blue-400 rounded-full"></div>
              Deepseek-Math integration
            </div>
            <div className="flex items-center gap-2 text-xs text-blue-300">
              <div className="w-2 h-2 bg-blue-400 rounded-full"></div>
              FinGPT market analysis
            </div>
            <div className="flex items-center gap-2 text-xs text-blue-300">
              <div className="w-2 h-2 bg-blue-400 rounded-full"></div>
              Consensus mechanisms
            </div>
          </div>
        </div>
      </div>

      {/* Main Analysis Panel */}
      <div className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6">
        <EnhancedAnalysisPanel />
      </div>

      {/* Usage Examples */}
      <div className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6">
        <h3 className="text-xl font-semibold text-white mb-4">Example Queries</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="space-y-3">
            <h4 className="text-sm font-medium text-gray-300">Trading Analysis</h4>
            <div className="space-y-2">
              <div className="bg-gray-800/50 rounded-lg p-3 text-sm text-gray-300">
                "Should I buy SOL right now?"
              </div>
              <div className="bg-gray-800/50 rounded-lg p-3 text-sm text-gray-300">
                "Analyze the risk of a 1 SOL position in RAY"
              </div>
              <div className="bg-gray-800/50 rounded-lg p-3 text-sm text-gray-300">
                "What's the market sentiment for ORCA?"
              </div>
            </div>
          </div>
          
          <div className="space-y-3">
            <h4 className="text-sm font-medium text-gray-300">Strategy Optimization</h4>
            <div className="space-y-2">
              <div className="bg-gray-800/50 rounded-lg p-3 text-sm text-gray-300">
                "Optimize my sandwich strategy parameters"
              </div>
              <div className="bg-gray-800/50 rounded-lg p-3 text-sm text-gray-300">
                "Review my portfolio allocation"
              </div>
              <div className="bg-gray-800/50 rounded-lg p-3 text-sm text-gray-300">
                "Suggest improvements for arbitrage strategy"
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Performance Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6">
          <div className="flex items-center justify-between mb-2">
            <h4 className="text-sm font-medium text-gray-400">Analysis Speed</h4>
            <Zap className="h-4 w-4 text-yellow-400" />
          </div>
          <div className="text-2xl font-bold text-white">~2.5s</div>
          <div className="text-xs text-gray-500">Average response time</div>
        </div>

        <div className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6">
          <div className="flex items-center justify-between mb-2">
            <h4 className="text-sm font-medium text-gray-400">Accuracy</h4>
            <Brain className="h-4 w-4 text-purple-400" />
          </div>
          <div className="text-2xl font-bold text-white">94.2%</div>
          <div className="text-xs text-gray-500">Prediction accuracy</div>
        </div>

        <div className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6">
          <div className="flex items-center justify-between mb-2">
            <h4 className="text-sm font-medium text-gray-400">Auto-Approval</h4>
            <Shield className="h-4 w-4 text-green-400" />
          </div>
          <div className="text-2xl font-bold text-white">78%</div>
          <div className="text-xs text-gray-500">High confidence trades</div>
        </div>

        <div className="bg-[#1A1D29] border border-gray-700/50 rounded-xl p-6">
          <div className="flex items-center justify-between mb-2">
            <h4 className="text-sm font-medium text-gray-400">Consensus</h4>
            <Users className="h-4 w-4 text-blue-400" />
          </div>
          <div className="text-2xl font-bold text-white">3.2/4</div>
          <div className="text-xs text-gray-500">Avg agent agreement</div>
        </div>
      </div>
    </div>
  );
};

export default EnhancedAnalysisPage;
