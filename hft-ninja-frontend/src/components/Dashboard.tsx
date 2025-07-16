import React, { useState, useEffect } from 'react';
// import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar } from 'recharts';
// import axios from 'axios';
import StrategyManager from './StrategyManager';
import TransactionMonitor from './TransactionMonitor';

interface SystemMetrics {
  status: 'online' | 'offline' | 'warning';
  balance: number;
  totalTrades: number;
  profitLoss: number;
  latency: number;
  uptime: string;
  activeStrategies: string[];
  mempool: {
    transactions: number;
    processed: number;
  };
}

interface TradeData {
  timestamp: string;
  price: number;
  volume: number;
  profit: number;
}

const Dashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'dashboard' | 'strategies' | 'transactions' | 'settings'>('dashboard');
  const [metrics, setMetrics] = useState<SystemMetrics>({
    status: 'offline',
    balance: 0,
    totalTrades: 0,
    profitLoss: 0,
    latency: 0,
    uptime: '0h 0m',
    activeStrategies: [],
    mempool: { transactions: 0, processed: 0 }
  });

  const [tradeData, setTradeData] = useState<TradeData[]>([]);
  const [isConnected, setIsConnected] = useState(false);

  // Mock data for development
  const mockTradeData: TradeData[] = [
    { timestamp: '10:00', price: 23.45, volume: 1250, profit: 0.12 },
    { timestamp: '10:05', price: 23.52, volume: 980, profit: 0.08 },
    { timestamp: '10:10', price: 23.48, volume: 1100, profit: -0.04 },
    { timestamp: '10:15', price: 23.61, volume: 1350, profit: 0.15 },
    { timestamp: '10:20', price: 23.58, volume: 1200, profit: 0.09 },
  ];

  useEffect(() => {
    // Use mock data for now
    setIsConnected(false);
    setMetrics({
      status: 'online',
      balance: 1.25,
      totalTrades: 47,
      profitLoss: 0.087,
      latency: 12,
      uptime: '2h 34m',
      activeStrategies: ['Market Making', 'Arbitrage'],
      mempool: { transactions: 1247, processed: 1198 }
    });
    setTradeData(mockTradeData);

    // Simulate real-time updates
    const interval = setInterval(() => {
      setMetrics(prev => ({
        ...prev,
        totalTrades: prev.totalTrades + Math.floor(Math.random() * 3),
        profitLoss: prev.profitLoss + (Math.random() - 0.5) * 0.01,
        latency: 10 + Math.floor(Math.random() * 10)
      }));
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online': return 'text-green-500';
      case 'warning': return 'text-yellow-500';
      case 'offline': return 'text-red-500';
      default: return 'text-gray-500';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'online': return <span className="w-5 h-5">✅</span>;
      case 'warning': return <span className="w-5 h-5">⚠️</span>;
      case 'offline': return <span className="w-5 h-5">❌</span>;
      default: return <span className="w-5 h-5">📊</span>;
    }
  };

  const tabs = [
    { id: 'dashboard', name: 'Dashboard', icon: <span className="w-5 h-5">📊</span> },
    { id: 'strategies', name: 'Strategies', icon: <span className="w-5 h-5">📈</span> },
    { id: 'transactions', name: 'Transactions', icon: <span className="w-5 h-5">💰</span> },
    { id: 'settings', name: 'Settings', icon: <span className="w-5 h-5">⚙️</span> },
  ];

  const renderContent = () => {
    switch (activeTab) {
      case 'strategies':
        return <StrategyManager />;
      case 'transactions':
        return <TransactionMonitor />;
      case 'settings':
        return (
          <div className="bg-gray-800 rounded-lg p-6">
            <h2 className="text-2xl font-bold mb-4">Settings</h2>
            <p className="text-gray-400">Settings panel coming soon...</p>
          </div>
        );
      default:
        return renderDashboard();
    }
  };

  const renderDashboard = () => (
    <>
      {/* Key Metrics Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Balance</p>
              <p className="text-2xl font-bold">{metrics.balance.toFixed(3)} SOL</p>
            </div>
            <span className="text-4xl">💰</span>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Total Trades</p>
              <p className="text-2xl font-bold">{metrics.totalTrades}</p>
            </div>
            <span className="text-4xl">📈</span>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">P&L</p>
              <p className={`text-2xl font-bold ${metrics.profitLoss >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                {metrics.profitLoss >= 0 ? '+' : ''}{(metrics.profitLoss * 100).toFixed(2)}%
              </p>
            </div>
            <span className="text-4xl">{metrics.profitLoss >= 0 ? '📈' : '📉'}</span>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">Latency</p>
              <p className="text-2xl font-bold">{metrics.latency}ms</p>
            </div>
            <span className="text-4xl">⚡</span>
          </div>
        </div>
      </div>

      {/* Charts Section - Placeholder */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
        {/* Price Chart Placeholder */}
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-xl font-semibold mb-4">📈 Price Movement</h3>
          <div className="h-64 bg-gray-700 rounded-lg flex items-center justify-center">
            <div className="text-center">
              <div className="text-4xl mb-2">📊</div>
              <p className="text-gray-400">Price Chart</p>
              <p className="text-sm text-gray-500">Charts will be available after installing recharts</p>
            </div>
          </div>
        </div>

        {/* Volume Chart Placeholder */}
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-xl font-semibold mb-4">📊 Trading Volume</h3>
          <div className="h-64 bg-gray-700 rounded-lg flex items-center justify-center">
            <div className="text-center">
              <div className="text-4xl mb-2">📈</div>
              <p className="text-gray-400">Volume Chart</p>
              <p className="text-sm text-gray-500">Charts will be available after installing recharts</p>
            </div>
          </div>
        </div>
      </div>

      {/* System Info */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Active Strategies */}
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-xl font-semibold mb-4">Active Strategies</h3>
          <div className="space-y-3">
            {metrics.activeStrategies.map((strategy, index) => (
              <div key={index} className="flex items-center justify-between p-3 bg-gray-700 rounded-lg">
                <span>{strategy}</span>
                <span className="text-green-500 text-sm">Active</span>
              </div>
            ))}
          </div>
        </div>

        {/* Mempool Stats */}
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-xl font-semibold mb-4">Mempool Statistics</h3>
          <div className="space-y-4">
            <div className="flex justify-between">
              <span className="text-gray-400">Transactions Seen:</span>
              <span className="font-semibold">{metrics.mempool.transactions.toLocaleString()}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Processed:</span>
              <span className="font-semibold">{metrics.mempool.processed.toLocaleString()}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Success Rate:</span>
              <span className="font-semibold text-green-500">
                {((metrics.mempool.processed / metrics.mempool.transactions) * 100).toFixed(1)}%
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Uptime:</span>
              <span className="font-semibold">{metrics.uptime}</span>
            </div>
          </div>
        </div>
      </div>
    </>
  );

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {/* Header */}
      <div className="bg-gray-800 border-b border-gray-700 px-6 py-4">
        <div className="max-w-7xl mx-auto">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold">🥷 Solana HFT Ninja 2025.07</h1>
              <div className="flex items-center space-x-4 mt-1">
                <div className={`flex items-center space-x-2 ${getStatusColor(metrics.status)}`}>
                  {getStatusIcon(metrics.status)}
                  <span className="font-medium text-sm">
                    {metrics.status.charAt(0).toUpperCase() + metrics.status.slice(1)}
                  </span>
                </div>
                <div className="text-gray-400 text-sm">
                  {isConnected ? '🟢 Connected to Backend' : '🔴 Using Mock Data'}
                </div>
              </div>
            </div>

            {/* Navigation Tabs */}
            <div className="flex space-x-1">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as any)}
                  className={`flex items-center space-x-2 px-4 py-2 rounded-lg font-medium transition-colors ${
                    activeTab === tab.id
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-400 hover:text-white hover:bg-gray-700'
                  }`}
                >
                  {tab.icon}
                  <span>{tab.name}</span>
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="p-6">
        <div className="max-w-7xl mx-auto">
          {renderContent()}
        </div>
      </div>
    </div>
  );
};

export default Dashboard;
