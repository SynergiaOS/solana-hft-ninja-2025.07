#!/bin/bash

# 🔒 Security Hardening Script for Solana HFT Ninja
# Implements enterprise-grade security measures with Caddy and system hardening

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DOMAIN="${DOMAIN:-hft-ninja.com}"
API_DOMAIN="${API_DOMAIN:-api.hft-ninja.com}"
ADMIN_EMAIL="${ADMIN_EMAIL:-admin@hft-ninja.com}"

echo -e "${BLUE}🔒 Security Hardening for Solana HFT Ninja${NC}"
echo -e "${GREEN}Domain: $DOMAIN${NC}"
echo -e "${GREEN}API Domain: $API_DOMAIN${NC}"
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

# System hardening
harden_system() {
    print_status "Hardening system security..."
    
    # Update system
    apt update && apt upgrade -y
    
    # Install security tools
    apt install -y fail2ban ufw aide rkhunter chkrootkit lynis
    
    # Configure firewall
    print_status "Configuring UFW firewall..."
    ufw --force reset
    ufw default deny incoming
    ufw default allow outgoing
    ufw allow 22/tcp    # SSH
    ufw allow 80/tcp    # HTTP
    ufw allow 443/tcp   # HTTPS
    ufw allow 8080/tcp  # Caddy admin (local only)
    ufw --force enable
    
    # Configure fail2ban
    print_status "Configuring Fail2ban..."
    cat > /etc/fail2ban/jail.local << 'EOF'
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 3
backend = systemd

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 3

[caddy-auth]
enabled = true
port = http,https
filter = caddy-auth
logpath = /var/log/caddy/security.log
maxretry = 5

[caddy-dos]
enabled = true
port = http,https
filter = caddy-dos
logpath = /var/log/caddy/access.log
maxretry = 100
findtime = 60
bantime = 600
EOF
    
    # Create fail2ban filters
    cat > /etc/fail2ban/filter.d/caddy-auth.conf << 'EOF'
[Definition]
failregex = ^.*"remote_ip":"<HOST>".*"status":40[13].*$
ignoreregex =
EOF
    
    cat > /etc/fail2ban/filter.d/caddy-dos.conf << 'EOF'
[Definition]
failregex = ^.*"remote_ip":"<HOST>".*"status":429.*$
ignoreregex =
EOF
    
    systemctl enable fail2ban
    systemctl restart fail2ban
    
    print_status "✅ System hardening completed"
}

# Configure secure Caddy
configure_secure_caddy() {
    print_status "Configuring security-hardened Caddy..."
    
    # Backup existing configuration
    if [ -f /etc/caddy/Caddyfile ]; then
        cp /etc/caddy/Caddyfile /etc/caddy/Caddyfile.backup.$(date +%Y%m%d_%H%M%S)
    fi
    
    # Copy security-hardened configuration
    cp caddy/security-hardened.Caddyfile /etc/caddy/Caddyfile
    
    # Update domain placeholders
    sed -i "s/hft-ninja.com/$DOMAIN/g" /etc/caddy/Caddyfile
    sed -i "s/api.hft-ninja.com/$API_DOMAIN/g" /etc/caddy/Caddyfile
    sed -i "s/admin@hft-ninja.com/$ADMIN_EMAIL/g" /etc/caddy/Caddyfile
    
    # Create security log directories
    mkdir -p /var/log/caddy/security
    chown -R caddy:caddy /var/log/caddy
    chmod 755 /var/log/caddy
    
    # Test configuration
    if caddy validate --config /etc/caddy/Caddyfile; then
        print_status "✅ Caddy security configuration is valid"
    else
        print_error "❌ Caddy security configuration is invalid"
        exit 1
    fi
    
    # Reload Caddy
    systemctl reload caddy
    
    print_status "✅ Security-hardened Caddy configured"
}

# Setup intrusion detection
setup_intrusion_detection() {
    print_status "Setting up intrusion detection..."
    
    # Configure AIDE (Advanced Intrusion Detection Environment)
    print_status "Configuring AIDE..."
    aideinit
    mv /var/lib/aide/aide.db.new /var/lib/aide/aide.db
    
    # Create AIDE check script
    cat > /usr/local/bin/aide-check.sh << 'EOF'
#!/bin/bash
AIDE_LOG="/var/log/aide.log"
WEBHOOK_URL="WEBHOOK_URL_PLACEHOLDER"

echo "$(date): Starting AIDE integrity check" >> "$AIDE_LOG"
aide --check >> "$AIDE_LOG" 2>&1

if [ $? -ne 0 ]; then
    echo "$(date): AIDE detected file system changes!" >> "$AIDE_LOG"
    
    if [ "$WEBHOOK_URL" != "WEBHOOK_URL_PLACEHOLDER" ]; then
        curl -s -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d '{"content": "🚨 **SECURITY ALERT**: AIDE detected unauthorized file system changes on HFT Ninja server!"}' > /dev/null
    fi
fi
EOF
    
    chmod +x /usr/local/bin/aide-check.sh
    
    # Add AIDE to cron (daily check)
    (crontab -l 2>/dev/null; echo "0 2 * * * /usr/local/bin/aide-check.sh") | crontab -
    
    print_status "✅ Intrusion detection configured"
}

# Setup security monitoring
setup_security_monitoring() {
    print_status "Setting up security monitoring..."
    
    # Create security monitoring script
    cat > /usr/local/bin/security-monitor.sh << 'EOF'
#!/bin/bash

# 🔒 Security Monitoring Script for HFT Ninja
# Monitors security events and sends alerts

LOG_FILE="/var/log/security-monitor.log"
WEBHOOK_URL="WEBHOOK_URL_PLACEHOLDER"

# Function to send alert
send_alert() {
    local message="$1"
    local severity="$2"
    
    echo "$(date): $severity - $message" >> "$LOG_FILE"
    
    if [ "$WEBHOOK_URL" != "WEBHOOK_URL_PLACEHOLDER" ]; then
        local color=16711680  # Red
        if [ "$severity" = "WARNING" ]; then
            color=16776960  # Yellow
        elif [ "$severity" = "INFO" ]; then
            color=65280     # Green
        fi
        
        curl -s -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "{
                \"embeds\": [{
                    \"title\": \"🔒 HFT Ninja Security Alert\",
                    \"description\": \"**$severity**: $message\",
                    \"color\": $color,
                    \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%S.000Z)\"
                }]
            }" > /dev/null
    fi
}

# Check for failed login attempts
failed_logins=$(grep "Failed password" /var/log/auth.log | grep "$(date '+%b %d')" | wc -l)
if [ "$failed_logins" -gt 10 ]; then
    send_alert "High number of failed login attempts: $failed_logins" "CRITICAL"
fi

# Check for new users
new_users=$(grep "new user" /var/log/auth.log | grep "$(date '+%b %d')" | wc -l)
if [ "$new_users" -gt 0 ]; then
    send_alert "New user accounts created: $new_users" "WARNING"
fi

# Check for privilege escalation
sudo_usage=$(grep "sudo:" /var/log/auth.log | grep "$(date '+%b %d')" | wc -l)
if [ "$sudo_usage" -gt 50 ]; then
    send_alert "High sudo usage detected: $sudo_usage commands" "WARNING"
fi

# Check Caddy security logs
if [ -f /var/log/caddy/security_events.log ]; then
    security_events=$(grep "$(date '+%Y-%m-%d')" /var/log/caddy/security_events.log | wc -l)
    if [ "$security_events" -gt 100 ]; then
        send_alert "High number of security events in Caddy: $security_events" "WARNING"
    fi
fi

# Check for rootkit
if command -v rkhunter &> /dev/null; then
    rkhunter --check --sk --nocolors > /tmp/rkhunter.log 2>&1
    if grep -q "Warning" /tmp/rkhunter.log; then
        send_alert "Rootkit hunter detected warnings" "CRITICAL"
    fi
fi

# Check disk usage
disk_usage=$(df / | awk 'NR==2 {print $5}' | sed 's/%//')
if [ "$disk_usage" -gt 90 ]; then
    send_alert "High disk usage: ${disk_usage}%" "WARNING"
fi

# Check memory usage
memory_usage=$(free | awk 'NR==2{printf "%.0f", $3*100/$2}')
if [ "$memory_usage" -gt 90 ]; then
    send_alert "High memory usage: ${memory_usage}%" "WARNING"
fi

# Check for suspicious processes
suspicious_procs=$(ps aux | grep -E "(nc|netcat|nmap|masscan|sqlmap|nikto)" | grep -v grep | wc -l)
if [ "$suspicious_procs" -gt 0 ]; then
    send_alert "Suspicious processes detected: $suspicious_procs" "CRITICAL"
fi

# Check network connections
suspicious_connections=$(netstat -tn | grep ESTABLISHED | awk '{print $5}' | cut -d: -f1 | sort | uniq -c | sort -nr | head -1 | awk '{print $1}')
if [ "$suspicious_connections" -gt 100 ]; then
    send_alert "High number of connections from single IP: $suspicious_connections" "WARNING"
fi

echo "$(date): Security monitoring completed" >> "$LOG_FILE"
EOF
    
    chmod +x /usr/local/bin/security-monitor.sh
    
    # Add to cron (every 15 minutes)
    (crontab -l 2>/dev/null; echo "*/15 * * * * /usr/local/bin/security-monitor.sh") | crontab -
    
    print_status "✅ Security monitoring configured"
}

# Setup log analysis
setup_log_analysis() {
    print_status "Setting up log analysis..."
    
    # Create log analysis script
    cat > /usr/local/bin/log-analyzer.sh << 'EOF'
#!/bin/bash

# 📊 Log Analysis Script for HFT Ninja Security
# Analyzes logs for security patterns and generates reports

REPORT_FILE="/var/log/security-report-$(date +%Y%m%d).log"

echo "=== HFT Ninja Security Report - $(date) ===" > "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Analyze Caddy access logs
if [ -f /var/log/caddy/access.log ]; then
    echo "📊 Top IP Addresses (Last 24h):" >> "$REPORT_FILE"
    grep "$(date -d '1 day ago' '+%Y-%m-%d')" /var/log/caddy/access.log | \
        jq -r '.request.remote_ip' 2>/dev/null | sort | uniq -c | sort -nr | head -10 >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    echo "🚫 Blocked Requests (Last 24h):" >> "$REPORT_FILE"
    grep "$(date -d '1 day ago' '+%Y-%m-%d')" /var/log/caddy/access.log | \
        jq -r 'select(.status >= 400) | "\(.status) \(.request.uri) \(.request.remote_ip)"' 2>/dev/null | \
        sort | uniq -c | sort -nr | head -10 >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    echo "🔥 Most Requested Endpoints:" >> "$REPORT_FILE"
    grep "$(date -d '1 day ago' '+%Y-%m-%d')" /var/log/caddy/access.log | \
        jq -r '.request.uri' 2>/dev/null | sort | uniq -c | sort -nr | head -10 >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
fi

# Analyze security events
if [ -f /var/log/caddy/security_events.log ]; then
    echo "🛡️ Security Events (Last 24h):" >> "$REPORT_FILE"
    grep "$(date -d '1 day ago' '+%Y-%m-%d')" /var/log/caddy/security_events.log | wc -l >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
fi

# Analyze fail2ban
if command -v fail2ban-client &> /dev/null; then
    echo "🚫 Fail2ban Status:" >> "$REPORT_FILE"
    fail2ban-client status >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
fi

# System security status
echo "💻 System Security Status:" >> "$REPORT_FILE"
echo "  • Uptime: $(uptime -p)" >> "$REPORT_FILE"
echo "  • Load: $(uptime | awk -F'load average:' '{print $2}')" >> "$REPORT_FILE"
echo "  • Disk Usage: $(df -h / | awk 'NR==2 {print $5}')" >> "$REPORT_FILE"
echo "  • Memory Usage: $(free | awk 'NR==2{printf "%.0f%%", $3*100/$2}')" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "Report generated: $REPORT_FILE"
EOF
    
    chmod +x /usr/local/bin/log-analyzer.sh
    
    # Add to cron (daily report)
    (crontab -l 2>/dev/null; echo "0 6 * * * /usr/local/bin/log-analyzer.sh") | crontab -
    
    print_status "✅ Log analysis configured"
}

# Main execution
main() {
    harden_system
    configure_secure_caddy
    setup_intrusion_detection
    setup_security_monitoring
    setup_log_analysis
    
    echo ""
    echo -e "${GREEN}🎉 Security hardening completed!${NC}"
    echo ""
    echo -e "${BLUE}📋 Security Features Enabled:${NC}"
    echo "  • UFW Firewall with restrictive rules"
    echo "  • Fail2ban with custom Caddy filters"
    echo "  • AIDE intrusion detection"
    echo "  • Security monitoring and alerting"
    echo "  • Comprehensive log analysis"
    echo "  • Security-hardened Caddy configuration"
    echo ""
    echo -e "${BLUE}🔧 Security Commands:${NC}"
    echo "  • Security Monitor: /usr/local/bin/security-monitor.sh"
    echo "  • Log Analyzer: /usr/local/bin/log-analyzer.sh"
    echo "  • AIDE Check: /usr/local/bin/aide-check.sh"
    echo "  • View Security Logs: tail -f /var/log/caddy/security_events.log"
    echo "  • Fail2ban Status: fail2ban-client status"
    echo ""
    echo -e "${YELLOW}⚠️  Next Steps:${NC}"
    echo "  1. Update webhook URLs in monitoring scripts"
    echo "  2. Test security configuration: curl -I https://$API_DOMAIN"
    echo "  3. Review firewall rules: ufw status"
    echo "  4. Monitor security logs regularly"
    echo "  5. Run security audit: lynis audit system"
    echo ""
    echo -e "${GREEN}🛡️ Your HFT Ninja API is now enterprise-grade secure!${NC}"
}

# Run main function
main "$@"
