import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { PublicKey } from '@solana/web3.js';
import { WalletAdapter } from '@solana/wallet-adapter-base';
import { apiClient, WalletProfile, PortfolioData } from '@/services/api';
import { webSocketService } from '@/services/websocket';

// Store State Interface
interface WalletState {
  // Connection State
  connected: boolean;
  connecting: boolean;
  autoConnecting: boolean;
  publicKey: PublicKey | null;
  wallet: WalletAdapter | null;
  
  // Profile Data
  profile: WalletProfile | null;
  portfolio: PortfolioData | null;
  
  // Sync State
  lastSync: Date | null;
  syncInProgress: boolean;
  
  // Error State
  error: string | null;
  
  // WebSocket State
  wsConnected: boolean;
  
  // Actions
  setWallet: (wallet: WalletAdapter | null, publicKey: PublicKey | null) => void;
  setConnected: (connected: boolean) => void;
  setConnecting: (connecting: boolean) => void;
  setAutoConnecting: (autoConnecting: boolean) => void;
  setProfile: (profile: WalletProfile | null) => void;
  setPortfolio: (portfolio: PortfolioData | null) => void;
  setError: (error: string | null) => void;
  setWSConnected: (connected: boolean) => void;
  
  // Async Actions
  connectToBackend: () => Promise<void>;
  disconnectFromBackend: () => Promise<void>;
  syncWithBackend: () => Promise<void>;
  autoConnect: () => Promise<void>;
  
  // Utility Actions
  clearState: () => void;
  updateBalance: (balance: number) => void;
  updatePortfolio: (portfolio: Partial<PortfolioData>) => void;
}

// Create Zustand Store
export const useWalletStore = create<WalletState>()(
  persist(
    (set, get) => ({
      // Initial State
      connected: false,
      connecting: false,
      autoConnecting: false,
      publicKey: null,
      wallet: null,
      profile: null,
      portfolio: null,
      lastSync: null,
      syncInProgress: false,
      error: null,
      wsConnected: false,

      // Basic Setters
      setWallet: (wallet, publicKey) => set({ wallet, publicKey }),
      setConnected: (connected) => set({ connected }),
      setConnecting: (connecting) => set({ connecting }),
      setAutoConnecting: (autoConnecting) => set({ autoConnecting }),
      setProfile: (profile) => set({ profile }),
      setPortfolio: (portfolio) => set({ portfolio }),
      setError: (error) => set({ error }),
      setWSConnected: (wsConnected) => set({ wsConnected }),

      // Connect to Backend
      connectToBackend: async () => {
        const { wallet, publicKey } = get();
        
        if (!wallet || !publicKey) {
          throw new Error('Wallet not connected');
        }

        set({ connecting: true, error: null });

        try {
          // Create authentication message
          const message = `Cerebro Dashboard Authentication\nTimestamp: ${Date.now()}`;
          const messageBytes = new TextEncoder().encode(message);
          
          // Sign message with wallet
          const signature = await wallet.signMessage(messageBytes);
          const signatureBase58 = Buffer.from(signature).toString('base64');

          // Connect to backend
          const response = await apiClient.connectWallet(
            publicKey.toString(),
            signatureBase58,
            message
          );

          // Update state
          set({
            connected: true,
            connecting: false,
            profile: response.profile,
            lastSync: new Date(),
            error: null,
          });

          // Start WebSocket connection
          if (!webSocketService.isConnected()) {
            webSocketService.connect();
          }

          console.log('Successfully connected to backend');
        } catch (error) {
          console.error('Backend connection failed:', error);
          set({
            connecting: false,
            error: error instanceof Error ? error.message : 'Connection failed',
          });
          throw error;
        }
      },

      // Disconnect from Backend
      disconnectFromBackend: async () => {
        try {
          await apiClient.disconnectWallet();
          webSocketService.disconnect();
          
          set({
            connected: false,
            profile: null,
            portfolio: null,
            lastSync: null,
            error: null,
            wsConnected: false,
          });

          console.log('Disconnected from backend');
        } catch (error) {
          console.error('Backend disconnection failed:', error);
          // Still clear local state even if backend call fails
          get().clearState();
        }
      },

      // Sync with Backend
      syncWithBackend: async () => {
        const { connected } = get();
        
        if (!connected) {
          console.warn('Not connected to backend, skipping sync');
          return;
        }

        set({ syncInProgress: true, error: null });

        try {
          // Sync wallet profile
          const profile = await apiClient.syncWallet();
          
          // Get portfolio data
          const portfolio = await apiClient.getPortfolio();

          set({
            profile,
            portfolio,
            lastSync: new Date(),
            syncInProgress: false,
          });

          console.log('Successfully synced with backend');
        } catch (error) {
          console.error('Backend sync failed:', error);
          set({
            syncInProgress: false,
            error: error instanceof Error ? error.message : 'Sync failed',
          });
        }
      },

      // Auto Connect
      autoConnect: async () => {
        const storedAddress = localStorage.getItem('cerebro_wallet_address');
        const authToken = localStorage.getItem('cerebro_auth_token');
        
        if (!storedAddress || !authToken) {
          console.log('No stored wallet or auth token found');
          return;
        }

        set({ autoConnecting: true, error: null });

        try {
          // Try to get profile with existing token
          const profile = await apiClient.getWalletProfile();
          const portfolio = await apiClient.getPortfolio();

          set({
            connected: true,
            autoConnecting: false,
            profile,
            portfolio,
            lastSync: new Date(),
          });

          // Start WebSocket connection
          if (!webSocketService.isConnected()) {
            webSocketService.connect();
          }

          console.log('Auto-connected to backend');
        } catch (error) {
          console.error('Auto-connect failed:', error);
          
          // Clear invalid tokens
          apiClient.clearAuthToken();
          localStorage.removeItem('cerebro_wallet_address');
          
          set({
            autoConnecting: false,
            error: 'Auto-connect failed, please reconnect manually',
          });
        }
      },

      // Clear State
      clearState: () => set({
        connected: false,
        connecting: false,
        autoConnecting: false,
        publicKey: null,
        wallet: null,
        profile: null,
        portfolio: null,
        lastSync: null,
        syncInProgress: false,
        error: null,
        wsConnected: false,
      }),

      // Update Balance
      updateBalance: (balance) => {
        const { profile } = get();
        if (profile) {
          set({
            profile: { ...profile, balance },
          });
        }
      },

      // Update Portfolio
      updatePortfolio: (portfolioUpdate) => {
        const { portfolio } = get();
        if (portfolio) {
          set({
            portfolio: { ...portfolio, ...portfolioUpdate },
          });
        }
      },
    }),
    {
      name: 'cerebro-wallet-store',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        // Only persist these fields
        lastSync: state.lastSync,
        profile: state.profile,
        portfolio: state.portfolio,
      }),
    }
  )
);

// WebSocket Event Listeners
webSocketService.onWalletBalanceUpdate((data) => {
  const store = useWalletStore.getState();
  store.updateBalance(data.balance);
});

webSocketService.onPortfolioUpdate((data) => {
  const store = useWalletStore.getState();
  store.updatePortfolio({
    totalValue: data.totalValue,
    tokenBalances: data.tokens,
  });
});

webSocketService.on('connection.opened', () => {
  useWalletStore.getState().setWSConnected(true);
});

webSocketService.on('connection.closed', () => {
  useWalletStore.getState().setWSConnected(false);
});

webSocketService.on('connection.error', () => {
  useWalletStore.getState().setWSConnected(false);
});

// Auto-sync every 5 minutes
setInterval(() => {
  const store = useWalletStore.getState();
  if (store.connected && !store.syncInProgress) {
    store.syncWithBackend();
  }
}, 5 * 60 * 1000);
