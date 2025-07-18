import {
  Connection,
  PublicKey,
  Transaction,
  VersionedTransaction,
  TransactionSignature,
  SendOptions,
  Commitment,
  GetProgramAccountsFilter,
  AccountInfo,
  ParsedAccountData,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import { WalletAdapter } from '@solana/wallet-adapter-base';
import { connection } from '@/web3/utils/connection';
import { ParsedTransaction, TransactionDetails, ProgramAccount } from '@/web3/types';

export class SolanaService {
  private connection: Connection;

  constructor(customConnection?: Connection) {
    this.connection = customConnection || connection;
  }

  // Connection methods
  async getConnection(): Promise<Connection> {
    return this.connection;
  }

  async getNetworkVersion(): Promise<any> {
    return await this.connection.getVersion();
  }

  async getSlot(): Promise<number> {
    return await this.connection.getSlot();
  }

  async getBlockHeight(): Promise<number> {
    return await this.connection.getBlockHeight();
  }

  // Account methods
  async getBalance(publicKey: PublicKey): Promise<number> {
    const lamports = await this.connection.getBalance(publicKey);
    return lamports / LAMPORTS_PER_SOL;
  }

  async getAccountInfo(publicKey: PublicKey): Promise<AccountInfo<Buffer> | null> {
    return await this.connection.getAccountInfo(publicKey);
  }

  async getParsedAccountInfo(publicKey: PublicKey): Promise<AccountInfo<ParsedAccountData> | null> {
    const response = await this.connection.getParsedAccountInfo(publicKey);
    return response.value;
  }

  async getMultipleAccounts(publicKeys: PublicKey[]): Promise<(AccountInfo<Buffer> | null)[]> {
    const response = await this.connection.getMultipleAccountsInfo(publicKeys);
    return response;
  }

  // Transaction methods
  async sendTransaction(
    transaction: Transaction | VersionedTransaction,
    wallet: WalletAdapter,
    options?: SendOptions
  ): Promise<TransactionSignature> {
    if (!wallet.publicKey) {
      throw new Error('Wallet not connected');
    }

    // Sign the transaction
    const signedTransaction = await wallet.signTransaction(transaction);

    // Send the transaction
    const signature = await this.connection.sendRawTransaction(
      signedTransaction.serialize(),
      options
    );

    return signature;
  }

  async confirmTransaction(
    signature: TransactionSignature,
    commitment: Commitment = 'confirmed'
  ): Promise<boolean> {
    try {
      const confirmation = await this.connection.confirmTransaction(signature, commitment);
      return !confirmation.value.err;
    } catch (error) {
      console.error('Transaction confirmation error:', error);
      return false;
    }
  }

  async getTransaction(
    signature: TransactionSignature,
    commitment: Commitment = 'confirmed'
  ): Promise<TransactionDetails | null> {
    try {
      const transaction = await this.connection.getTransaction(signature, {
        commitment,
        maxSupportedTransactionVersion: 0,
      });

      if (!transaction) return null;

      return {
        signature,
        slot: transaction.slot,
        blockTime: transaction.blockTime,
        confirmationStatus: commitment,
        err: transaction.meta?.err || null,
        fee: transaction.meta?.fee || 0,
        preBalances: transaction.meta?.preBalances || [],
        postBalances: transaction.meta?.postBalances || [],
        preTokenBalances: transaction.meta?.preTokenBalances || [],
        postTokenBalances: transaction.meta?.postTokenBalances || [],
        logMessages: transaction.meta?.logMessages || [],
        computeUnitsConsumed: transaction.meta?.computeUnitsConsumed,
      };
    } catch (error) {
      console.error('Error fetching transaction:', error);
      return null;
    }
  }

  async getTransactionHistory(
    publicKey: PublicKey,
    limit: number = 10
  ): Promise<ParsedTransaction[]> {
    try {
      const signatures = await this.connection.getSignaturesForAddress(publicKey, { limit });
      const transactions: ParsedTransaction[] = [];

      for (const signatureInfo of signatures) {
        const transaction = await this.getTransaction(signatureInfo.signature);
        if (transaction) {
          const parsed = this.parseTransaction(transaction);
          transactions.push(parsed);
        }
      }

      return transactions;
    } catch (error) {
      console.error('Error fetching transaction history:', error);
      return [];
    }
  }

  private parseTransaction(transaction: TransactionDetails): ParsedTransaction {
    // Basic transaction parsing - can be enhanced based on needs
    return {
      signature: transaction.signature,
      blockTime: transaction.blockTime || 0,
      slot: transaction.slot,
      fee: transaction.fee,
      status: transaction.err ? 'failed' : 'success',
      type: 'unknown', // TODO: Implement transaction type detection
      instructions: [], // TODO: Parse instructions
    };
  }

  // Program methods
  async getProgramAccounts(
    programId: PublicKey,
    filters?: GetProgramAccountsFilter[]
  ): Promise<ProgramAccount[]> {
    try {
      const accounts = await this.connection.getProgramAccounts(programId, {
        filters: filters || [],
      });

      return accounts.map(({ pubkey, account }) => ({
        pubkey,
        account,
      }));
    } catch (error) {
      console.error('Error fetching program accounts:', error);
      return [];
    }
  }

  // Subscription methods
  onAccountChange(
    publicKey: PublicKey,
    callback: (accountInfo: AccountInfo<Buffer>, context: any) => void,
    commitment: Commitment = 'confirmed'
  ): number {
    return this.connection.onAccountChange(publicKey, callback, commitment);
  }

  onProgramAccountChange(
    programId: PublicKey,
    callback: (keyedAccountInfo: any, context: any) => void,
    commitment: Commitment = 'confirmed',
    filters?: GetProgramAccountsFilter[]
  ): number {
    return this.connection.onProgramAccountChange(
      programId,
      callback,
      commitment,
      filters
    );
  }

  onSignature(
    signature: TransactionSignature,
    callback: (signatureResult: any, context: any) => void,
    commitment: Commitment = 'confirmed'
  ): number {
    return this.connection.onSignature(signature, callback, commitment);
  }

  removeAccountChangeListener(id: number): Promise<void> {
    return this.connection.removeAccountChangeListener(id);
  }

  removeProgramAccountChangeListener(id: number): Promise<void> {
    return this.connection.removeProgramAccountChangeListener(id);
  }

  removeSignatureListener(id: number): Promise<void> {
    return this.connection.removeSignatureListener(id);
  }

  // Utility methods
  async airdrop(publicKey: PublicKey, lamports: number): Promise<TransactionSignature> {
    return await this.connection.requestAirdrop(publicKey, lamports);
  }

  async getMinimumBalanceForRentExemption(dataLength: number): Promise<number> {
    return await this.connection.getMinimumBalanceForRentExemption(dataLength);
  }

  async getRecentBlockhash(): Promise<string> {
    const { blockhash } = await this.connection.getLatestBlockhash();
    return blockhash;
  }

  async simulateTransaction(
    transaction: Transaction | VersionedTransaction
  ): Promise<any> {
    return await this.connection.simulateTransaction(transaction);
  }

  // Health check
  async isHealthy(): Promise<boolean> {
    try {
      await this.connection.getSlot();
      return true;
    } catch (error) {
      console.error('Connection health check failed:', error);
      return false;
    }
  }
}

// Export singleton instance
export const solanaService = new SolanaService();
