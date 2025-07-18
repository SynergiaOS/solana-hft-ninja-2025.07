import React, { useState, useEffect } from 'react';

interface TradingSignal {
  action: 'buy' | 'sell' | 'hold';
  confidence: number;
  reason: string;
  price: number;
  timestamp: string;
}

interface Position {
  token: string;
  amount: number;
  entryPrice: number;
  currentPrice: number;
  pnl: number;
  pnlPercent: number;
}

interface AIRecommendation {
  position_size: number;
  kelly_fraction: number;
  risk_score: number;
  max_loss: number;
  confidence: number;
}

const TradingPanel: React.FC = () => {
  const [currentSignal, setCurrentSignal] = useState<TradingSignal>({
    action: 'hold',
    confidence: 0.75,
    reason: 'Market conditions stable, waiting for better opportunity',
    price: 23.45,
    timestamp: new Date().toISOString()
  });

  const [positions, setPositions] = useState<Position[]>([
    {
      token: 'SOL',
      amount: 2.5,
      entryPrice: 22.80,
      currentPrice: 23.45,
      pnl: 1.625,
      pnlPercent: 2.85
    },
    {
      token: 'USDC',
      amount: 450.0,
      entryPrice: 1.0,
      currentPrice: 1.0,
      pnl: 0,
      pnlPercent: 0
    }
  ]);

  const [aiRecommendation, setAiRecommendation] = useState<AIRecommendation | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [tradeAmount, setTradeAmount] = useState<string>('1.0');
  const [selectedToken, setSelectedToken] = useState<string>('SOL');

  // Fetch AI recommendation
  const fetchAIRecommendation = async () => {
    setIsLoading(true);
    try {
      const response = await fetch('http://localhost:8002/ai/calculate/position-size', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          capital: 8.0,
          risk_tolerance: 0.05,
          expected_return: 0.15,
          volatility: 0.3,
          strategy: 'wallet_tracker'
        }),
      });

      if (response.ok) {
        const data = await response.json();
        setAiRecommendation(data.result);
      }
    } catch (error) {
      console.error('Failed to fetch AI recommendation:', error);
    } finally {
      setIsLoading(false);
    }
  };

  // Execute trade action
  const executeTrade = async (action: 'buy' | 'sell' | 'hold') => {
    setIsLoading(true);
    try {
      // Simulate API call to trading engine
      const response = await fetch('http://localhost:8002/api/trading/execute', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          action,
          token: selectedToken,
          amount: parseFloat(tradeAmount),
          strategy: 'manual'
        }),
      });

      if (response.ok) {
        // Update signal based on action
        setCurrentSignal({
          action,
          confidence: 0.95,
          reason: `Manual ${action} order executed`,
          price: currentSignal.price,
          timestamp: new Date().toISOString()
        });

        // Simulate position update
        if (action === 'buy') {
          setPositions(prev => prev.map(pos => 
            pos.token === selectedToken 
              ? { ...pos, amount: pos.amount + parseFloat(tradeAmount) }
              : pos
          ));
        } else if (action === 'sell') {
          setPositions(prev => prev.map(pos => 
            pos.token === selectedToken 
              ? { ...pos, amount: Math.max(0, pos.amount - parseFloat(tradeAmount)) }
              : pos
          ));
        }
      }
    } catch (error) {
      console.error('Trade execution failed:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchAIRecommendation();
    
    // Simulate real-time price updates
    const interval = setInterval(() => {
      setCurrentSignal(prev => ({
        ...prev,
        price: prev.price + (Math.random() - 0.5) * 0.1,
        timestamp: new Date().toISOString()
      }));

      setPositions(prev => prev.map(pos => {
        const newPrice = pos.currentPrice + (Math.random() - 0.5) * 0.05;
        const pnl = (newPrice - pos.entryPrice) * pos.amount;
        const pnlPercent = ((newPrice - pos.entryPrice) / pos.entryPrice) * 100;
        
        return {
          ...pos,
          currentPrice: newPrice,
          pnl,
          pnlPercent
        };
      }));
    }, 3000);

    return () => clearInterval(interval);
  }, []);

  const getSignalColor = (action: string) => {
    switch (action) {
      case 'buy': return 'text-green-500';
      case 'sell': return 'text-red-500';
      case 'hold': return 'text-yellow-500';
      default: return 'text-gray-500';
    }
  };

  const getSignalIcon = (action: string) => {
    switch (action) {
      case 'buy': return '📈';
      case 'sell': return '📉';
      case 'hold': return '⏸️';
      default: return '📊';
    }
  };

  return (
    <div className="space-y-6">
      {/* Current Signal */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-2xl font-bold mb-4">🎯 Current Trading Signal</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="text-center">
            <div className="text-6xl mb-2">{getSignalIcon(currentSignal.action)}</div>
            <div className={`text-2xl font-bold ${getSignalColor(currentSignal.action)}`}>
              {currentSignal.action.toUpperCase()}
            </div>
            <div className="text-gray-400 text-sm mt-1">
              Confidence: {(currentSignal.confidence * 100).toFixed(1)}%
            </div>
          </div>
          
          <div className="md:col-span-2">
            <div className="space-y-3">
              <div>
                <span className="text-gray-400">Current Price:</span>
                <span className="ml-2 text-xl font-bold">${currentSignal.price.toFixed(2)}</span>
              </div>
              <div>
                <span className="text-gray-400">Reason:</span>
                <p className="mt-1 text-sm">{currentSignal.reason}</p>
              </div>
              <div>
                <span className="text-gray-400">Last Updated:</span>
                <span className="ml-2 text-sm">
                  {new Date(currentSignal.timestamp).toLocaleTimeString()}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* AI Recommendation */}
      {aiRecommendation && (
        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-xl font-bold">🧮 AI Recommendation</h3>
            <button
              onClick={fetchAIRecommendation}
              disabled={isLoading}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors disabled:opacity-50"
            >
              {isLoading ? '🔄 Loading...' : '🔄 Refresh'}
            </button>
          </div>
          
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <p className="text-gray-400 text-sm">Position Size</p>
              <p className="text-lg font-bold">{aiRecommendation.position_size.toFixed(3)} SOL</p>
            </div>
            <div className="text-center">
              <p className="text-gray-400 text-sm">Kelly Fraction</p>
              <p className="text-lg font-bold">{(aiRecommendation.kelly_fraction * 100).toFixed(1)}%</p>
            </div>
            <div className="text-center">
              <p className="text-gray-400 text-sm">Risk Score</p>
              <p className="text-lg font-bold">{aiRecommendation.risk_score.toFixed(2)}</p>
            </div>
            <div className="text-center">
              <p className="text-gray-400 text-sm">Confidence</p>
              <p className="text-lg font-bold text-green-500">{(aiRecommendation.confidence * 100).toFixed(1)}%</p>
            </div>
          </div>
        </div>
      )}

      {/* Trading Controls */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-xl font-bold mb-4">💰 Manual Trading</h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-2">Token</label>
              <select
                value={selectedToken}
                onChange={(e) => setSelectedToken(e.target.value)}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:border-blue-500"
              >
                <option value="SOL">SOL</option>
                <option value="USDC">USDC</option>
                <option value="USDT">USDT</option>
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-2">Amount</label>
              <input
                type="number"
                value={tradeAmount}
                onChange={(e) => setTradeAmount(e.target.value)}
                step="0.1"
                min="0"
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:border-blue-500"
                placeholder="Enter amount"
              />
            </div>
          </div>
          
          <div className="flex flex-col justify-center space-y-3">
            <button
              onClick={() => executeTrade('buy')}
              disabled={isLoading}
              className="w-full py-3 bg-green-600 hover:bg-green-700 text-white rounded-lg font-bold text-lg transition-colors disabled:opacity-50"
            >
              📈 BUY {selectedToken}
            </button>
            
            <button
              onClick={() => executeTrade('hold')}
              disabled={isLoading}
              className="w-full py-3 bg-yellow-600 hover:bg-yellow-700 text-white rounded-lg font-bold text-lg transition-colors disabled:opacity-50"
            >
              ⏸️ HOLD
            </button>
            
            <button
              onClick={() => executeTrade('sell')}
              disabled={isLoading}
              className="w-full py-3 bg-red-600 hover:bg-red-700 text-white rounded-lg font-bold text-lg transition-colors disabled:opacity-50"
            >
              📉 SELL {selectedToken}
            </button>
          </div>
        </div>
      </div>

      {/* Current Positions */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-xl font-bold mb-4">📊 Current Positions</h3>
        
        <div className="space-y-3">
          {positions.map((position, index) => (
            <div key={index} className="bg-gray-700 rounded-lg p-4">
              <div className="grid grid-cols-2 md:grid-cols-6 gap-4 items-center">
                <div>
                  <p className="font-bold text-lg">{position.token}</p>
                  <p className="text-gray-400 text-sm">Token</p>
                </div>
                
                <div>
                  <p className="font-semibold">{position.amount.toFixed(3)}</p>
                  <p className="text-gray-400 text-sm">Amount</p>
                </div>
                
                <div>
                  <p className="font-semibold">${position.entryPrice.toFixed(2)}</p>
                  <p className="text-gray-400 text-sm">Entry Price</p>
                </div>
                
                <div>
                  <p className="font-semibold">${position.currentPrice.toFixed(2)}</p>
                  <p className="text-gray-400 text-sm">Current Price</p>
                </div>
                
                <div>
                  <p className={`font-semibold ${position.pnl >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                    ${position.pnl.toFixed(2)}
                  </p>
                  <p className="text-gray-400 text-sm">P&L</p>
                </div>
                
                <div>
                  <p className={`font-semibold ${position.pnlPercent >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                    {position.pnlPercent >= 0 ? '+' : ''}{position.pnlPercent.toFixed(2)}%
                  </p>
                  <p className="text-gray-400 text-sm">P&L %</p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default TradingPanel;
