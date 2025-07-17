import React, { useState, useEffect, useRef } from 'react';
import { Send, Bot, User, Loader, AlertCircle, TrendingUp, Brain } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { tomorrow } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface Message {
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

interface CerebroChatInterfaceProps {
  apiUrl?: string;
  className?: string;
  onMessageSent?: (message: string) => void;
  onResponseReceived?: (response: Message) => void;
}

const CerebroChatInterface: React.FC<CerebroChatInterfaceProps> = ({
  apiUrl = 'http://localhost:8000',
  className = '',
  onMessageSent,
  onResponseReceived
}) => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sessionId, setSessionId] = useState<string>('');
  
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Initialize with welcome message
  useEffect(() => {
    const welcomeMessage: Message = {
      id: 'welcome',
      type: 'system',
      content: `# 🧠 Welcome to Cerebro AI Assistant

I'm your intelligent trading companion for Solana HFT analysis. I can help you with:

- **📊 Performance Analysis** - Review your trading metrics and profitability
- **⚡ Strategy Optimization** - Improve your MEV strategies and parameters  
- **📈 Market Insights** - Analyze market conditions and sentiment
- **🔧 System Monitoring** - Check system health and performance
- **💡 Recommendations** - Get AI-powered suggestions for better trading

**Quick commands to try:**
- "How is my trading performance today?"
- "Analyze my sandwich strategy"
- "What's the current market sentiment?"
- "Check system health"

What would you like to know?`,
      timestamp: new Date().toISOString()
    };
    
    setMessages([welcomeMessage]);
    setSessionId(`session_${Date.now()}`);
  }, []);

  const sendMessage = async () => {
    if (!inputValue.trim() || isLoading) return;

    const userMessage: Message = {
      id: `user_${Date.now()}`,
      type: 'user',
      content: inputValue.trim(),
      timestamp: new Date().toISOString()
    };

    setMessages(prev => [...prev, userMessage]);
    setInputValue('');
    setIsLoading(true);
    setError(null);

    // Call onMessageSent callback
    onMessageSent?.(userMessage.content);

    try {
      const response = await fetch(`${apiUrl}/api/prompt`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          prompt: userMessage.content,
          user_id: sessionId,
          context: {
            session_id: sessionId,
            timestamp: userMessage.timestamp
          }
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();

      const assistantMessage: Message = {
        id: `assistant_${Date.now()}`,
        type: 'assistant',
        content: data.response,
        timestamp: data.timestamp,
        metadata: {
          execution_time: data.execution_time_ms,
          llm_used: data.llm_used,
          actions_executed: data.actions_executed,
          intent: data.intent
        }
      };

      setMessages(prev => [...prev, assistantMessage]);
      
      // Call onResponseReceived callback
      onResponseReceived?.(assistantMessage);

    } catch (err) {
      const errorMessage: Message = {
        id: `error_${Date.now()}`,
        type: 'system',
        content: `❌ **Error**: ${err instanceof Error ? err.message : 'Unknown error occurred'}`,
        timestamp: new Date().toISOString()
      };
      
      setMessages(prev => [...prev, errorMessage]);
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  const clearChat = () => {
    setMessages([]);
    setError(null);
    setSessionId(`session_${Date.now()}`);
  };

  const getMessageIcon = (type: string) => {
    switch (type) {
      case 'user':
        return <User className="w-5 h-5 text-blue-500" />;
      case 'assistant':
        return <Brain className="w-5 h-5 text-purple-500" />;
      case 'system':
        return <Bot className="w-5 h-5 text-gray-500" />;
      default:
        return <AlertCircle className="w-5 h-5 text-red-500" />;
    }
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  return (
    <div className={`flex flex-col h-full bg-white border border-gray-200 rounded-lg shadow-lg ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200 bg-gradient-to-r from-purple-50 to-blue-50">
        <div className="flex items-center space-x-2">
          <Brain className="w-6 h-6 text-purple-600" />
          <h3 className="text-lg font-semibold text-gray-800">Cerebro AI Assistant</h3>
          {sessionId && (
            <span className="text-xs text-gray-500 bg-gray-100 px-2 py-1 rounded">
              {sessionId.slice(-8)}
            </span>
          )}
        </div>
        <button
          onClick={clearChat}
          className="text-sm text-gray-500 hover:text-gray-700 px-3 py-1 rounded hover:bg-gray-100"
        >
          Clear Chat
        </button>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex items-start space-x-3 ${
              message.type === 'user' ? 'flex-row-reverse space-x-reverse' : ''
            }`}
          >
            <div className="flex-shrink-0 mt-1">
              {getMessageIcon(message.type)}
            </div>
            
            <div className={`flex-1 ${message.type === 'user' ? 'text-right' : ''}`}>
              <div
                className={`inline-block max-w-full p-3 rounded-lg ${
                  message.type === 'user'
                    ? 'bg-blue-500 text-white'
                    : message.type === 'system'
                    ? 'bg-gray-100 text-gray-800'
                    : 'bg-purple-50 text-gray-800 border border-purple-200'
                }`}
              >
                {message.type === 'user' ? (
                  <p className="whitespace-pre-wrap">{message.content}</p>
                ) : (
                  <ReactMarkdown
                    className="prose prose-sm max-w-none"
                    components={{
                      code({ node, inline, className, children, ...props }) {
                        const match = /language-(\w+)/.exec(className || '');
                        return !inline && match ? (
                          <SyntaxHighlighter
                            style={tomorrow}
                            language={match[1]}
                            PreTag="div"
                            className="rounded-md"
                            {...props}
                          >
                            {String(children).replace(/\n$/, '')}
                          </SyntaxHighlighter>
                        ) : (
                          <code className={className} {...props}>
                            {children}
                          </code>
                        );
                      },
                    }}
                  >
                    {message.content}
                  </ReactMarkdown>
                )}
              </div>
              
              {/* Message metadata */}
              <div className={`mt-1 text-xs text-gray-500 ${message.type === 'user' ? 'text-right' : ''}`}>
                <span>{formatTimestamp(message.timestamp)}</span>
                {message.metadata && (
                  <>
                    {message.metadata.execution_time && (
                      <span className="ml-2">⚡ {message.metadata.execution_time}ms</span>
                    )}
                    {message.metadata.llm_used && (
                      <span className="ml-2">🧠 {message.metadata.llm_used}</span>
                    )}
                    {message.metadata.actions_executed && (
                      <span className="ml-2">🔧 {message.metadata.actions_executed} actions</span>
                    )}
                    {message.metadata.intent && (
                      <span className="ml-2">🎯 {message.metadata.intent}</span>
                    )}
                  </>
                )}
              </div>
            </div>
          </div>
        ))}
        
        {/* Loading indicator */}
        {isLoading && (
          <div className="flex items-center space-x-3">
            <div className="flex-shrink-0">
              <Brain className="w-5 h-5 text-purple-500" />
            </div>
            <div className="flex-1">
              <div className="inline-block p-3 bg-purple-50 border border-purple-200 rounded-lg">
                <div className="flex items-center space-x-2">
                  <Loader className="w-4 h-4 animate-spin text-purple-500" />
                  <span className="text-gray-600">Cerebro is thinking...</span>
                </div>
              </div>
            </div>
          </div>
        )}
        
        <div ref={messagesEndRef} />
      </div>

      {/* Error display */}
      {error && (
        <div className="p-3 bg-red-50 border-t border-red-200">
          <div className="flex items-center space-x-2 text-red-700">
            <AlertCircle className="w-4 h-4" />
            <span className="text-sm">{error}</span>
          </div>
        </div>
      )}

      {/* Input */}
      <div className="p-4 border-t border-gray-200 bg-gray-50">
        <div className="flex space-x-2">
          <input
            ref={inputRef}
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Ask Cerebro about your trading performance..."
            className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent"
            disabled={isLoading}
          />
          <button
            onClick={sendMessage}
            disabled={!inputValue.trim() || isLoading}
            className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Send className="w-4 h-4" />
          </button>
        </div>
        
        {/* Quick actions */}
        <div className="mt-2 flex flex-wrap gap-2">
          {[
            "Check performance",
            "Analyze strategies", 
            "Market sentiment",
            "System health"
          ].map((action) => (
            <button
              key={action}
              onClick={() => setInputValue(action)}
              className="text-xs px-2 py-1 bg-white border border-gray-300 rounded text-gray-600 hover:bg-gray-50"
              disabled={isLoading}
            >
              {action}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};

export default CerebroChatInterface;
