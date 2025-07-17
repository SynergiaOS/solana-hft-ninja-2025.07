import React from 'react';

const SettingsPage: React.FC = () => {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-white">Settings</h1>
        <p className="text-gray-400 mt-1">Configure your dashboard and preferences</p>
      </div>
      
      <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-8 text-center">
        <h2 className="text-xl font-semibold text-white mb-4">Dashboard Settings</h2>
        <p className="text-gray-400">Settings panel coming soon...</p>
      </div>
    </div>
  );
};

export default SettingsPage;
