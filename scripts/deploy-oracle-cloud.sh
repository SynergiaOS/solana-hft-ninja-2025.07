#!/bin/bash

# Oracle Cloud Deployment Script for Solana HFT Ninja
# Optimized for Oracle VCN: subnet07161247.vcn07161247.oraclevcn.com
# Private IP: 10.0.0.59

set -e

echo "🌐 Oracle Cloud Deployment - Solana HFT Ninja"
echo "=============================================="
echo "📍 Target: 10.0.0.59 (subnet07161247.vcn07161247.oraclevcn.com)"
echo ""

# Configuration
ORACLE_IP="10.0.0.59"
ORACLE_USER="opc"
PROJECT_DIR="/opt/solana-hft-ninja"
SERVICE_NAME="solana-hft-ninja"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Check if we're running on Oracle Cloud
check_oracle_environment() {
    print_info "Checking Oracle Cloud environment..."
    
    if curl -s --connect-timeout 2 http://169.254.169.254/opc/v2/instance/ > /dev/null 2>&1; then
        print_status "Running on Oracle Cloud Infrastructure"
        INSTANCE_ID=$(curl -s http://169.254.169.254/opc/v2/instance/id)
        INSTANCE_NAME=$(curl -s http://169.254.169.254/opc/v2/instance/displayName)
        print_info "Instance ID: $INSTANCE_ID"
        print_info "Instance Name: $INSTANCE_NAME"
    else
        print_warning "Not running on Oracle Cloud or metadata service unavailable"
    fi
}

# Install dependencies
install_dependencies() {
    print_info "Installing dependencies..."
    
    # Update system
    sudo dnf update -y || sudo yum update -y || sudo apt update -y
    
    # Install essential packages
    if command -v dnf &> /dev/null; then
        sudo dnf install -y curl wget git htop iotop sysstat net-tools
    elif command -v yum &> /dev/null; then
        sudo yum install -y curl wget git htop iotop sysstat net-tools
    else
        sudo apt install -y curl wget git htop iotop sysstat net-tools
    fi
    
    # Install Rust if not present
    if ! command -v cargo &> /dev/null; then
        print_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
    
    # Install Infisical CLI
    if ! command -v infisical &> /dev/null; then
        print_info "Installing Infisical CLI..."
        curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.rpm.sh' | sudo -E bash
        sudo dnf install -y infisical || sudo yum install -y infisical
    fi
    
    print_status "Dependencies installed"
}

# Optimize system for HFT
optimize_system() {
    print_info "Optimizing system for HFT performance..."
    
    # Network optimizations
    sudo tee -a /etc/sysctl.conf << EOF

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
EOF
    
    # Apply sysctl settings
    sudo sysctl -p
    
    # CPU governor optimization
    echo 'performance' | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor > /dev/null 2>&1 || true
    
    # Disable Oracle Cloud Agent (reduces overhead)
    sudo systemctl stop oracle-cloud-agent > /dev/null 2>&1 || true
    sudo systemctl disable oracle-cloud-agent > /dev/null 2>&1 || true
    
    print_status "System optimized for HFT"
}

# Configure firewall
configure_firewall() {
    print_info "Configuring firewall for HFT..."
    
    # Configure iptables
    sudo iptables -A INPUT -p tcp --dport 8080 -s 10.0.0.0/16 -j ACCEPT
    sudo iptables -A INPUT -p tcp --dport 8080 -j DROP
    sudo iptables -A INPUT -p tcp --dport 22 -j ACCEPT
    sudo iptables -A INPUT -p tcp --dport 8899 -s 169.254.169.254 -j DROP
    sudo iptables -A OUTPUT -d 10.0.0.0/8 -j ACCEPT
    
    # Save iptables rules
    sudo mkdir -p /etc/iptables
    sudo iptables-save | sudo tee /etc/iptables/rules.v4 > /dev/null
    
    print_status "Firewall configured"
}

# Setup project directory
setup_project() {
    print_info "Setting up project directory..."
    
    # Create project directory
    sudo mkdir -p $PROJECT_DIR
    sudo chown $USER:$USER $PROJECT_DIR
    
    # Copy project files if running locally
    if [ -f "Cargo.toml" ]; then
        print_info "Copying project files..."
        cp -r . $PROJECT_DIR/
        cd $PROJECT_DIR
    else
        print_warning "Run this script from the project directory or clone manually"
        return 1
    fi
    
    print_status "Project directory setup complete"
}

# Setup Infisical
setup_infisical() {
    print_info "Setting up Infisical configuration..."
    
    # Create Infisical environment file
    cat > .env.local << EOF
# Infisical Configuration for Oracle Cloud
INFISICAL_TOKEN=st.7ab7091a-ae4f-41ba-b31c-bde5bafa4599.47542cb1d455d61335eaca92b2f6abfa.941bf8d2786836054e1fec510dd5f86b
INFISICAL_PROJECT_ID=73c2f3cb-c922-4a46-a333-7b96fbc6301a
INFISICAL_ENVIRONMENT=production
INFISICAL_LOG_LEVEL=info
INFISICAL_DISABLE_UPDATE_CHECK=true
EOF
    
    # Test Infisical connection
    if source .env.local && infisical secrets --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENVIRONMENT" --token="$INFISICAL_TOKEN" > /dev/null 2>&1; then
        print_status "Infisical connection verified"
    else
        print_error "Infisical connection failed"
        return 1
    fi
}

# Build application
build_application() {
    print_info "Building Solana HFT Ninja..."
    
    # Build release version
    cargo build --release --bin hft_main
    
    if [ -f "target/release/hft_main" ]; then
        print_status "Application built successfully"
    else
        print_error "Build failed"
        return 1
    fi
}

# Create systemd service
create_service() {
    print_info "Creating systemd service..."
    
    sudo tee /etc/systemd/system/$SERVICE_NAME.service << EOF
[Unit]
Description=Solana HFT Ninja - High Frequency Trading Engine
After=network.target
Wants=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$PROJECT_DIR
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1
ExecStart=$PROJECT_DIR/scripts/run-with-infisical.sh
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
ReadWritePaths=$PROJECT_DIR

[Install]
WantedBy=multi-user.target
EOF
    
    # Reload systemd and enable service
    sudo systemctl daemon-reload
    sudo systemctl enable $SERVICE_NAME
    
    print_status "Systemd service created"
}

# Start application
start_application() {
    print_info "Starting Solana HFT Ninja..."
    
    # Start service
    sudo systemctl start $SERVICE_NAME
    
    # Wait for startup
    sleep 5
    
    # Check status
    if sudo systemctl is-active --quiet $SERVICE_NAME; then
        print_status "Application started successfully"
        
        # Test health endpoint
        if curl -s http://localhost:8080/health > /dev/null 2>&1; then
            print_status "Health endpoint responding"
        else
            print_warning "Health endpoint not responding yet"
        fi
    else
        print_error "Application failed to start"
        sudo systemctl status $SERVICE_NAME
        return 1
    fi
}

# Display deployment summary
show_summary() {
    echo ""
    echo "🎉 Oracle Cloud Deployment Complete!"
    echo "====================================="
    echo ""
    echo "📊 Access Points:"
    echo "  • Health Check: http://10.0.0.59:8080/health"
    echo "  • Metrics: http://10.0.0.59:8080/metrics"
    echo "  • DNS: http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080"
    echo ""
    echo "🔧 Management Commands:"
    echo "  • Status: sudo systemctl status $SERVICE_NAME"
    echo "  • Logs: sudo journalctl -u $SERVICE_NAME -f"
    echo "  • Restart: sudo systemctl restart $SERVICE_NAME"
    echo "  • Stop: sudo systemctl stop $SERVICE_NAME"
    echo ""
    echo "📈 Monitoring:"
    echo "  • CPU: htop"
    echo "  • Network: iotop"
    echo "  • Logs: tail -f /var/log/messages"
    echo ""
    print_status "Deployment successful! 🥷"
}

# Main deployment function
main() {
    echo "Starting Oracle Cloud deployment..."
    
    check_oracle_environment
    install_dependencies
    optimize_system
    configure_firewall
    setup_project
    setup_infisical
    build_application
    create_service
    start_application
    show_summary
}

# Run main function
main "$@"
