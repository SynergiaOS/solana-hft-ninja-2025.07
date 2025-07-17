import React from 'react';

const SystemPage: React.FC = () => {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-white">System</h1>
        <p className="text-gray-400 mt-1">System health and monitoring</p>
      </div>
      
      <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-8 text-center">
        <h2 className="text-xl font-semibold text-white mb-4">System Monitoring</h2>
        <p className="text-gray-400">System dashboard coming soon...</p>
      </div>
    </div>
  );
};

export default SystemPage;
