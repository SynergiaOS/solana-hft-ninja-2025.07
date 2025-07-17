#!/bin/bash

# Deploy Solana HFT Ninja to Oracle Cloud with Infisical
# This script deploys from local machine to Oracle Cloud instance

set -e

# Configuration
ORACLE_IP="130.61.104.45"
ORACLE_USER="opc"
SSH_KEY="ssh-key-2025-07-16.key"
PROJECT_DIR="/opt/solana-hft-ninja"
REPO_URL="https://github.com/SynergiaOS/solana-hft-ninja-2025.07.git"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

echo "🚀 Deploying Solana HFT Ninja to Oracle Cloud"
echo "=============================================="
echo "📍 Target: $ORACLE_IP"
echo "👤 User: $ORACLE_USER"
echo "🔑 SSH Key: $SSH_KEY"
echo ""

# Check SSH key
if [ ! -f "$SSH_KEY" ]; then
    print_error "SSH key not found: $SSH_KEY"
    echo "Please ensure your Oracle Cloud SSH key is available"
    exit 1
fi

print_status "SSH key found"

# Test connection
print_info "Testing SSH connection..."
if ssh -i "$SSH_KEY" -o ConnectTimeout=10 -o BatchMode=yes "$ORACLE_USER@$ORACLE_IP" "echo 'Connection successful'" > /dev/null 2>&1; then
    print_status "SSH connection successful"
else
    print_error "SSH connection failed"
    echo "Please check:"
    echo "  - SSH key permissions: chmod 600 $SSH_KEY"
    echo "  - Oracle Cloud security rules"
    echo "  - Instance is running"
    exit 1
fi

# Deploy function
deploy_to_oracle() {
    print_info "Starting deployment to Oracle Cloud..."
    
    ssh -i "$SSH_KEY" "$ORACLE_USER@$ORACLE_IP" << 'EOF'
        set -e
        
        echo "🔧 Setting up Oracle Cloud environment..."
        
        # Update system
        sudo dnf update -y || sudo yum update -y
        
        # Install essential packages
        sudo dnf install -y curl wget git htop iotop sysstat net-tools || \
        sudo yum install -y curl wget git htop iotop sysstat net-tools
        
        # Install Rust if not present
        if ! command -v cargo &> /dev/null; then
            echo "📦 Installing Rust..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source ~/.cargo/env
        fi
        
        # Install Infisical CLI
        if ! command -v infisical &> /dev/null; then
            echo "🔐 Installing Infisical CLI..."
            curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.rpm.sh' | sudo -E bash
            sudo dnf install -y infisical || sudo yum install -y infisical
        fi
        
        # Create project directory
        sudo mkdir -p /opt/solana-hft-ninja
        sudo chown $USER:$USER /opt/solana-hft-ninja
        
        # Clone or update repository
        if [ -d "/opt/solana-hft-ninja/.git" ]; then
            echo "📥 Updating repository..."
            cd /opt/solana-hft-ninja
            git fetch origin
            git reset --hard origin/main
        else
            echo "📥 Cloning repository..."
            git clone https://github.com/SynergiaOS/solana-hft-ninja-2025.07.git /opt/solana-hft-ninja
            cd /opt/solana-hft-ninja
        fi
        
        # Setup Infisical environment
        echo "🔐 Setting up Infisical..."
        cat > .env.local << 'ENVEOF'
INFISICAL_TOKEN=st.7ab7091a-ae4f-41ba-b31c-bde5bafa4599.47542cb1d455d61335eaca92b2f6abfa.941bf8d2786836054e1fec510dd5f86b
INFISICAL_PROJECT_ID=73c2f3cb-c922-4a46-a333-7b96fbc6301a
INFISICAL_ENVIRONMENT=production
INFISICAL_LOG_LEVEL=info
INFISICAL_DISABLE_UPDATE_CHECK=true
ENVEOF
        
        # Test Infisical connection
        source .env.local
        if infisical secrets --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENVIRONMENT" --token="$INFISICAL_TOKEN" > /dev/null 2>&1; then
            echo "✅ Infisical connection verified"
        else
            echo "❌ Infisical connection failed"
            exit 1
        fi
        
        # Build application
        echo "🔨 Building application..."
        source ~/.cargo/env
        cargo build --release --bin hft_main
        
        if [ -f "target/release/hft_main" ]; then
            echo "✅ Application built successfully"
        else
            echo "❌ Build failed"
            exit 1
        fi
        
        # Create systemd service
        echo "⚙️ Creating systemd service..."
        sudo tee /etc/systemd/system/solana-hft-ninja.service << 'SERVICEEOF'
[Unit]
Description=Solana HFT Ninja - High Frequency Trading Engine
After=network.target
Wants=network.target

[Service]
Type=simple
User=opc
WorkingDirectory=/opt/solana-hft-ninja
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1
ExecStart=/opt/solana-hft-ninja/scripts/run-with-infisical.sh
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=solana-hft-ninja

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/solana-hft-ninja

[Install]
WantedBy=multi-user.target
SERVICEEOF
        
        # Make scripts executable
        chmod +x scripts/*.sh
        
        # Reload systemd and enable service
        sudo systemctl daemon-reload
        sudo systemctl enable solana-hft-ninja
        
        # System optimizations for HFT
        echo "⚡ Optimizing system for HFT..."
        sudo tee -a /etc/sysctl.conf << 'SYSCTLEOF'

# HFT Network Optimization for Oracle Cloud
net.core.rmem_max = 2147483647
net.core.wmem_max = 2147483647
net.ipv4.tcp_rmem = 4096 87380 2147483647
net.ipv4.tcp_wmem = 4096 65536 2147483647
net.ipv4.udp_mem = 3145728 4194304 16777216
net.core.netdev_max_backlog = 30000
net.ipv4.tcp_congestion_control = bbr
net.ipv4.tcp_fastopen = 3
net.core.somaxconn = 65535
SYSCTLEOF
        
        sudo sysctl -p
        
        # Configure firewall
        echo "🔥 Configuring firewall..."
        sudo iptables -A INPUT -p tcp --dport 8080 -s 10.0.0.0/16 -j ACCEPT
        sudo iptables -A INPUT -p tcp --dport 8080 -j DROP
        sudo iptables -A INPUT -p tcp --dport 22 -j ACCEPT
        
        # Start the service
        echo "🚀 Starting Solana HFT Ninja..."
        sudo systemctl start solana-hft-ninja
        
        # Wait for startup
        sleep 10
        
        # Check status
        if sudo systemctl is-active --quiet solana-hft-ninja; then
            echo "✅ Service started successfully"
            
            # Test health endpoint
            if curl -s http://localhost:8080/health > /dev/null 2>&1; then
                echo "✅ Health endpoint responding"
            else
                echo "⚠️ Health endpoint not responding yet (may need more time)"
            fi
        else
            echo "❌ Service failed to start"
            sudo systemctl status solana-hft-ninja
            exit 1
        fi
        
        echo ""
        echo "🎉 Deployment Complete!"
        echo "======================"
        echo ""
        echo "📊 Access Points:"
        echo "  • Health: http://130.61.104.45:8080/health"
        echo "  • Metrics: http://130.61.104.45:8080/metrics"
        echo ""
        echo "🔧 Management:"
        echo "  • Status: sudo systemctl status solana-hft-ninja"
        echo "  • Logs: sudo journalctl -u solana-hft-ninja -f"
        echo "  • Restart: sudo systemctl restart solana-hft-ninja"
        echo ""
        echo "✅ Solana HFT Ninja is running on Oracle Cloud! 🥷💰"
EOF
}

# Run deployment
deploy_to_oracle

print_status "Deployment completed successfully!"
print_info "You can now access your HFT system at:"
echo "  • Health Check: http://$ORACLE_IP:8080/health"
echo "  • Metrics: http://$ORACLE_IP:8080/metrics"
echo ""
print_info "To monitor the system:"
echo "  ssh -i $SSH_KEY $ORACLE_USER@$ORACLE_IP"
echo "  sudo journalctl -u solana-hft-ninja -f"
