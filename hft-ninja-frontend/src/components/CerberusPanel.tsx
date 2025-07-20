import React, { useState, useEffect } from 'react';

interface Position {
  mint: string;
  entryPrice: number;
  currentPrice: number;
  positionSizeSol: number;
  pnlPercent: number;
  status: 'open' | 'closed' | 'pending';
  strategy: string;
  ageSeconds: number;
  takeProfitTarget: number;
  stopLossTarget: number;
  timeoutSeconds: number;
}

interface CerberusMetrics {
  totalPositions: number;
  profitablePositions: number;
  totalValueSol: number;
  decisionLatency: number;
  executionLatency: number;
  successRate: number;
  emergencyStopEnabled: boolean;
  lastDecisionTime: string;
}

interface DecisionLog {
  timestamp: string;
  mint: string;
  decision: 'SELL' | 'BUY_MORE' | 'HOLD';
  reason: string;
  confidence: number;
}

const CerberusPanel: React.FC = () => {
  const [positions, setPositions] = useState<Position[]>([]);
  const [metrics, setMetrics] = useState<CerberusMetrics>({
    totalPositions: 0,
    profitablePositions: 0,
    totalValueSol: 0,
    decisionLatency: 0,
    executionLatency: 0,
    successRate: 0,
    emergencyStopEnabled: true,
    lastDecisionTime: new Date().toISOString()
  });
  const [decisionLogs, setDecisionLogs] = useState<DecisionLog[]>([]);
  const [isConnected, setIsConnected] = useState(false);

  // Mock data for development
  const mockPositions: Position[] = [
    {
      mint: 'So11111111111111111111111111111111111111112',
      entryPrice: 0.001,
      currentPrice: 0.0012,
      positionSizeSol: 0.1,
      pnlPercent: 20.0,
      status: 'open',
      strategy: 'sandwich_strategy',
      ageSeconds: 120,
      takeProfitTarget: 100.0,
      stopLossTarget: -25.0,
      timeoutSeconds: 600
    },
    {
      mint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
      entryPrice: 1.0,
      currentPrice: 0.98,
      positionSizeSol: 0.05,
      pnlPercent: -2.0,
      status: 'open',
      strategy: 'arbitrage_strategy',
      ageSeconds: 45,
      takeProfitTarget: 20.0,
      stopLossTarget: -10.0,
      timeoutSeconds: 300
    }
  ];

  const mockDecisionLogs: DecisionLog[] = [
    {
      timestamp: new Date(Date.now() - 30000).toISOString(),
      mint: 'So11111111111111111111111111111111111111112',
      decision: 'HOLD',
      reason: 'Within profit targets',
      confidence: 0.85
    },
    {
      timestamp: new Date(Date.now() - 60000).toISOString(),
      mint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
      decision: 'HOLD',
      reason: 'Market conditions stable',
      confidence: 0.75
    },
    {
      timestamp: new Date(Date.now() - 120000).toISOString(),
      mint: 'Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB',
      decision: 'SELL',
      reason: 'TAKE_PROFIT',
      confidence: 0.95
    }
  ];

  useEffect(() => {
    // Use mock data for now
    setIsConnected(false);
    setPositions(mockPositions);
    setDecisionLogs(mockDecisionLogs);
    
    // Calculate metrics from positions
    const totalPositions = mockPositions.length;
    const profitablePositions = mockPositions.filter(p => p.pnlPercent > 0).length;
    const totalValueSol = mockPositions.reduce((sum, p) => sum + p.positionSizeSol, 0);
    
    setMetrics({
      totalPositions,
      profitablePositions,
      totalValueSol,
      decisionLatency: 150 + Math.random() * 50, // 150-200ms
      executionLatency: 80 + Math.random() * 40,  // 80-120ms
      successRate: 97.3,
      emergencyStopEnabled: true,
      lastDecisionTime: new Date().toISOString()
    });

    // Simulate real-time updates
    const interval = setInterval(() => {
      setMetrics(prev => ({
        ...prev,
        decisionLatency: 150 + Math.random() * 50,
        executionLatency: 80 + Math.random() * 40,
        lastDecisionTime: new Date().toISOString()
      }));
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  const handleEmergencyStop = () => {
    if (window.confirm('Are you sure you want to trigger emergency stop? This will close ALL positions immediately.')) {
      // In real implementation, this would send emergency stop command
      alert('Emergency stop triggered! All positions will be closed.');
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'open': return 'text-green-500';
      case 'closed': return 'text-gray-500';
      case 'pending': return 'text-yellow-500';
      default: return 'text-gray-500';
    }
  };

  const getDecisionColor = (decision: string) => {
    switch (decision) {
      case 'SELL': return 'text-red-500';
      case 'BUY_MORE': return 'text-green-500';
      case 'HOLD': return 'text-blue-500';
      default: return 'text-gray-500';
    }
  };

  const formatMint = (mint: string) => {
    return `${mint.slice(0, 8)}...${mint.slice(-8)}`;
  };

  const formatTimeAgo = (timestamp: string) => {
    const seconds = Math.floor((Date.now() - new Date(timestamp).getTime()) / 1000);
    if (seconds < 60) return `${seconds}s ago`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    return `${Math.floor(seconds / 3600)}h ago`;
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold flex items-center">
              🧠 Cerberus Trade Execution Brain
            </h2>
            <p className="text-gray-400 mt-1">
              Autonomous position management with sub-second decision making
            </p>
          </div>
          <div className="flex items-center space-x-4">
            <div className="text-center">
              <div className="text-sm text-gray-400">Status</div>
              <div className="text-green-500 font-semibold">
                {isConnected ? '🟢 Connected' : '🔴 Mock Data'}
              </div>
            </div>
            <button
              onClick={handleEmergencyStop}
              className="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded-lg font-semibold transition-colors"
            >
              🚨 Emergency Stop
            </button>
          </div>
        </div>
      </div>

      {/* Metrics Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Active Positions</p>
              <p className="text-2xl font-bold">{metrics.totalPositions}</p>
            </div>
            <span className="text-4xl">📊</span>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Profitable</p>
              <p className="text-2xl font-bold text-green-500">
                {metrics.profitablePositions}/{metrics.totalPositions}
              </p>
            </div>
            <span className="text-4xl">💰</span>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Decision Latency</p>
              <p className="text-2xl font-bold">{metrics.decisionLatency.toFixed(0)}ms</p>
            </div>
            <span className="text-4xl">⚡</span>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Success Rate</p>
              <p className="text-2xl font-bold text-green-500">{metrics.successRate}%</p>
            </div>
            <span className="text-4xl">🎯</span>
          </div>
        </div>
      </div>

      {/* Positions and Decision Logs */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Active Positions */}
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-xl font-semibold mb-4">Active Positions</h3>
          <div className="space-y-3">
            {positions.map((position, index) => (
              <div key={index} className="bg-gray-700 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <span className="font-semibold">{formatMint(position.mint)}</span>
                  <span className={`text-sm font-semibold ${getStatusColor(position.status)}`}>
                    {position.status.toUpperCase()}
                  </span>
                </div>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-gray-400">Size:</span>
                    <span className="ml-2">{position.positionSizeSol} SOL</span>
                  </div>
                  <div>
                    <span className="text-gray-400">PnL:</span>
                    <span className={`ml-2 ${position.pnlPercent >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                      {position.pnlPercent >= 0 ? '+' : ''}{position.pnlPercent.toFixed(2)}%
                    </span>
                  </div>
                  <div>
                    <span className="text-gray-400">Age:</span>
                    <span className="ml-2">{position.ageSeconds}s</span>
                  </div>
                  <div>
                    <span className="text-gray-400">Strategy:</span>
                    <span className="ml-2">{position.strategy}</span>
                  </div>
                </div>
              </div>
            ))}
            {positions.length === 0 && (
              <div className="text-center text-gray-400 py-8">
                No active positions
              </div>
            )}
          </div>
        </div>

        {/* Decision Logs */}
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-xl font-semibold mb-4">Recent Decisions</h3>
          <div className="space-y-3">
            {decisionLogs.map((log, index) => (
              <div key={index} className="bg-gray-700 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <span className={`font-semibold ${getDecisionColor(log.decision)}`}>
                    {log.decision}
                  </span>
                  <span className="text-sm text-gray-400">
                    {formatTimeAgo(log.timestamp)}
                  </span>
                </div>
                <div className="text-sm">
                  <div className="text-gray-400 mb-1">
                    {formatMint(log.mint)}
                  </div>
                  <div className="text-gray-300">
                    {log.reason}
                  </div>
                  <div className="text-gray-400 mt-1">
                    Confidence: {(log.confidence * 100).toFixed(0)}%
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default CerberusPanel;
