// Core Types for Cerebro Dashboard

export interface User {
  id: string;
  name: string;
  email: string;
  avatar?: string;
  role: 'admin' | 'trader' | 'viewer';
  preferences: UserPreferences;
}

export interface UserPreferences {
  theme: 'dark' | 'light';
  notifications: boolean;
  autoRefresh: boolean;
  refreshInterval: number;
  defaultTimeframe: string;
  favoriteStrategies: string[];
}

// Trading Types
export interface TradingMetrics {
  totalPnL: number;
  totalTrades: number;
  winRate: number;
  avgProfit: number;
  avgLoss: number;
  sharpeRatio: number;
  maxDrawdown: number;
  currentBalance: number;
  dailyPnL: number;
  weeklyPnL: number;
  monthlyPnL: number;
}

export interface Strategy {
  id: string;
  name: string;
  type: 'sandwich' | 'arbitrage' | 'liquidation' | 'sniping' | 'market_making';
  status: 'active' | 'inactive' | 'paused' | 'error';
  config: StrategyConfig;
  metrics: StrategyMetrics;
  lastUpdate: string;
}

export interface StrategyConfig {
  enabled: boolean;
  maxPositionSize: number;
  minProfitThreshold: number;
  maxSlippage: number;
  gasPrice: number;
  timeoutMs: number;
  [key: string]: any;
}

export interface StrategyMetrics {
  totalTrades: number;
  successfulTrades: number;
  successRate: number;
  totalProfit: number;
  avgLatency: number;
  lastTrade?: Trade;
}

export interface Trade {
  id: string;
  strategy: string;
  type: 'buy' | 'sell';
  token: string;
  amount: number;
  price: number;
  profit: number;
  fee: number;
  latency: number;
  timestamp: string;
  status: 'pending' | 'completed' | 'failed';
  txHash?: string;
}

// Market Data Types
export interface MarketData {
  symbol: string;
  price: number;
  change24h: number;
  volume24h: number;
  marketCap: number;
  timestamp: string;
}

export interface PriceHistory {
  timestamp: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

// FinGPT Types
export interface FinGPTSentiment {
  text: string;
  sentiment: 'positive' | 'negative' | 'neutral';
  confidence: number;
  reasoning: string;
  model_used: string;
  timestamp: string;
}

export interface FinGPTForecast {
  ticker: string;
  forecast: 'up' | 'down' | 'stable';
  confidence: number;
  reasoning: string;
  timeframe: string;
  model_used: string;
  timestamp: string;
}

export interface FinGPTAnalysis {
  text: string;
  task: string;
  analysis_result: string;
  model_used: string;
  timestamp: string;
}

export interface FinGPTInsights {
  market_sentiment: {
    sentiment: string;
    confidence: number;
    reasoning: string;
  };
  financial_analysis: {
    result: string;
  };
  trading_recommendations: string[];
  risk_assessment: string;
  model_used: string;
  analysis_timestamp: string;
}

// System Health Types
export interface SystemHealth {
  status: 'healthy' | 'degraded' | 'unhealthy';
  timestamp: string;
  services: Record<string, string>;
  version: string;
  dragonfly_info?: {
    version: string;
    memory_usage: string;
    connected_clients: number;
    uptime: number;
  };
}

export interface SystemMetrics {
  cpu_usage: number;
  memory_usage: number;
  disk_usage: number;
  network_io: {
    bytes_sent: number;
    bytes_recv: number;
  };
  active_connections: number;
  response_time: number;
  error_rate: number;
}

// Notification Types
export interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message: string;
  timestamp: string;
  read: boolean;
  actions?: NotificationAction[];
}

export interface NotificationAction {
  label: string;
  action: () => void;
  variant?: 'primary' | 'secondary';
}

// Chat Types
export interface ChatMessage {
  id: string;
  type: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  metadata?: {
    execution_time?: number;
    llm_used?: string;
    actions_executed?: number;
    intent?: string;
  };
}

export interface ChatSession {
  id: string;
  messages: ChatMessage[];
  created_at: string;
  updated_at: string;
}

// API Response Types
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: string;
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
  pagination: {
    page: number;
    limit: number;
    total: number;
    pages: number;
  };
}

// Chart Data Types
export interface ChartDataPoint {
  timestamp: string;
  value: number;
  label?: string;
}

export interface TimeSeriesData {
  name: string;
  data: ChartDataPoint[];
  color?: string;
}

// Dashboard Layout Types
export interface DashboardWidget {
  id: string;
  type: 'metric' | 'chart' | 'table' | 'chat' | 'custom';
  title: string;
  size: 'small' | 'medium' | 'large' | 'full';
  position: {
    x: number;
    y: number;
    w: number;
    h: number;
  };
  config: Record<string, any>;
  data?: any;
}

export interface DashboardLayout {
  id: string;
  name: string;
  widgets: DashboardWidget[];
  isDefault: boolean;
  created_at: string;
  updated_at: string;
}

// WebSocket Types
export interface WebSocketMessage {
  type: string;
  payload: any;
  timestamp: string;
}

export interface RealtimeUpdate {
  type: 'trade' | 'price' | 'metric' | 'notification' | 'system';
  data: any;
  timestamp: string;
}

// Error Types
export interface AppError {
  code: string;
  message: string;
  details?: any;
  timestamp: string;
}

// Theme Types
export interface Theme {
  name: string;
  colors: {
    primary: string;
    secondary: string;
    success: string;
    warning: string;
    error: string;
    background: string;
    surface: string;
    text: string;
  };
}

// 🧠 AI & Memory Types
export interface ContextEntry {
  context_id: string;
  content: string;
  context_type: 'MEV_OPPORTUNITY' | 'TRADE_OUTCOME_SUCCESS' | 'TRADE_OUTCOME_FAILURE' | 'RISK_ALERT' | 'WALLET_ACTIVITY';
  source: 'HFT_NINJA_WEBHOOK' | 'OUMI_DATA' | 'SCRAPY_CRAWLER' | 'MANUAL_INPUT';
  timestamp: number;
  confidence: number;
  related_strategy?: string;
  metadata?: Record<string, any>;
}

export interface RAGSearchResult {
  context_entry: ContextEntry;
  similarity_score: number;
  relevance_explanation?: string;
}

// 🔗 Webhook Event Types
export interface OpportunityEvent {
  event_type: 'opportunity_detected';
  token_address: string;
  opportunity_type: string;
  confidence: number;
  profit_potential: number;
  risk_score: number;
  trigger_wallet?: string;
  dex_involved: string;
  timestamp: number;
  metadata: Record<string, any>;
}

export interface ExecutionEvent {
  event_type: 'execution_result';
  transaction_id: string;
  strategy: string;
  token_address: string;
  outcome: 'success' | 'failure' | 'partial';
  pnl_sol: number;
  execution_time_ms: number;
  gas_used: number;
  trigger_wallet?: string;
  timestamp: number;
  metadata: Record<string, any>;
}

export interface RiskEvent {
  event_type: 'risk_event';
  risk_type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  affected_strategies: string[];
  action_taken: string;
  timestamp: number;
  metadata: Record<string, any>;
}

export interface WalletEvent {
  event_type: 'wallet_event';
  wallet_address: string;
  event_subtype: string;
  token_address?: string;
  amount_sol?: number;
  confidence: number;
  timestamp: number;
  metadata: Record<string, any>;
}

// 🤖 AI Prediction Types
export interface TradingPrediction {
  token_address: string;
  prediction_type: 'PriceIncrease' | 'PriceDecrease' | 'HighVolatility' | 'RugPull' | 'WhaleActivity';
  confidence: number;
  price_target?: number;
  time_horizon_minutes: number;
  risk_score: number;
  sentiment_score: number;
  technical_indicators: Record<string, number>;
  reasoning: string;
}

export interface MarketAnalysisAI {
  overall_sentiment: number;
  market_trend: 'Bullish' | 'Bearish' | 'Sideways' | 'Uncertain';
  volatility_index: number;
  risk_level: 'Low' | 'Medium' | 'High' | 'Critical';
  recommended_actions: RecommendedAction[];
  key_insights: string[];
}

export interface RecommendedAction {
  action_type: 'Buy' | 'Sell' | 'Hold' | 'AvoidToken' | 'IncreasePosition' | 'DecreasePosition';
  token_address?: string;
  confidence: number;
  urgency: 'Low' | 'Medium' | 'High' | 'Immediate';
  reasoning: string;
}

// 📊 Enhanced Trading Metrics
export interface EnhancedTradingMetrics extends TradingMetrics {
  aiPredictionAccuracy: number;
  webhookLatency: number;
  cacheHitRatio: number;
  activeStrategies: number;
  memoryUtilization: number;
  ragSearchQueries: number;
}

// 🔄 Real-time Data Stream Types
export interface DataStreamEvent {
  stream_type: 'opportunity' | 'execution' | 'risk' | 'wallet' | 'ai_prediction';
  event_data: OpportunityEvent | ExecutionEvent | RiskEvent | WalletEvent | TradingPrediction;
  processing_time_ms: number;
  source_system: 'rust_hft' | 'python_cerebro' | 'ai_coordinator';
}

// Export utility types
export type Status = 'loading' | 'success' | 'error' | 'idle';
export type TimeFrame = '1m' | '5m' | '15m' | '1h' | '4h' | '1d' | '1w';
export type SortDirection = 'asc' | 'desc';

export interface SortConfig {
  key: string;
  direction: SortDirection;
}

export interface FilterConfig {
  [key: string]: any;
}
