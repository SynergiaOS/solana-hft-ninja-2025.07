import React, { useMemo } from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Area,
  AreaChart,
} from 'recharts';

// Mock data - in real app this would come from API
const generateMockData = () => {
  const data = [];
  const baseValue = 45000;
  let currentValue = baseValue;
  
  for (let i = 0; i < 30; i++) {
    const change = (Math.random() - 0.5) * 2000;
    currentValue += change;
    
    data.push({
      date: new Date(Date.now() - (29 - i) * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
      value: Math.round(currentValue),
      profit: Math.round(currentValue - baseValue),
      volume: Math.round(Math.random() * 10000 + 5000),
    });
  }
  
  return data;
};

const TradingChart: React.FC = () => {
  const data = useMemo(() => generateMockData(), []);

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      const data = payload[0].payload;
      return (
        <div className="bg-[#0F1419] border border-[#2A2D3A] rounded-lg p-3 shadow-lg">
          <p className="text-gray-400 text-sm mb-2">{label}</p>
          <div className="space-y-1">
            <p className="text-white font-medium">
              Portfolio: ${data.value.toLocaleString()}
            </p>
            <p className={`text-sm font-medium ${
              data.profit >= 0 ? 'text-green-400' : 'text-red-400'
            }`}>
              P&L: {data.profit >= 0 ? '+' : ''}${data.profit.toLocaleString()}
            </p>
            <p className="text-gray-400 text-sm">
              Volume: ${data.volume.toLocaleString()}
            </p>
          </div>
        </div>
      );
    }
    return null;
  };

  const formatXAxis = (tickItem: string) => {
    const date = new Date(tickItem);
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  };

  const formatYAxis = (value: number) => {
    return `$${(value / 1000).toFixed(0)}k`;
  };

  return (
    <div className="h-80 w-full">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart
          data={data}
          margin={{
            top: 10,
            right: 30,
            left: 0,
            bottom: 0,
          }}
        >
          <defs>
            <linearGradient id="portfolioGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#8B5CF6" stopOpacity={0.3}/>
              <stop offset="95%" stopColor="#8B5CF6" stopOpacity={0}/>
            </linearGradient>
          </defs>
          
          <CartesianGrid 
            strokeDasharray="3 3" 
            stroke="#2A2D3A" 
            horizontal={true}
            vertical={false}
          />
          
          <XAxis
            dataKey="date"
            axisLine={false}
            tickLine={false}
            tick={{ fill: '#6B7280', fontSize: 12 }}
            tickFormatter={formatXAxis}
          />
          
          <YAxis
            axisLine={false}
            tickLine={false}
            tick={{ fill: '#6B7280', fontSize: 12 }}
            tickFormatter={formatYAxis}
          />
          
          <Tooltip content={<CustomTooltip />} />
          
          <Area
            type="monotone"
            dataKey="value"
            stroke="#8B5CF6"
            strokeWidth={2}
            fill="url(#portfolioGradient)"
            dot={false}
            activeDot={{
              r: 4,
              fill: '#8B5CF6',
              stroke: '#FFFFFF',
              strokeWidth: 2,
            }}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
};

export default TradingChart;
