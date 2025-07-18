import React from 'react';
import { cn } from '@/lib/utils';

interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: 'default' | 'secondary' | 'success' | 'destructive' | 'warning' | 'outline';
  size?: 'sm' | 'md' | 'lg';
  children: React.ReactNode;
}

const Badge: React.FC<BadgeProps> = ({ 
  variant = 'default', 
  size = 'md',
  className, 
  children, 
  ...props 
}) => {
  const baseClasses = "inline-flex items-center font-medium rounded-full transition-colors";
  
  const sizeClasses = {
    sm: "px-2 py-0.5 text-xs",
    md: "px-2.5 py-1 text-sm",
    lg: "px-3 py-1.5 text-base"
  };

  const variantClasses = {
    default: "bg-purple-500/20 text-purple-300 border border-purple-500/30",
    secondary: "bg-gray-500/20 text-gray-300 border border-gray-500/30",
    success: "bg-green-500/20 text-green-300 border border-green-500/30",
    destructive: "bg-red-500/20 text-red-300 border border-red-500/30",
    warning: "bg-yellow-500/20 text-yellow-300 border border-yellow-500/30",
    outline: "bg-transparent text-gray-300 border border-gray-500/50"
  };

  return (
    <span 
      className={cn(baseClasses, sizeClasses[size], variantClasses[variant], className)}
      {...props}
    >
      {children}
    </span>
  );
};

export default Badge;
