"use client";

import { useState } from "react";
import { toast } from "sonner";
import LiveTicker from "@/components/LiveTicker";
import PanicButton from "@/components/PanicButton";
import PositionCard from "@/components/PositionCard";
import { usePositions } from "@/hooks/usePositions";
import { TrendingUp, TrendingDown, DollarSign, Target, Clock, AlertTriangle } from "lucide-react";

export default function DashboardPage() {
  const { data, positions, isConnected, refetch } = usePositions();
  const [selectedFilter, setSelectedFilter] = useState<"all" | "open" | "profitable">("all");

  const handleSellPosition = async (mint: string) => {
    try {
      const response = await fetch("/api/positions/panic", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          action: "SELL_POSITION",
          mint,
          reason: "MANUAL_SELL",
        }),
      });

      if (response.ok) {
        toast.success("Position sell order initiated");
        refetch();
      } else {
        throw new Error("Failed to sell position");
      }
    } catch (error) {
      toast.error("Failed to sell position");
    }
  };

  const handleEditPosition = (mint: string) => {
    toast.info(`Edit position ${mint.slice(0, 8)}... (Coming soon)`);
  };

  const filteredPositions = positions.filter(pos => {
    switch (selectedFilter) {
      case "open": return pos.status === "open";
      case "profitable": return pos.pnl > 0;
      default: return true;
    }
  });

  if (!data) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <AlertTriangle className="w-12 h-12 text-red-500 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
            Connection Error
          </h2>
          <p className="text-gray-600 dark:text-gray-400 mb-4">Failed to load data</p>
          <button
            onClick={refetch}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Retry Connection
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Live Ticker */}
      <LiveTicker />

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
              🧠 Cerberus Dashboard
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mt-1">
              Autonomous position management and real-time monitoring
            </p>
          </div>
          
          <div className="flex items-center gap-2">
            <div className={`w-3 h-3 rounded-full ${isConnected ? "bg-green-500" : "bg-red-500"}`} />
            <span className="text-sm text-gray-600 dark:text-gray-400">
              {isConnected ? "Connected" : "Disconnected"}
            </span>
          </div>
        </div>

        {/* Metrics Cards */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Total Value</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white">
                  {data?.totalValue.toFixed(3) || "0.000"} SOL
                </p>
              </div>
              <DollarSign className="w-8 h-8 text-blue-500" />
            </div>
          </div>

          <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Total P&L</p>
                <p className={`text-2xl font-bold ${
                  (data?.totalPnlPercent || 0) >= 0 
                    ? "text-green-500 dark:text-green-400" 
                    : "text-red-500 dark:text-red-400"
                }`}>
                  {(data?.totalPnlPercent || 0) >= 0 ? "+" : ""}
                  {data?.totalPnlPercent.toFixed(2) || "0.00"}%
                </p>
              </div>
              {(data?.totalPnlPercent || 0) >= 0 ? (
                <TrendingUp className="w-8 h-8 text-green-500" />
              ) : (
                <TrendingDown className="w-8 h-8 text-red-500" />
              )}
            </div>
          </div>

          <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Active Positions</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white">
                  {data?.activeCount || 0}
                </p>
              </div>
              <Target className="w-8 h-8 text-purple-500" />
            </div>
          </div>

          <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Profitable</p>
                <p className="text-2xl font-bold text-green-500 dark:text-green-400">
                  {data?.profitableCount || 0}/{data?.activeCount || 0}
                </p>
              </div>
              <TrendingUp className="w-8 h-8 text-green-500" />
            </div>
          </div>
        </div>

        {/* Emergency Controls */}
        <div className="mb-8">
          <PanicButton />
        </div>

        {/* Position Filters */}
        <div className="flex items-center gap-4 mb-6">
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Filter:</span>
          {[
            { key: "all", label: "All Positions" },
            { key: "open", label: "Open Only" },
            { key: "profitable", label: "Profitable" },
          ].map(filter => (
            <button
              key={filter.key}
              onClick={() => setSelectedFilter(filter.key as "all" | "open" | "profitable")}
              className={`px-3 py-1 text-sm rounded-full transition-colors ${
                selectedFilter === filter.key
                  ? "bg-blue-600 text-white"
                  : "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
              }`}
            >
              {filter.label}
            </button>
          ))}
        </div>

        {/* Positions List */}
        <div className="space-y-4">
          {filteredPositions.length > 0 ? (
            filteredPositions.map((position) => (
              <PositionCard
                key={position.mint}
                position={position}
                onSell={handleSellPosition}
                onEdit={handleEditPosition}
              />
            ))
          ) : (
            <div className="text-center py-12">
              <Clock className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                No positions found
              </h3>
              <p className="text-gray-600 dark:text-gray-400">
                {selectedFilter === "all" 
                  ? "No active positions at the moment"
                  : `No ${selectedFilter} positions found`
                }
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
