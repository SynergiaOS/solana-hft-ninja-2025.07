import React from 'react';
import { cn } from '@/lib/utils';

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
  variant?: 'default' | 'gradient' | 'glass';
}

const Card: React.FC<CardProps> = ({ 
  children, 
  className, 
  variant = 'default',
  ...props 
}) => {
  const baseClasses = "rounded-xl border transition-all duration-200";
  
  const variantClasses = {
    default: "bg-[#1A1D29] border-[#2A2D3A] hover:border-[#3A3D4A]",
    gradient: "bg-gradient-to-br from-[#1A1D29] to-[#2A2D3A] border-[#3A3D4A] hover:border-[#4A4D5A]",
    glass: "bg-black/20 backdrop-blur-sm border-white/10 hover:border-white/20"
  };

  return (
    <div 
      className={cn(baseClasses, variantClasses[variant], className)}
      {...props}
    >
      {children}
    </div>
  );
};

export default Card;
