import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'react-hot-toast';

// Web3 Components
import WalletProvider from '@/web3/components/WalletProvider';

// Layout Components
import DashboardLayout from '@/components/layout/DashboardLayout';

// Page Components
import OverviewPage from '@/pages/OverviewPage';
import TradingPage from '@/pages/TradingPage';
import FinGPTPage from '@/pages/FinGPTPage';
import StrategiesPage from '@/pages/StrategiesPage';
import AnalyticsPage from '@/pages/AnalyticsPage';
import SystemPage from '@/pages/SystemPage';
import SettingsPage from '@/pages/SettingsPage';

// Hooks
import { useBackendSync } from '@/hooks/useBackendSync';

// Styles
import './index.css';

// Create a client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      cacheTime: 1000 * 60 * 10, // 10 minutes
      retry: 2,
      refetchOnWindowFocus: false,
    },
  },
});

// Backend Sync Component
const BackendSync: React.FC = () => {
  useBackendSync({
    enableAutoSync: import.meta.env.VITE_ENABLE_BACKGROUND_SYNC !== 'false',
    syncInterval: 5 * 60 * 1000, // 5 minutes
    enableWebSocket: import.meta.env.VITE_ENABLE_WEBSOCKET !== 'false',
  });
  return null;
};

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <WalletProvider>
        <BackendSync />
        <div className="min-h-screen bg-[#0B0E1A] text-white">
          <Router>
            <Routes>
              {/* Dashboard Routes */}
              <Route path="/" element={<DashboardLayout />}>
                <Route index element={<Navigate to="/overview" replace />} />
                <Route path="overview" element={<OverviewPage />} />
                <Route path="trading" element={<TradingPage />} />
                <Route path="fingpt" element={<FinGPTPage />} />
                <Route path="strategies" element={<StrategiesPage />} />
                <Route path="analytics" element={<AnalyticsPage />} />
                <Route path="system" element={<SystemPage />} />
                <Route path="settings" element={<SettingsPage />} />
              </Route>

              {/* Fallback Route */}
              <Route path="*" element={<Navigate to="/overview" replace />} />
            </Routes>
          </Router>

          {/* Global Toast Notifications */}
          <Toaster
            position="top-right"
            toastOptions={{
              duration: 4000,
              style: {
                background: '#1A1D29',
                color: '#ffffff',
                border: '1px solid #2A2D3A',
                borderRadius: '12px',
                fontSize: '14px',
                fontWeight: '500',
                boxShadow: '0 10px 25px rgba(0, 0, 0, 0.3)',
              },
              success: {
                iconTheme: {
                  primary: '#10B981',
                  secondary: '#ffffff',
                },
              },
              error: {
                iconTheme: {
                  primary: '#EF4444',
                  secondary: '#ffffff',
                },
              },
              loading: {
                iconTheme: {
                  primary: '#8B5CF6',
                  secondary: '#ffffff',
                },
              },
            }}
          />
        </div>
      </WalletProvider>
    </QueryClientProvider>
  );
}

export default App;
