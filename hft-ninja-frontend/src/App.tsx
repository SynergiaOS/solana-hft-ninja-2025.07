import React, { useState } from 'react';
import Dashboard from './components/Dashboard';
import TradingPanel from './components/TradingPanel';
import TransactionMonitor from './components/TransactionMonitor';
import StrategyManager from './components/StrategyManager';
import CerberusPanel from './components/CerberusPanel';
import './App.css';

type TabType = 'dashboard' | 'trading' | 'transactions' | 'strategies' | 'cerberus';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('dashboard');

  const renderContent = () => {
    switch (activeTab) {
      case 'dashboard':
        return <Dashboard />;
      case 'trading':
        return <TradingPanel />;
      case 'transactions':
        return <TransactionMonitor />;
      case 'strategies':
        return <StrategyManager />;
      case 'cerberus':
        return <CerberusPanel />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {/* Navigation */}
      <nav className="bg-gray-800 border-b border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center">
              <h1 className="text-xl font-bold">🥷 Solana HFT Ninja 2025.07</h1>
            </div>
            <div className="flex space-x-4">
              {[
                { id: 'dashboard', label: '📊 Dashboard', icon: '📊' },
                { id: 'trading', label: '💹 Trading', icon: '💹' },
                { id: 'transactions', label: '📋 Transactions', icon: '📋' },
                { id: 'strategies', label: '🎯 Strategies', icon: '🎯' },
                { id: 'cerberus', label: '🧠 Cerberus', icon: '🧠' },
              ].map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as TabType)}
                  className={`px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                    activeTab === tab.id
                      ? 'bg-gray-700 text-white'
                      : 'text-gray-300 hover:bg-gray-700 hover:text-white'
                  }`}
                >
                  {tab.label}
                </button>
              ))}
            </div>
          </div>
        </div>
      </nav>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        <div className="px-4 py-6 sm:px-0">
          {renderContent()}
        </div>
      </main>
    </div>
  );
}

export default App;
