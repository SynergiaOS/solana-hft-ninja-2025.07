import React, { FC, ReactNode, useMemo, useEffect } from 'react';
import {
  ConnectionProvider,
  WalletProvider as SolanaWalletProvider,
  useWallet,
} from '@solana/wallet-adapter-react';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
} from '@solana/wallet-adapter-wallets';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import { clusterApiUrl } from '@solana/web3.js';
import { useWalletStore } from '@/stores/walletStore';

// Import wallet adapter CSS
import '@solana/wallet-adapter-react-ui/styles.css';

interface WalletProviderProps {
  children: ReactNode;
  network?: WalletAdapterNetwork;
  endpoint?: string;
}

// Wallet Integration Component
const WalletIntegration: FC = () => {
  const { wallet, publicKey, connected, connecting, disconnect } = useWallet();
  const {
    setWallet,
    setConnected,
    setConnecting,
    connectToBackend,
    disconnectFromBackend,
    autoConnect,
    autoConnecting,
  } = useWalletStore();

  // Sync wallet state with store
  useEffect(() => {
    setWallet(wallet, publicKey);
  }, [wallet, publicKey, setWallet]);

  useEffect(() => {
    setConnected(connected);
  }, [connected, setConnected]);

  useEffect(() => {
    setConnecting(connecting);
  }, [connecting, setConnecting]);

  // Auto-connect on app start
  useEffect(() => {
    if (!connected && !connecting && !autoConnecting) {
      autoConnect();
    }
  }, [connected, connecting, autoConnecting, autoConnect]);

  // Connect to backend when wallet connects
  useEffect(() => {
    if (connected && wallet && publicKey && !useWalletStore.getState().connected) {
      connectToBackend().catch(console.error);
    }
  }, [connected, wallet, publicKey, connectToBackend]);

  // Disconnect from backend when wallet disconnects
  useEffect(() => {
    if (!connected && useWalletStore.getState().connected) {
      disconnectFromBackend().catch(console.error);
    }
  }, [connected, disconnectFromBackend]);

  return null;
};

const WalletProvider: FC<WalletProviderProps> = ({
  children,
  network = WalletAdapterNetwork.Devnet,
  endpoint,
}) => {
  // Configure the RPC endpoint
  const rpcEndpoint = useMemo(() => {
    if (endpoint) return endpoint;

    switch (network) {
      case WalletAdapterNetwork.Mainnet:
        return process.env.REACT_APP_SOLANA_RPC_URL || clusterApiUrl('mainnet-beta');
      case WalletAdapterNetwork.Testnet:
        return clusterApiUrl('testnet');
      case WalletAdapterNetwork.Devnet:
      default:
        return clusterApiUrl('devnet');
    }
  }, [network, endpoint]);

  // Configure supported wallets
  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SolflareWalletAdapter({ network }),
      // Add more wallet adapters as needed
    ],
    [network]
  );

  return (
    <ConnectionProvider endpoint={rpcEndpoint}>
      <SolanaWalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <WalletIntegration />
          {children}
        </WalletModalProvider>
      </SolanaWalletProvider>
    </ConnectionProvider>
  );
};

export default WalletProvider;
