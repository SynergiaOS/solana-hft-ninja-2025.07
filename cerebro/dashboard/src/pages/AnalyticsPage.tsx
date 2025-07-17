import React from 'react';

const AnalyticsPage: React.FC = () => {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-white">Analytics</h1>
        <p className="text-gray-400 mt-1">Deep insights and performance analytics</p>
      </div>
      
      <div className="bg-[#1A1D29] border border-[#2A2D3A] rounded-xl p-8 text-center">
        <h2 className="text-xl font-semibold text-white mb-4">Advanced Analytics</h2>
        <p className="text-gray-400">Analytics dashboard coming soon...</p>
      </div>
    </div>
  );
};

export default AnalyticsPage;
