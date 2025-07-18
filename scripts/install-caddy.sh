#!/bin/bash

# 🔧 Caddy Installation Script for Solana HFT Ninja
# Installs and configures Caddy v2 with automatic HTTPS and security features

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CADDY_VERSION="2.7.6"
DOMAIN="${DOMAIN:-hft-ninja.com}"
API_DOMAIN="${API_DOMAIN:-api.hft-ninja.com}"
EMAIL="${EMAIL:-admin@hft-ninja.com}"

echo -e "${BLUE}🔧 Installing Caddy v2 for Solana HFT Ninja${NC}"
echo -e "${GREEN}Domain: $DOMAIN${NC}"
echo -e "${GREEN}API Domain: $API_DOMAIN${NC}"
echo -e "${GREEN}Email: $EMAIL${NC}"
echo ""

# Function to print status
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    print_error "Please run as root (use sudo)"
    exit 1
fi

# Update system
print_status "Updating system packages..."
apt update && apt upgrade -y

# Install dependencies
print_status "Installing dependencies..."
apt install -y curl wget gnupg2 software-properties-common apt-transport-https ca-certificates

# Install Caddy
print_status "Installing Caddy v2..."

# Add Caddy repository
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list

# Update and install
apt update
apt install -y caddy

# Verify installation
INSTALLED_VERSION=$(caddy version | head -n1 | awk '{print $1}')
print_status "Caddy installed: $INSTALLED_VERSION"

# Create directories
print_status "Creating Caddy directories..."
mkdir -p /etc/caddy
mkdir -p /var/log/caddy
mkdir -p /var/lib/caddy
mkdir -p /etc/caddy/sites

# Set permissions
chown -R caddy:caddy /var/log/caddy
chown -R caddy:caddy /var/lib/caddy
chmod 755 /var/log/caddy
chmod 755 /var/lib/caddy

# Create Caddyfile from template
print_status "Creating Caddyfile configuration..."

cat > /etc/caddy/Caddyfile << EOF
# =========================================================================
#  🛡️ Caddy v2 Configuration for Solana HFT Ninja API Gateway
#  Auto-generated on $(date)
# =========================================================================

# Global options
{
    email $EMAIL
    admin localhost:2019
    
    log {
        output file /var/log/caddy/access.log {
            roll_size 100mb
            roll_keep 5
            roll_keep_for 720h
        }
        format json
        level INFO
    }
    
    experimental_http3
}

# =============================================================================
# 🚀 Main API Domain
# =============================================================================
$API_DOMAIN {
    tls {
        protocols tls1.2 tls1.3
    }
    
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
        X-XSS-Protection "1; mode=block"
        Content-Security-Policy "default-src 'none'; frame-ancestors 'none';"
        Referrer-Policy "strict-origin-when-cross-origin"
        -Server
        X-API-Version "2025.07"
        X-Powered-By "Solana-HFT-Ninja"
    }
    
    # AI Endpoints - High Security
    @ai_endpoints {
        path /ai/calculate/*
    }
    
    handle @ai_endpoints {
        rate_limit {
            zone ai_zone
            key {remote_ip}
            window 1m
            max_requests 10
            deny_status 429
        }
        
        reverse_proxy localhost:8003 {
            health_uri /health
            health_interval 30s
            health_timeout 10s
            
            header_up X-Real-IP {remote_ip}
            header_up X-Forwarded-For {remote_ip}
            header_up X-Forwarded-Proto {scheme}
            header_up X-Request-ID {uuid}
        }
    }
    
    # BFF API Endpoints
    @bff_endpoints {
        path /api/*
    }
    
    handle @bff_endpoints {
        rate_limit {
            zone api_zone
            key {remote_ip}
            window 1m
            max_requests 100
            deny_status 429
        }
        
        reverse_proxy localhost:8002 {
            health_uri /health
            health_interval 15s
            health_timeout 5s
            
            header_up X-Real-IP {remote_ip}
            header_up X-Forwarded-For {remote_ip}
            header_up X-Forwarded-Proto {scheme}
            header_up X-Request-ID {uuid}
        }
    }
    
    # Health Check
    @health_endpoints {
        path /health
        path /status
        path /ping
    }
    
    handle @health_endpoints {
        rate_limit {
            zone health_zone
            key {remote_ip}
            window 1m
            max_requests 60
            deny_status 429
        }
        
        reverse_proxy localhost:8002 {
            health_uri /health
        }
        
        handle_errors {
            respond \`{
                "status": "healthy",
                "service": "caddy-gateway",
                "timestamp": "{time.now.unix}",
                "version": "2025.07"
            }\` 200 {
                header Content-Type application/json
            }
        }
    }
    
    # Metrics (local only)
    @metrics {
        path /metrics
    }
    
    handle @metrics {
        @local_only {
            remote_ip 127.0.0.1/8 10.0.0.0/8 172.16.0.0/12 192.168.0.0/16
        }
        
        handle @local_only {
            metrics /metrics
        }
        
        handle {
            respond "Access denied" 403
        }
    }
    
    # Block malicious paths
    @blocked_paths {
        path /.env*
        path /.git*
        path /admin*
        path /wp-admin*
        path *.php
    }
    
    handle @blocked_paths {
        respond "Not found" 404
    }
    
    # Default 404
    handle {
        respond \`{
            "error": "Not Found",
            "message": "API endpoint not found"
        }\` 404 {
            header Content-Type application/json
        }
    }
}

# =============================================================================
# 🌐 Frontend Domain
# =============================================================================
$DOMAIN, www.$DOMAIN {
    tls {
        protocols tls1.2 tls1.3
    }
    
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "SAMEORIGIN"
        X-XSS-Protection "1; mode=block"
        -Server
    }
    
    rate_limit {
        zone frontend_zone
        key {remote_ip}
        window 1m
        max_requests 200
        deny_status 429
    }
    
    reverse_proxy localhost:3000 {
        health_uri /
        health_interval 30s
        health_timeout 5s
        
        header_up X-Real-IP {remote_ip}
        header_up X-Forwarded-For {remote_ip}
        header_up X-Forwarded-Proto {scheme}
    }
}
EOF

# Create systemd service
print_status "Creating systemd service..."

cat > /etc/systemd/system/caddy.service << EOF
[Unit]
Description=Caddy HTTP/2 web server
Documentation=https://caddyserver.com/docs/
After=network.target network-online.target
Requires=network-online.target

[Service]
Type=notify
User=caddy
Group=caddy
ExecStart=/usr/bin/caddy run --environ --config /etc/caddy/Caddyfile
ExecReload=/usr/bin/caddy reload --config /etc/caddy/Caddyfile --force
TimeoutStopSec=5s
LimitNOFILE=1048576
LimitNPROC=1048576
PrivateTmp=true
ProtectSystem=full
AmbientCapabilities=CAP_NET_BIND_SERVICE

[Install]
WantedBy=multi-user.target
EOF

# Create log rotation
print_status "Setting up log rotation..."

cat > /etc/logrotate.d/caddy << EOF
/var/log/caddy/*.log {
    daily
    missingok
    rotate 52
    compress
    delaycompress
    notifempty
    create 644 caddy caddy
    postrotate
        systemctl reload caddy
    endscript
}
EOF

# Create health check script
print_status "Creating health check script..."

cat > /usr/local/bin/caddy-health-check.sh << 'EOF'
#!/bin/bash

# Caddy Health Check Script
API_ENDPOINT="https://api.hft-ninja.com/health"
FRONTEND_ENDPOINT="https://hft-ninja.com"
LOG_FILE="/var/log/caddy/health-check.log"

check_endpoint() {
    local url=$1
    local name=$2
    
    response=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "$url")
    
    if [ "$response" = "200" ]; then
        echo "$(date): $name OK (HTTP $response)" >> "$LOG_FILE"
        return 0
    else
        echo "$(date): $name FAILED (HTTP $response)" >> "$LOG_FILE"
        return 1
    fi
}

# Check API endpoint
if ! check_endpoint "$API_ENDPOINT" "API"; then
    echo "API health check failed"
    exit 1
fi

# Check frontend
if ! check_endpoint "$FRONTEND_ENDPOINT" "Frontend"; then
    echo "Frontend health check failed"
    exit 1
fi

echo "All health checks passed"
EOF

chmod +x /usr/local/bin/caddy-health-check.sh

# Test configuration
print_status "Testing Caddy configuration..."
if caddy validate --config /etc/caddy/Caddyfile; then
    print_status "✅ Caddy configuration is valid"
else
    print_error "❌ Caddy configuration is invalid"
    exit 1
fi

# Enable and start service
print_status "Enabling and starting Caddy service..."
systemctl daemon-reload
systemctl enable caddy
systemctl start caddy

# Wait for service to start
sleep 3

# Check service status
if systemctl is-active --quiet caddy; then
    print_status "✅ Caddy service is running"
else
    print_error "❌ Caddy service failed to start"
    systemctl status caddy
    exit 1
fi

# Create monitoring cron job
print_status "Setting up monitoring cron job..."
(crontab -l 2>/dev/null; echo "*/5 * * * * /usr/local/bin/caddy-health-check.sh") | crontab -

# Display final information
echo ""
echo -e "${GREEN}🎉 Caddy installation completed successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Configuration Summary:${NC}"
echo "  • Caddy Version: $(caddy version | head -n1)"
echo "  • Configuration: /etc/caddy/Caddyfile"
echo "  • Logs: /var/log/caddy/"
echo "  • Service: systemctl status caddy"
echo ""
echo -e "${BLUE}🌐 Domains Configured:${NC}"
echo "  • API: https://$API_DOMAIN"
echo "  • Frontend: https://$DOMAIN"
echo "  • Admin: http://localhost:2019"
echo ""
echo -e "${BLUE}🔧 Management Commands:${NC}"
echo "  • Reload config: systemctl reload caddy"
echo "  • View logs: journalctl -u caddy -f"
echo "  • Test config: caddy validate --config /etc/caddy/Caddyfile"
echo "  • Health check: /usr/local/bin/caddy-health-check.sh"
echo ""
echo -e "${YELLOW}⚠️  Next Steps:${NC}"
echo "  1. Update DNS records to point to this server"
echo "  2. Ensure HFT Ninja services are running on localhost:8002 and localhost:8003"
echo "  3. Test endpoints: curl https://$API_DOMAIN/health"
echo "  4. Monitor logs: tail -f /var/log/caddy/access.log"
echo ""
echo -e "${GREEN}🛡️ Your API is now protected by Caddy reverse proxy!${NC}"
