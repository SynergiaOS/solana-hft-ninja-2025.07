"use client";

import { useState } from "react";
import { toast } from "sonner";
import { AlertTriangle, Shield } from "lucide-react";

interface PanicButtonProps {
  className?: string;
  size?: "sm" | "md" | "lg";
}

export default function PanicButton({ className = "", size = "lg" }: PanicButtonProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);

  const sizeClasses = {
    sm: "px-4 py-2 text-sm",
    md: "px-6 py-3 text-base",
    lg: "px-8 py-4 text-lg",
  };

  const handlePanic = async () => {
    if (!showConfirm) {
      setShowConfirm(true);
      // Auto-hide confirmation after 5 seconds
      setTimeout(() => setShowConfirm(false), 5000);
      return;
    }

    setIsLoading(true);
    
    try {
      const response = await fetch("/api/positions/panic", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          action: "EXIT_ALL_POSITIONS",
          reason: "MANUAL_PANIC_STOP",
          timestamp: Date.now(),
        }),
      });

      if (response.ok) {
        const result = await response.json();
        toast.success("🚨 Emergency Stop Executed", {
          description: `All positions closed. ${result.positionsClosed || 0} positions affected.`,
          duration: 10000,
        });
      } else {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
    } catch (error) {
      console.error("Panic button error:", error);
      toast.error("❌ Emergency Stop Failed", {
        description: "Failed to execute emergency stop. Please try again or contact support.",
        duration: 10000,
      });
    } finally {
      setIsLoading(false);
      setShowConfirm(false);
    }
  };

  const handleCancel = () => {
    setShowConfirm(false);
  };

  if (showConfirm) {
    return (
      <div className={`space-y-3 ${className}`}>
        {/* Confirmation Message */}
        <div className="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <div className="flex items-center gap-2 text-red-800 dark:text-red-200">
            <AlertTriangle className="w-5 h-5" />
            <span className="font-semibold">Confirm Emergency Stop</span>
          </div>
          <p className="text-sm text-red-700 dark:text-red-300 mt-1">
            This will immediately close ALL active positions. This action cannot be undone.
          </p>
        </div>

        {/* Confirmation Buttons */}
        <div className="flex gap-3">
          <button
            onClick={handlePanic}
            disabled={isLoading}
            className={`flex-1 ${sizeClasses[size]} font-bold text-white bg-red-600 hover:bg-red-700 disabled:bg-red-400 rounded-xl shadow-lg transition-all duration-200 transform hover:scale-105 disabled:scale-100 disabled:cursor-not-allowed`}
          >
            {isLoading ? (
              <div className="flex items-center justify-center gap-2">
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                Executing...
              </div>
            ) : (
              <div className="flex items-center justify-center gap-2">
                <AlertTriangle className="w-5 h-5" />
                YES, STOP ALL
              </div>
            )}
          </button>

          <button
            onClick={handleCancel}
            disabled={isLoading}
            className={`flex-1 ${sizeClasses[size]} font-semibold text-gray-700 dark:text-gray-300 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-50 rounded-xl transition-all duration-200`}
          >
            Cancel
          </button>
        </div>
      </div>
    );
  }

  return (
    <button
      onClick={handlePanic}
      disabled={isLoading}
      className={`w-full ${sizeClasses[size]} font-bold text-white bg-gradient-to-r from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 disabled:from-red-400 disabled:to-red-500 rounded-xl shadow-lg transition-all duration-200 transform hover:scale-105 disabled:scale-100 disabled:cursor-not-allowed ${className}`}
    >
      <div className="flex items-center justify-center gap-3">
        <div className="relative">
          <Shield className="w-6 h-6" />
          <div className="absolute -top-1 -right-1 w-3 h-3 bg-yellow-400 rounded-full animate-pulse" />
        </div>
        <span>🚨 PANIC SELL ALL</span>
      </div>
    </button>
  );
}
