import React, { useState, useEffect } from 'react';

interface Transaction {
  id: string;
  timestamp: string;
  type: 'buy' | 'sell' | 'arbitrage' | 'market_making';
  token: string;
  amount: number;
  price: number;
  profit: number;
  status: 'pending' | 'confirmed' | 'failed';
  signature?: string;
  dex: string;
}

const TransactionMonitor: React.FC = () => {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [filter, setFilter] = useState<'all' | 'buy' | 'sell' | 'arbitrage' | 'market_making'>('all');

  // Mock transaction data
  const mockTransactions: Transaction[] = [
    {
      id: '1',
      timestamp: '10:23:45',
      type: 'buy',
      token: 'SOL/USDC',
      amount: 0.5,
      price: 23.45,
      profit: 0.012,
      status: 'confirmed',
      signature: '5KJp7...9mNx',
      dex: 'Raydium'
    },
    {
      id: '2',
      timestamp: '10:23:12',
      type: 'arbitrage',
      token: 'RAY/SOL',
      amount: 1.2,
      price: 0.089,
      profit: 0.008,
      status: 'confirmed',
      signature: '3Hx9k...7pLm',
      dex: 'Orca → Jupiter'
    },
    {
      id: '3',
      timestamp: '10:22:58',
      type: 'sell',
      token: 'SOL/USDC',
      amount: 0.3,
      price: 23.52,
      profit: -0.003,
      status: 'confirmed',
      signature: '8Ry2m...4nQz',
      dex: 'Raydium'
    },
    {
      id: '4',
      timestamp: '10:22:34',
      type: 'market_making',
      token: 'BONK/SOL',
      amount: 1000,
      price: 0.000012,
      profit: 0.005,
      status: 'pending',
      dex: 'Orca'
    },
    {
      id: '5',
      timestamp: '10:22:01',
      type: 'buy',
      token: 'JUP/USDC',
      amount: 25,
      price: 0.67,
      profit: 0.015,
      status: 'failed',
      dex: 'Jupiter'
    }
  ];

  useEffect(() => {
    setTransactions(mockTransactions);
    
    // Simulate real-time updates
    const interval = setInterval(() => {
      const newTransaction: Transaction = {
        id: Date.now().toString(),
        timestamp: new Date().toLocaleTimeString(),
        type: ['buy', 'sell', 'arbitrage', 'market_making'][Math.floor(Math.random() * 4)] as any,
        token: ['SOL/USDC', 'RAY/SOL', 'JUP/USDC', 'BONK/SOL'][Math.floor(Math.random() * 4)],
        amount: Math.random() * 2,
        price: Math.random() * 50,
        profit: (Math.random() - 0.5) * 0.05,
        status: ['confirmed', 'pending', 'failed'][Math.floor(Math.random() * 3)] as any,
        signature: Math.random().toString(36).substring(2, 8) + '...' + Math.random().toString(36).substring(2, 5),
        dex: ['Raydium', 'Orca', 'Jupiter'][Math.floor(Math.random() * 3)]
      };
      
      setTransactions(prev => [newTransaction, ...prev.slice(0, 19)]); // Keep last 20
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  const filteredTransactions = filter === 'all' 
    ? transactions 
    : transactions.filter(tx => tx.type === filter);

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'buy': return 'text-green-500 bg-green-500/10';
      case 'sell': return 'text-red-500 bg-red-500/10';
      case 'arbitrage': return 'text-blue-500 bg-blue-500/10';
      case 'market_making': return 'text-purple-500 bg-purple-500/10';
      default: return 'text-gray-500 bg-gray-500/10';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'confirmed': return 'text-green-500';
      case 'pending': return 'text-yellow-500';
      case 'failed': return 'text-red-500';
      default: return 'text-gray-500';
    }
  };

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold">Transaction Monitor</h2>
        
        {/* Filter Buttons */}
        <div className="flex space-x-2">
          {['all', 'buy', 'sell', 'arbitrage', 'market_making'].map((filterType) => (
            <button
              key={filterType}
              onClick={() => setFilter(filterType as any)}
              className={`px-3 py-1 rounded-lg text-sm font-medium transition-colors ${
                filter === filterType
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              {filterType.charAt(0).toUpperCase() + filterType.slice(1).replace('_', ' ')}
            </button>
          ))}
        </div>
      </div>

      {/* Transaction List */}
      <div className="space-y-3 max-h-96 overflow-y-auto">
        {filteredTransactions.map((tx) => (
          <div key={tx.id} className="bg-gray-700 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-3">
                {/* Type Badge */}
                <span className={`px-2 py-1 rounded-lg text-xs font-medium ${getTypeColor(tx.type)}`}>
                  {tx.type.replace('_', ' ').toUpperCase()}
                </span>
                
                {/* Transaction Details */}
                <div>
                  <div className="flex items-center space-x-2">
                    <span className="font-semibold">{tx.token}</span>
                    <span className="text-gray-400">•</span>
                    <span className="text-gray-400">{tx.amount.toFixed(3)}</span>
                    <span className="text-gray-400">@</span>
                    <span className="text-gray-400">${tx.price.toFixed(4)}</span>
                  </div>
                  <div className="flex items-center space-x-2 text-sm text-gray-400">
                    <span>🕐</span>
                    <span>{tx.timestamp}</span>
                    <span>•</span>
                    <span>{tx.dex}</span>
                  </div>
                </div>
              </div>

              <div className="flex items-center space-x-4">
                {/* Profit */}
                <div className="text-right">
                  <div className={`flex items-center space-x-1 ${tx.profit >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                    <span>{tx.profit >= 0 ? '📈' : '📉'}</span>
                    <span className="font-semibold">
                      {tx.profit >= 0 ? '+' : ''}{(tx.profit * 100).toFixed(2)}%
                    </span>
                  </div>
                  <div className="text-xs text-gray-400">
                    {tx.profit >= 0 ? '+' : ''}${(tx.profit * tx.amount * tx.price).toFixed(4)}
                  </div>
                </div>

                {/* Status */}
                <div className="text-right">
                  <div className={`font-medium ${getStatusColor(tx.status)}`}>
                    {tx.status.charAt(0).toUpperCase() + tx.status.slice(1)}
                  </div>
                  {tx.signature && (
                    <button className="flex items-center space-x-1 text-xs text-blue-400 hover:text-blue-300">
                      <span>{tx.signature}</span>
                      <span>🔗</span>
                    </button>
                  )}
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Summary Stats */}
      <div className="mt-6 pt-6 border-t border-gray-600">
        <div className="grid grid-cols-4 gap-4">
          <div className="text-center">
            <p className="text-gray-400 text-sm">Total Transactions</p>
            <p className="text-xl font-bold">{transactions.length}</p>
          </div>
          <div className="text-center">
            <p className="text-gray-400 text-sm">Successful</p>
            <p className="text-xl font-bold text-green-500">
              {transactions.filter(tx => tx.status === 'confirmed').length}
            </p>
          </div>
          <div className="text-center">
            <p className="text-gray-400 text-sm">Pending</p>
            <p className="text-xl font-bold text-yellow-500">
              {transactions.filter(tx => tx.status === 'pending').length}
            </p>
          </div>
          <div className="text-center">
            <p className="text-gray-400 text-sm">Failed</p>
            <p className="text-xl font-bold text-red-500">
              {transactions.filter(tx => tx.status === 'failed').length}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default TransactionMonitor;
