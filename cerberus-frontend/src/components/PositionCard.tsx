"use client";

import { motion } from "framer-motion";
import { useState } from "react";
import { toast } from "sonner";
import { TrendingUp, TrendingDown, Clock, DollarSign, Target, AlertTriangle } from "lucide-react";

interface Position {
  mint: string;
  symbol?: string;
  entryPrice: number;
  currentPrice: number;
  positionSizeSol: number;
  pnl: number;
  pnlPercent: number;
  status: "open" | "closed" | "pending" | "failed";
  strategy: string;
  ageSeconds: number;
  takeProfitTarget: number;
  stopLossTarget: number;
  timeoutSeconds: number;
}

interface PositionCardProps {
  position: Position;
  onSell?: (mint: string) => void;
  onEdit?: (mint: string) => void;
}

export default function PositionCard({ position, onSell, onEdit }: PositionCardProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState(0);

  const formatMint = (mint: string) => {
    return position.symbol || `${mint.slice(0, 4)}...${mint.slice(-4)}`;
  };

  const formatTime = (seconds: number) => {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    return `${Math.floor(seconds / 3600)}h`;
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "open": return "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400";
      case "closed": return "bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400";
      case "pending": return "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400";
      case "failed": return "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400";
      default: return "bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400";
    }
  };

  const handleDragEnd = (_event: unknown, info: { offset: { x: number } }) => {
    setIsDragging(false);
    setDragOffset(0);

    // Swipe left to sell (threshold: -80px)
    if (info.offset.x < -80) {
      if (onSell) {
        onSell(position.mint);
        toast.success(`Selling ${formatMint(position.mint)}`, {
          description: "Position sell order initiated",
        });
      }
    }
    // Swipe right to edit (threshold: 80px)
    else if (info.offset.x > 80) {
      if (onEdit) {
        onEdit(position.mint);
      }
    }
  };

  const handleDrag = (_event: unknown, info: { offset: { x: number } }) => {
    setDragOffset(info.offset.x);
  };

  return (
    <motion.div
      drag="x"
      dragConstraints={{ left: -120, right: 120 }}
      dragElastic={0.2}
      onDragStart={() => setIsDragging(true)}
      onDrag={handleDrag}
      onDragEnd={handleDragEnd}
      className="relative overflow-hidden"
      whileTap={{ scale: 0.98 }}
    >
      {/* Swipe Actions Background */}
      <div className="absolute inset-0 flex">
        {/* Edit Action (Right Swipe) */}
        <div className="flex-1 bg-blue-500 flex items-center justify-start pl-6">
          <div className="text-white font-semibold">
            <Target className="w-6 h-6" />
          </div>
        </div>
        
        {/* Sell Action (Left Swipe) */}
        <div className="flex-1 bg-red-500 flex items-center justify-end pr-6">
          <div className="text-white font-semibold">
            <AlertTriangle className="w-6 h-6" />
          </div>
        </div>
      </div>

      {/* Main Card */}
      <motion.div
        className={`relative bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl p-4 shadow-sm transition-all duration-200 ${
          isDragging ? "shadow-lg" : ""
        }`}
        style={{
          x: dragOffset,
        }}
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-3">
            <div className="text-lg font-bold text-gray-900 dark:text-white">
              {formatMint(position.mint)}
            </div>
            <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(position.status)}`}>
              {position.status.toUpperCase()}
            </span>
          </div>
          
          <div className="text-right">
            <div className={`text-lg font-bold ${
              position.pnlPercent >= 0 
                ? "text-green-500 dark:text-green-400" 
                : "text-red-500 dark:text-red-400"
            }`}>
              {position.pnlPercent >= 0 ? "+" : ""}{position.pnlPercent.toFixed(2)}%
            </div>
            <div className="text-sm text-gray-500 dark:text-gray-400">
              {position.pnl >= 0 ? "+" : ""}{position.pnl.toFixed(3)} SOL
            </div>
          </div>
        </div>

        {/* Price Info */}
        <div className="grid grid-cols-2 gap-4 mb-3">
          <div>
            <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">Entry Price</div>
            <div className="font-mono text-sm text-gray-900 dark:text-white">
              ${position.entryPrice.toFixed(4)}
            </div>
          </div>
          <div>
            <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">Current Price</div>
            <div className="font-mono text-sm text-gray-900 dark:text-white flex items-center gap-1">
              ${position.currentPrice.toFixed(4)}
              {position.currentPrice > position.entryPrice ? (
                <TrendingUp className="w-3 h-3 text-green-500" />
              ) : (
                <TrendingDown className="w-3 h-3 text-red-500" />
              )}
            </div>
          </div>
        </div>

        {/* Position Details */}
        <div className="grid grid-cols-3 gap-3 text-xs">
          <div className="flex items-center gap-1">
            <DollarSign className="w-3 h-3 text-gray-400" />
            <span className="text-gray-500 dark:text-gray-400">Size:</span>
            <span className="font-medium text-gray-900 dark:text-white">
              {position.positionSizeSol.toFixed(3)}
            </span>
          </div>
          
          <div className="flex items-center gap-1">
            <Clock className="w-3 h-3 text-gray-400" />
            <span className="text-gray-500 dark:text-gray-400">Age:</span>
            <span className="font-medium text-gray-900 dark:text-white">
              {formatTime(position.ageSeconds)}
            </span>
          </div>
          
          <div className="flex items-center gap-1">
            <Target className="w-3 h-3 text-gray-400" />
            <span className="text-gray-500 dark:text-gray-400">Strategy:</span>
            <span className="font-medium text-gray-900 dark:text-white truncate">
              {position.strategy.replace("_", " ")}
            </span>
          </div>
        </div>

        {/* Risk Levels */}
        <div className="mt-3 pt-3 border-t border-gray-100 dark:border-gray-700">
          <div className="flex justify-between text-xs">
            <div className="flex items-center gap-1">
              <div className="w-2 h-2 bg-red-500 rounded-full"></div>
              <span className="text-gray-500 dark:text-gray-400">SL:</span>
              <span className="text-red-500 font-medium">{position.stopLossTarget}%</span>
            </div>
            
            <div className="flex items-center gap-1">
              <div className="w-2 h-2 bg-green-500 rounded-full"></div>
              <span className="text-gray-500 dark:text-gray-400">TP:</span>
              <span className="text-green-500 font-medium">{position.takeProfitTarget}%</span>
            </div>
            
            <div className="flex items-center gap-1">
              <Clock className="w-3 h-3 text-gray-400" />
              <span className="text-gray-500 dark:text-gray-400">Timeout:</span>
              <span className="text-gray-900 dark:text-white font-medium">
                {formatTime(position.timeoutSeconds)}
              </span>
            </div>
          </div>
        </div>

        {/* Swipe Hint */}
        {isDragging && (
          <div className="absolute top-2 right-2 text-xs text-gray-500 dark:text-gray-400">
            {dragOffset > 20 ? "→ Edit" : dragOffset < -20 ? "← Sell" : "Swipe"}
          </div>
        )}
      </motion.div>
    </motion.div>
  );
}
