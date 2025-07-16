import React, { useState } from 'react';

interface Strategy {
  id: string;
  name: string;
  description: string;
  isActive: boolean;
  profit: number;
  trades: number;
  successRate: number;
  icon: React.ReactNode;
}

const StrategyManager: React.FC = () => {
  const [strategies, setStrategies] = useState<Strategy[]>([
    {
      id: 'market_making',
      name: 'Market Making',
      description: 'Provides liquidity by placing buy/sell orders around current price',
      isActive: true,
      profit: 0.045,
      trades: 23,
      successRate: 87.5,
      icon: <span className="text-2xl">📈</span>
    },
    {
      id: 'arbitrage',
      name: 'DEX Arbitrage',
      description: 'Exploits price differences between Raydium, Orca, and Jupiter',
      isActive: true,
      profit: 0.032,
      trades: 12,
      successRate: 91.2,
      icon: <span className="text-2xl">⚡</span>
    },
    {
      id: 'mev_basic',
      name: 'MEV Basic',
      description: 'Basic MEV strategies (sandwich attacks disabled)',
      isActive: false,
      profit: 0.0,
      trades: 0,
      successRate: 0,
      icon: <span className="text-2xl">🎯</span>
    }
  ]);

  const toggleStrategy = (id: string) => {
    setStrategies(prev => 
      prev.map(strategy => 
        strategy.id === id 
          ? { ...strategy, isActive: !strategy.isActive }
          : strategy
      )
    );
  };

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      <h2 className="text-2xl font-bold mb-6">Strategy Management</h2>
      
      <div className="space-y-4">
        {strategies.map((strategy) => (
          <div key={strategy.id} className="bg-gray-700 rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center space-x-3">
                <div className={`p-2 rounded-lg ${strategy.isActive ? 'bg-green-600' : 'bg-gray-600'}`}>
                  {strategy.icon}
                </div>
                <div>
                  <h3 className="text-lg font-semibold">{strategy.name}</h3>
                  <p className="text-gray-400 text-sm">{strategy.description}</p>
                </div>
              </div>
              
              <div className="flex items-center space-x-2">
                <button
                  onClick={() => toggleStrategy(strategy.id)}
                  className={`flex items-center space-x-2 px-4 py-2 rounded-lg font-medium transition-colors ${
                    strategy.isActive
                      ? 'bg-red-600 hover:bg-red-700 text-white'
                      : 'bg-green-600 hover:bg-green-700 text-white'
                  }`}
                >
                  {strategy.isActive ? (
                    <>
                      <span>⏸️</span>
                      <span>Stop</span>
                    </>
                  ) : (
                    <>
                      <span>▶️</span>
                      <span>Start</span>
                    </>
                  )}
                </button>

                <button className="p-2 bg-gray-600 hover:bg-gray-500 rounded-lg transition-colors">
                  <span>⚙️</span>
                </button>
              </div>
            </div>
            
            {/* Strategy Stats */}
            <div className="grid grid-cols-3 gap-4 mt-4">
              <div className="text-center">
                <p className="text-gray-400 text-sm">Profit</p>
                <p className={`font-semibold ${strategy.profit >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                  {strategy.profit >= 0 ? '+' : ''}{(strategy.profit * 100).toFixed(2)}%
                </p>
              </div>
              
              <div className="text-center">
                <p className="text-gray-400 text-sm">Trades</p>
                <p className="font-semibold">{strategy.trades}</p>
              </div>
              
              <div className="text-center">
                <p className="text-gray-400 text-sm">Success Rate</p>
                <p className="font-semibold text-blue-500">{strategy.successRate.toFixed(1)}%</p>
              </div>
            </div>
            
            {/* Status Indicator */}
            <div className="mt-3 flex items-center space-x-2">
              <div className={`w-2 h-2 rounded-full ${strategy.isActive ? 'bg-green-500' : 'bg-gray-500'}`}></div>
              <span className="text-sm text-gray-400">
                {strategy.isActive ? 'Active' : 'Inactive'}
              </span>
            </div>
          </div>
        ))}
      </div>
      
      {/* Global Controls */}
      <div className="mt-6 pt-6 border-t border-gray-600">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold">Global Controls</h3>
            <p className="text-gray-400 text-sm">Emergency controls for all strategies</p>
          </div>
          
          <div className="flex space-x-3">
            <button className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium transition-colors">
              Emergency Stop
            </button>
            <button className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors">
              Restart All
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default StrategyManager;
