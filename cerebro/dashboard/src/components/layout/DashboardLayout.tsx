import React, { useState } from 'react';
import { Outlet } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';

// Components
import Sidebar from './Sidebar';
import Header from './Header';
import NotificationPanel from '../notifications/NotificationPanel';

// Hooks
import { useNotifications } from '@/hooks/useNotifications';

const DashboardLayout: React.FC = () => {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [notificationPanelOpen, setNotificationPanelOpen] = useState(false);
  const { notifications } = useNotifications();

  const unreadCount = notifications.filter(n => !n.read).length;

  return (
    <div className="flex h-screen bg-[#0B0E1A] overflow-hidden">
      {/* Sidebar */}
      <AnimatePresence mode="wait">
        {sidebarOpen && (
          <motion.div
            initial={{ x: -280, opacity: 0 }}
            animate={{ x: 0, opacity: 1 }}
            exit={{ x: -280, opacity: 0 }}
            transition={{ duration: 0.3, ease: "easeInOut" }}
            className="relative z-30"
          >
            <Sidebar onClose={() => setSidebarOpen(false)} />
          </motion.div>
        )}
      </AnimatePresence>

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <Header
          onMenuClick={() => setSidebarOpen(!sidebarOpen)}
          onNotificationClick={() => setNotificationPanelOpen(!notificationPanelOpen)}
          notificationCount={unreadCount}
        />

        {/* Page Content */}
        <main className="flex-1 overflow-y-auto bg-[#0B0E1A] relative">
          {/* Background Pattern */}
          <div className="absolute inset-0 opacity-5">
            <div className="absolute inset-0 bg-grid-pattern bg-grid"></div>
          </div>
          
          {/* Content Container */}
          <div className="relative z-10 p-6">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.4 }}
              className="max-w-7xl mx-auto"
            >
              <Outlet />
            </motion.div>
          </div>
        </main>
      </div>

      {/* Notification Panel */}
      <AnimatePresence>
        {notificationPanelOpen && (
          <motion.div
            initial={{ x: 400, opacity: 0 }}
            animate={{ x: 0, opacity: 1 }}
            exit={{ x: 400, opacity: 0 }}
            transition={{ duration: 0.3, ease: "easeInOut" }}
            className="fixed right-0 top-0 h-full w-96 z-50"
          >
            <NotificationPanel
              onClose={() => setNotificationPanelOpen(false)}
              notifications={notifications}
            />
          </motion.div>
        )}
      </AnimatePresence>

      {/* Overlay for mobile */}
      {(sidebarOpen || notificationPanelOpen) && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 bg-black/50 z-20 lg:hidden"
          onClick={() => {
            setSidebarOpen(false);
            setNotificationPanelOpen(false);
          }}
        />
      )}
    </div>
  );
};

export default DashboardLayout;
