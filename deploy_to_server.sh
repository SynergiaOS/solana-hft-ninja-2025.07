#!/bin/bash
# Deploy Solana HFT Ninja + Cerebro to Server
# Analogicznie do lokalnego setupu, ale na produkcji

echo "🚀 Deploying Solana HFT Ninja + Cerebro to Server"
echo "=================================================="

# Server configuration
SERVER_IP="your-server-ip"
SERVER_USER="ubuntu"
SSH_KEY="ssh-key-2025-07-16-new.key"
PROJECT_DIR="/home/ubuntu/solana-hft-ninja"

echo "📋 Deployment Configuration:"
echo "Server: $SERVER_USER@$SERVER_IP"
echo "SSH Key: $SSH_KEY"
echo "Target Dir: $PROJECT_DIR"
echo ""

# Step 1: Create deployment package
echo "📦 Step 1: Creating deployment package..."
tar -czf solana-hft-deployment.tar.gz \
    --exclude='target' \
    --exclude='node_modules' \
    --exclude='.git' \
    --exclude='*.log' \
    --exclude='venv' \
    --exclude='__pycache__' \
    src/ \
    cerebro/ \
    Cargo.toml \
    Cargo.lock \
    README.md \
    .env.example

echo "✅ Deployment package created: solana-hft-deployment.tar.gz"

# Step 2: Upload to server
echo "📤 Step 2: Uploading to server..."
scp -i $SSH_KEY solana-hft-deployment.tar.gz $SERVER_USER@$SERVER_IP:~/

echo "✅ Files uploaded to server"

# Step 3: Server setup script
cat > server_setup.sh << 'EOF'
#!/bin/bash
echo "🔧 Setting up Solana HFT Ninja on server..."

# Update system
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y curl build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install Python
sudo apt install -y python3 python3-pip python3-venv

# Create project directory
mkdir -p /home/ubuntu/solana-hft-ninja
cd /home/ubuntu/solana-hft-ninja

# Extract deployment package
tar -xzf ~/solana-hft-deployment.tar.gz

# Build HFT Ninja (Rust)
echo "🦀 Building HFT Ninja..."
cargo build --release

# Setup Cerebro (Python)
echo "🧠 Setting up Cerebro..."
cd cerebro
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Setup Dashboard (Node.js)
echo "📊 Setting up Dashboard..."
cd dashboard
npm install
npm run build

echo "✅ Server setup complete!"
echo "🎯 Next steps:"
echo "1. Configure environment variables"
echo "2. Start services with systemd"
echo "3. Setup nginx reverse proxy"
EOF

# Step 4: Upload and run setup script
echo "📤 Step 3: Uploading setup script..."
scp -i $SSH_KEY server_setup.sh $SERVER_USER@$SERVER_IP:~/
ssh -i $SSH_KEY $SERVER_USER@$SERVER_IP 'chmod +x server_setup.sh && ./server_setup.sh'

echo "✅ Server setup initiated!"

# Step 5: Create systemd services
cat > create_services.sh << 'EOF'
#!/bin/bash
echo "⚙️ Creating systemd services..."

# HFT Ninja service
sudo tee /etc/systemd/system/hft-ninja.service > /dev/null << 'SERVICE'
[Unit]
Description=Solana HFT Ninja
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/solana-hft-ninja
ExecStart=/home/ubuntu/solana-hft-ninja/target/release/solana-hft-ninja
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
SERVICE

# Cerebro BFF service
sudo tee /etc/systemd/system/cerebro-bff.service > /dev/null << 'SERVICE'
[Unit]
Description=Cerebro BFF
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/solana-hft-ninja/cerebro/bff
ExecStart=/home/ubuntu/solana-hft-ninja/cerebro/venv/bin/python main.py
Restart=always
RestartSec=10
Environment=PYTHONPATH=/home/ubuntu/solana-hft-ninja/cerebro

[Install]
WantedBy=multi-user.target
SERVICE

# Cerebro Dashboard service (nginx will serve static files)
sudo tee /etc/systemd/system/cerebro-dashboard.service > /dev/null << 'SERVICE'
[Unit]
Description=Cerebro Dashboard
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/solana-hft-ninja/cerebro/dashboard
ExecStart=/usr/bin/npm run preview
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
SERVICE

# Enable and start services
sudo systemctl daemon-reload
sudo systemctl enable hft-ninja cerebro-bff cerebro-dashboard
sudo systemctl start hft-ninja cerebro-bff cerebro-dashboard

echo "✅ Services created and started!"
sudo systemctl status hft-ninja cerebro-bff cerebro-dashboard
EOF

echo "📤 Step 4: Creating services..."
scp -i $SSH_KEY create_services.sh $SERVER_USER@$SERVER_IP:~/
ssh -i $SSH_KEY $SERVER_USER@$SERVER_IP 'chmod +x create_services.sh && ./create_services.sh'

echo ""
echo "🎉 DEPLOYMENT COMPLETE!"
echo "======================"
echo "✅ HFT Ninja deployed and running"
echo "✅ Cerebro BFF deployed and running"  
echo "✅ Dashboard deployed and running"
echo ""
echo "🌐 Access your system:"
echo "Dashboard: http://$SERVER_IP:3000"
echo "BFF API: http://$SERVER_IP:8000"
echo "HFT Ninja: http://$SERVER_IP:3030"
echo ""
echo "🔧 Monitor services:"
echo "sudo systemctl status hft-ninja"
echo "sudo systemctl status cerebro-bff"
echo "sudo systemctl status cerebro-dashboard"
