import { useState, useEffect } from 'react';
import { Notification } from '@/types';

// Mock notifications - in real app this would come from WebSocket or API
const mockNotifications: Notification[] = [
  {
    id: '1',
    type: 'success',
    title: 'Trade Executed',
    message: 'Sandwich strategy executed successfully. Profit: +$234.56',
    timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
    read: false,
  },
  {
    id: '2',
    type: 'info',
    title: 'FinGPT Analysis',
    message: 'New market sentiment analysis available for SOL',
    timestamp: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
    read: false,
  },
  {
    id: '3',
    type: 'warning',
    title: 'High Slippage Detected',
    message: 'Arbitrage strategy experiencing higher than normal slippage',
    timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
    read: true,
  },
  {
    id: '4',
    type: 'error',
    title: 'Strategy Paused',
    message: 'Liquidation strategy paused due to insufficient balance',
    timestamp: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
    read: true,
  },
];

export const useNotifications = () => {
  const [notifications, setNotifications] = useState<Notification[]>(mockNotifications);

  const markAsRead = (id: string) => {
    setNotifications(prev =>
      prev.map(notification =>
        notification.id === id
          ? { ...notification, read: true }
          : notification
      )
    );
  };

  const markAllAsRead = () => {
    setNotifications(prev =>
      prev.map(notification => ({ ...notification, read: true }))
    );
  };

  const removeNotification = (id: string) => {
    setNotifications(prev =>
      prev.filter(notification => notification.id !== id)
    );
  };

  const addNotification = (notification: Omit<Notification, 'id'>) => {
    const newNotification: Notification = {
      ...notification,
      id: Date.now().toString(),
    };
    setNotifications(prev => [newNotification, ...prev]);
  };

  // Simulate real-time notifications
  useEffect(() => {
    const interval = setInterval(() => {
      if (Math.random() > 0.8) { // 20% chance every 30 seconds
        const types: Notification['type'][] = ['info', 'success', 'warning'];
        const randomType = types[Math.floor(Math.random() * types.length)];
        
        addNotification({
          type: randomType,
          title: 'System Update',
          message: 'New trading opportunity detected',
          timestamp: new Date().toISOString(),
          read: false,
        });
      }
    }, 30000);

    return () => clearInterval(interval);
  }, []);

  return {
    notifications,
    markAsRead,
    markAllAsRead,
    removeNotification,
    addNotification,
  };
};
