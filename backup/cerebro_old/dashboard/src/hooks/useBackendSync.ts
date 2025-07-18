import { useEffect, useCallback, useRef } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { useWalletStore } from '@/stores/walletStore';
import { webSocketService } from '@/services/websocket';
import { apiClient } from '@/services/api';

interface UseBackendSyncOptions {
  enableAutoSync?: boolean;
  syncInterval?: number; // in milliseconds
  enableWebSocket?: boolean;
}

export const useBackendSync = (options: UseBackendSyncOptions = {}) => {
  const {
    enableAutoSync = true,
    syncInterval = 5 * 60 * 1000, // 5 minutes
    enableWebSocket = true,
  } = options;

  const { connected: walletConnected, publicKey } = useWallet();
  const {
    connected: backendConnected,
    syncWithBackend,
    syncInProgress,
    lastSync,
    setError,
  } = useWalletStore();

  const syncIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const lastSyncRef = useRef<Date | null>(null);

  // Manual sync function
  const manualSync = useCallback(async () => {
    if (!backendConnected || syncInProgress) {
      return;
    }

    try {
      await syncWithBackend();
      lastSyncRef.current = new Date();
      console.log('Manual sync completed');
    } catch (error) {
      console.error('Manual sync failed:', error);
      setError(error instanceof Error ? error.message : 'Sync failed');
    }
  }, [backendConnected, syncInProgress, syncWithBackend, setError]);

  // Auto sync setup
  useEffect(() => {
    if (!enableAutoSync || !backendConnected || !walletConnected) {
      if (syncIntervalRef.current) {
        clearInterval(syncIntervalRef.current);
        syncIntervalRef.current = null;
      }
      return;
    }

    // Initial sync if needed
    const now = new Date();
    const timeSinceLastSync = lastSync ? now.getTime() - new Date(lastSync).getTime() : Infinity;
    
    if (timeSinceLastSync > syncInterval) {
      manualSync();
    }

    // Set up periodic sync
    syncIntervalRef.current = setInterval(() => {
      if (backendConnected && !syncInProgress) {
        manualSync();
      }
    }, syncInterval);

    return () => {
      if (syncIntervalRef.current) {
        clearInterval(syncIntervalRef.current);
        syncIntervalRef.current = null;
      }
    };
  }, [
    enableAutoSync,
    backendConnected,
    walletConnected,
    syncInterval,
    lastSync,
    manualSync,
    syncInProgress,
  ]);

  // WebSocket event handlers
  useEffect(() => {
    if (!enableWebSocket || !backendConnected) {
      return;
    }

    // Handle wallet balance updates
    const unsubscribeBalance = webSocketService.onWalletBalanceUpdate((data) => {
      console.log('Received balance update:', data);
      // Balance is automatically updated in the store via WebSocket service
    });

    // Handle portfolio updates
    const unsubscribePortfolio = webSocketService.onPortfolioUpdate((data) => {
      console.log('Received portfolio update:', data);
      // Portfolio is automatically updated in the store via WebSocket service
    });

    // Handle strategy status changes
    const unsubscribeStrategy = webSocketService.onStrategyStatusChange((data) => {
      console.log('Received strategy update:', data);
      // Could trigger a strategy refresh here if needed
    });

    // Handle trading execution events
    const unsubscribeTrading = webSocketService.onTradingExecution((data) => {
      console.log('Received trading execution:', data);
      // Could trigger a portfolio sync after successful trades
      if (data.status === 'success') {
        setTimeout(() => manualSync(), 2000); // Sync after 2 seconds
      }
    });

    // Handle connection events
    const unsubscribeConnectionOpen = webSocketService.on('connection.opened', () => {
      console.log('WebSocket connected, triggering sync');
      manualSync();
    });

    const unsubscribeConnectionError = webSocketService.on('connection.error', (error) => {
      console.error('WebSocket connection error:', error);
      setError('Real-time connection lost');
    });

    return () => {
      unsubscribeBalance();
      unsubscribePortfolio();
      unsubscribeStrategy();
      unsubscribeTrading();
      unsubscribeConnectionOpen();
      unsubscribeConnectionError();
    };
  }, [enableWebSocket, backendConnected, manualSync, setError]);

  // Sync on wallet address change
  useEffect(() => {
    if (walletConnected && backendConnected && publicKey) {
      console.log('Wallet address changed, triggering sync');
      manualSync();
    }
  }, [walletConnected, backendConnected, publicKey, manualSync]);

  // Sync on page visibility change (when user returns to tab)
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (!document.hidden && backendConnected && walletConnected) {
        const now = new Date();
        const timeSinceLastSync = lastSyncRef.current 
          ? now.getTime() - lastSyncRef.current.getTime() 
          : Infinity;

        // Sync if it's been more than 1 minute since last sync
        if (timeSinceLastSync > 60 * 1000) {
          console.log('Page became visible, triggering sync');
          manualSync();
        }
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, [backendConnected, walletConnected, manualSync]);

  // Network status monitoring
  useEffect(() => {
    const handleOnline = () => {
      if (backendConnected && walletConnected) {
        console.log('Network came back online, triggering sync');
        manualSync();
      }
    };

    const handleOffline = () => {
      console.log('Network went offline');
      setError('Network connection lost');
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, [backendConnected, walletConnected, manualSync, setError]);

  return {
    manualSync,
    syncInProgress,
    lastSync,
    isAutoSyncEnabled: enableAutoSync && backendConnected && walletConnected,
    isWebSocketEnabled: enableWebSocket,
    wsConnected: webSocketService.isConnected(),
  };
};
