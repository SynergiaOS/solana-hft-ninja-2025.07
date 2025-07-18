import React, { useState, useEffect } from 'react';
import { 
  Brain, 
  Users, 
  Shield, 
  CheckCircle, 
  Clock, 
  TrendingUp,
  AlertTriangle,
  Zap
} from 'lucide-react';

interface TradingDecision {
  action: string;
  token: string;
  amount: number;
  confidence: number;
  risk_level: string;
  approval_status: string;
}

interface EnhancementStatus {
  multi_agent: boolean;
  human_loop: boolean;
  advanced_confidence: boolean;
}

interface AnalysisResponse {
  response: string;
  sources: string[];
  confidence: number;
  execution_time: number;
  metadata: {
    enhancements_used: EnhancementStatus;
    trading_decision: TradingDecision;
    analysis_type: string;
  };
}

interface ApprovalRequest {
  request_id: string;
  decision: TradingDecision;
  created_at: string;
  expires_at: string;
  status: string;
}

// Simple UI Components
const Card: React.FC<{children: React.ReactNode, className?: string}> = ({children, className = ""}) => (
  <div className={`bg-[#1A1D29] border border-gray-700/50 rounded-xl ${className}`}>{children}</div>
);

const CardHeader: React.FC<{children: React.ReactNode}> = ({children}) => (
  <div className="p-6 border-b border-gray-700/50">{children}</div>
);

const CardTitle: React.FC<{children: React.ReactNode, className?: string}> = ({children, className = ""}) => (
  <h3 className={`text-xl font-semibold text-white ${className}`}>{children}</h3>
);

const CardContent: React.FC<{children: React.ReactNode, className?: string}> = ({children, className = ""}) => (
  <div className={`p-6 ${className}`}>{children}</div>
);

const Button: React.FC<{
  children: React.ReactNode,
  onClick?: () => void,
  disabled?: boolean,
  className?: string,
  size?: string,
  variant?: string
}> = ({children, onClick, disabled, className = "", size, variant}) => (
  <button
    onClick={onClick}
    disabled={disabled}
    className={`px-4 py-2 rounded-lg font-medium transition-colors ${
      disabled ? 'bg-gray-600 text-gray-400 cursor-not-allowed' :
      variant === 'outline' ? 'border border-gray-600 text-gray-300 hover:bg-gray-700' :
      'bg-purple-600 hover:bg-purple-700 text-white'
    } ${className}`}
  >
    {children}
  </button>
);

const Input: React.FC<{
  placeholder?: string,
  value: string,
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void,
  onKeyPress?: (e: React.KeyboardEvent<HTMLInputElement>) => void,
  className?: string
}> = ({placeholder, value, onChange, onKeyPress, className = ""}) => (
  <input
    type="text"
    placeholder={placeholder}
    value={value}
    onChange={onChange}
    onKeyPress={onKeyPress}
    className={`px-4 py-2 bg-gray-800 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-purple-500 ${className}`}
  />
);

const Badge: React.FC<{children: React.ReactNode, className?: string, size?: string}> = ({children, className = "", size}) => (
  <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${className}`}>
    {children}
  </span>
);

const Alert: React.FC<{children: React.ReactNode, className?: string}> = ({children, className = ""}) => (
  <div className={`p-4 rounded-lg border ${className}`}>{children}</div>
);

const AlertDescription: React.FC<{children: React.ReactNode, className?: string}> = ({children, className = ""}) => (
  <div className={className}>{children}</div>
);

const EnhancedAnalysisPanel: React.FC = () => {
  const [query, setQuery] = useState('');
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResponse | null>(null);
  const [approvalRequests, setApprovalRequests] = useState<ApprovalRequest[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Fetch pending approval requests
  const fetchApprovalRequests = async () => {
    try {
      const response = await fetch('/api/approval-requests');
      if (response.ok) {
        const data = await response.json();
        setApprovalRequests(data.pending_requests || []);
      }
    } catch (err) {
      console.error('Failed to fetch approval requests:', err);
    }
  };

  // Perform enhanced analysis
  const performEnhancedAnalysis = async () => {
    if (!query.trim()) return;

    setIsAnalyzing(true);
    setError(null);

    try {
      const response = await fetch('/api/enhanced-analysis', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          prompt: query,
          user_id: 'dashboard_user',
          context: {
            source: 'enhanced_panel',
            timestamp: new Date().toISOString()
          }
        }),
      });

      if (response.ok) {
        const result = await response.json();
        setAnalysisResult(result);
        
        // Refresh approval requests after analysis
        setTimeout(fetchApprovalRequests, 1000);
      } else {
        throw new Error(`Analysis failed: ${response.status}`);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Analysis failed');
    } finally {
      setIsAnalyzing(false);
    }
  };

  // Approve a request
  const approveRequest = async (requestId: string) => {
    try {
      const response = await fetch(`/api/approve-request/${requestId}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          approved_by: 'dashboard_user'
        }),
      });

      if (response.ok) {
        // Refresh approval requests
        fetchApprovalRequests();
      }
    } catch (err) {
      console.error('Failed to approve request:', err);
    }
  };

  // Auto-refresh approval requests
  useEffect(() => {
    fetchApprovalRequests();
    const interval = setInterval(fetchApprovalRequests, 5000); // Every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const getRiskColor = (riskLevel: string) => {
    switch (riskLevel?.toLowerCase()) {
      case 'low': return 'bg-green-100 text-green-800';
      case 'medium': return 'bg-yellow-100 text-yellow-800';
      case 'high': return 'bg-orange-100 text-orange-800';
      case 'critical': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getApprovalStatusColor = (status: string) => {
    switch (status?.toLowerCase()) {
      case 'auto_approved': return 'bg-green-100 text-green-800';
      case 'pending': return 'bg-yellow-100 text-yellow-800';
      case 'approved': return 'bg-blue-100 text-blue-800';
      case 'rejected': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div className="space-y-6">
      {/* Enhanced Analysis Input */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Brain className="h-5 w-5 text-purple-400" />
            TensorZero-Enhanced Analysis
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <Input
              placeholder="Ask about trading strategies, market analysis, or risk assessment..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && performEnhancedAnalysis()}
              className="flex-1"
            />
            <Button 
              onClick={performEnhancedAnalysis}
              disabled={isAnalyzing || !query.trim()}
              className="bg-purple-600 hover:bg-purple-700"
            >
              {isAnalyzing ? (
                <>
                  <Zap className="h-4 w-4 mr-2 animate-spin" />
                  Analyzing...
                </>
              ) : (
                <>
                  <Brain className="h-4 w-4 mr-2" />
                  Analyze
                </>
              )}
            </Button>
          </div>

          {error && (
            <Alert className="border-red-200 bg-red-50">
              <AlertTriangle className="h-4 w-4 text-red-600" />
              <AlertDescription className="text-red-800">
                {error}
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Analysis Results */}
      {analysisResult && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              <span>Analysis Results</span>
              <div className="flex gap-2">
                {analysisResult.metadata.enhancements_used.multi_agent && (
                  <Badge className="bg-blue-100 text-blue-800">
                    <Users className="h-3 w-3 mr-1" />
                    Multi-Agent
                  </Badge>
                )}
                {analysisResult.metadata.enhancements_used.human_loop && (
                  <Badge className="bg-green-100 text-green-800">
                    <Shield className="h-3 w-3 mr-1" />
                    Human Loop
                  </Badge>
                )}
                {analysisResult.metadata.enhancements_used.advanced_confidence && (
                  <Badge className="bg-purple-100 text-purple-800">
                    <TrendingUp className="h-3 w-3 mr-1" />
                    Advanced AI
                  </Badge>
                )}
              </div>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="bg-gray-50 p-4 rounded-lg">
              <pre className="whitespace-pre-wrap text-sm">
                {analysisResult.response}
              </pre>
            </div>

            {/* Trading Decision Summary */}
            {analysisResult.metadata.trading_decision && (
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 p-4 bg-blue-50 rounded-lg">
                <div>
                  <div className="text-sm font-medium text-gray-600">Action</div>
                  <div className="text-lg font-bold text-blue-800">
                    {analysisResult.metadata.trading_decision.action}
                  </div>
                </div>
                <div>
                  <div className="text-sm font-medium text-gray-600">Amount</div>
                  <div className="text-lg font-bold">
                    {analysisResult.metadata.trading_decision.amount} {analysisResult.metadata.trading_decision.token}
                  </div>
                </div>
                <div>
                  <div className="text-sm font-medium text-gray-600">Confidence</div>
                  <div className="text-lg font-bold text-green-600">
                    {(analysisResult.metadata.trading_decision.confidence * 100).toFixed(1)}%
                  </div>
                </div>
                <div>
                  <div className="text-sm font-medium text-gray-600">Risk Level</div>
                  <Badge className={getRiskColor(analysisResult.metadata.trading_decision.risk_level)}>
                    {analysisResult.metadata.trading_decision.risk_level}
                  </Badge>
                </div>
              </div>
            )}

            <div className="flex justify-between text-sm text-gray-600">
              <span>Execution Time: {analysisResult.execution_time.toFixed(2)}s</span>
              <span>Confidence: {(analysisResult.confidence * 100).toFixed(1)}%</span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Pending Approval Requests */}
      {approvalRequests.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Clock className="h-5 w-5 text-orange-600" />
              Pending Approvals ({approvalRequests.length})
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {approvalRequests.map((request) => (
              <div key={request.request_id} className="border rounded-lg p-4 space-y-3">
                <div className="flex justify-between items-start">
                  <div>
                    <div className="font-medium">
                      {request.decision.action} {request.decision.amount} {request.decision.token}
                    </div>
                    <div className="text-sm text-gray-600">
                      Request ID: {request.request_id}
                    </div>
                  </div>
                  <Badge className={getApprovalStatusColor(request.status)}>
                    {request.status}
                  </Badge>
                </div>

                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div>
                    <span className="font-medium">Confidence:</span> {(request.decision.confidence * 100).toFixed(1)}%
                  </div>
                  <div>
                    <span className="font-medium">Risk:</span> 
                    <Badge className={`ml-1 ${getRiskColor(request.decision.risk_level)}`} size="sm">
                      {request.decision.risk_level}
                    </Badge>
                  </div>
                  <div>
                    <span className="font-medium">Expires:</span> {new Date(request.expires_at).toLocaleTimeString()}
                  </div>
                </div>

                <div className="flex gap-2">
                  <Button
                    size="sm"
                    onClick={() => approveRequest(request.request_id)}
                    className="bg-green-600 hover:bg-green-700"
                  >
                    <CheckCircle className="h-4 w-4 mr-1" />
                    Approve
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    className="border-red-200 text-red-600 hover:bg-red-50"
                  >
                    <AlertTriangle className="h-4 w-4 mr-1" />
                    Reject
                  </Button>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      )}
    </div>
  );
};

export default EnhancedAnalysisPanel;
