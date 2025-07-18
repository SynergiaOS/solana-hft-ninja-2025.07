#!/bin/bash

# 📊 Cloudflare Analytics Integration for Solana HFT Ninja
# Setup analytics, monitoring, and health checks with Cloudflare API

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
CF_TOKEN="${CF_TOKEN:-your_cloudflare_token}"
CF_ZONE_ID="${CF_ZONE_ID:-your_zone_id}"
WEBHOOK_URL="${WEBHOOK_URL:-your_discord_webhook_url}"

echo -e "${BLUE}📊 Setting up Cloudflare Analytics for Solana HFT Ninja${NC}"
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

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    for cmd in curl jq; do
        if ! command -v $cmd &> /dev/null; then
            print_error "$cmd is not installed. Please install it first."
            exit 1
        fi
    done
    
    print_status "✅ All dependencies are available"
}

# Get Cloudflare Zone ID
get_zone_id() {
    if [ "$CF_ZONE_ID" = "your_zone_id" ]; then
        print_status "Getting Zone ID for $DOMAIN..."
        
        CF_ZONE_ID=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones?name=$DOMAIN" \
            -H "Authorization: Bearer $CF_TOKEN" \
            -H "Content-Type: application/json" | \
            jq -r '.result[0].id')
        
        if [ "$CF_ZONE_ID" = "null" ] || [ -z "$CF_ZONE_ID" ]; then
            print_error "Could not get Zone ID. Please check your domain and token."
            exit 1
        fi
        
        print_status "✅ Zone ID: $CF_ZONE_ID"
    fi
}

# Create Cloudflare Analytics Rules
create_analytics_rules() {
    print_status "Creating Cloudflare Analytics rules..."
    
    # Rule 1: Track API Health Checks
    print_status "Creating health check tracking rule..."
    curl -s -X POST "https://api.cloudflare.com/client/v4/zones/$CF_ZONE_ID/rulesets/phases/http_request_firewall_custom" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        --data '{
            "name": "HFT-Ninja-Health-Tracking",
            "description": "Track health check requests for monitoring",
            "kind": "zone",
            "phase": "http_request_firewall_custom",
            "rules": [
                {
                    "action": "log",
                    "expression": "(http.host eq \"'$API_DOMAIN'\" and http.request.uri.path eq \"/health\")",
                    "description": "Log health check requests",
                    "enabled": true
                }
            ]
        }' > /dev/null
    
    # Rule 2: Track AI Calculation Requests
    print_status "Creating AI calculation tracking rule..."
    curl -s -X POST "https://api.cloudflare.com/client/v4/zones/$CF_ZONE_ID/rulesets/phases/http_request_firewall_custom" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        --data '{
            "name": "HFT-Ninja-AI-Tracking",
            "description": "Track AI calculation requests",
            "kind": "zone",
            "phase": "http_request_firewall_custom",
            "rules": [
                {
                    "action": "log",
                    "expression": "(http.host eq \"'$API_DOMAIN'\" and http.request.uri.path contains \"/ai/calculate\")",
                    "description": "Log AI calculation requests",
                    "enabled": true
                }
            ]
        }' > /dev/null
    
    # Rule 3: Track Error Responses
    print_status "Creating error tracking rule..."
    curl -s -X POST "https://api.cloudflare.com/client/v4/zones/$CF_ZONE_ID/rulesets/phases/http_response_firewall_custom" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        --data '{
            "name": "HFT-Ninja-Error-Tracking",
            "description": "Track error responses for monitoring",
            "kind": "zone",
            "phase": "http_response_firewall_custom",
            "rules": [
                {
                    "action": "log",
                    "expression": "(http.host eq \"'$API_DOMAIN'\" and http.response.code ge 400)",
                    "description": "Log error responses",
                    "enabled": true
                }
            ]
        }' > /dev/null
    
    print_status "✅ Analytics rules created"
}

# Create health monitoring script
create_health_monitor() {
    print_status "Creating health monitoring script..."
    
    cat > /usr/local/bin/hft-ninja-monitor.sh << 'EOF'
#!/bin/bash

# 🏥 HFT Ninja Health Monitor with Cloudflare Analytics
# Monitors API health and sends alerts via webhook

# Configuration
API_ENDPOINTS=(
    "https://api.hft-ninja.com/health:API Health"
    "https://api.hft-ninja.com/ai/health:AI Health"
    "https://hft-ninja.com:Frontend"
)

LOG_FILE="/var/log/hft-ninja-monitor.log"
STATUS_FILE="/tmp/hft-ninja-status.json"
WEBHOOK_URL="WEBHOOK_URL_PLACEHOLDER"
CF_TOKEN="CF_TOKEN_PLACEHOLDER"
CF_ZONE_ID="CF_ZONE_ID_PLACEHOLDER"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Function to log with timestamp
log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

# Function to send webhook notification
send_webhook() {
    local message="$1"
    local color="$2"
    
    if [ "$WEBHOOK_URL" != "WEBHOOK_URL_PLACEHOLDER" ]; then
        curl -s -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "{
                \"embeds\": [{
                    \"title\": \"🥷 HFT Ninja Alert\",
                    \"description\": \"$message\",
                    \"color\": $color,
                    \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%S.000Z)\"
                }]
            }" > /dev/null
    fi
}

# Function to get Cloudflare analytics
get_cf_analytics() {
    if [ "$CF_TOKEN" != "CF_TOKEN_PLACEHOLDER" ]; then
        local since=$(date -d '1 hour ago' -u +%Y-%m-%dT%H:%M:%SZ)
        local until=$(date -u +%Y-%m-%dT%H:%M:%SZ)
        
        curl -s -X GET "https://api.cloudflare.com/client/v4/zones/$CF_ZONE_ID/analytics/dashboard" \
            -H "Authorization: Bearer $CF_TOKEN" \
            -H "Content-Type: application/json" \
            -G -d "since=$since" -d "until=$until" | \
            jq -r '.result.totals | "Requests: \(.requests.all), Bandwidth: \(.bandwidth.all), Threats: \(.threats.all)"'
    fi
}

# Function to check endpoint health
check_endpoint() {
    local url="$1"
    local name="$2"
    local timeout=10
    
    local start_time=$(date +%s%N)
    local response=$(curl -s -o /dev/null -w "%{http_code}:%{time_total}" --max-time $timeout "$url" 2>/dev/null)
    local end_time=$(date +%s%N)
    
    local http_code=$(echo "$response" | cut -d: -f1)
    local response_time=$(echo "$response" | cut -d: -f2)
    local total_time=$(( (end_time - start_time) / 1000000 ))
    
    if [ "$http_code" = "200" ]; then
        log_message "✅ $name: OK (HTTP $http_code, ${response_time}s)"
        echo "{\"name\": \"$name\", \"status\": \"healthy\", \"http_code\": $http_code, \"response_time\": $response_time, \"timestamp\": $(date +%s)}"
        return 0
    else
        log_message "❌ $name: FAILED (HTTP $http_code, ${response_time}s)"
        echo "{\"name\": \"$name\", \"status\": \"unhealthy\", \"http_code\": $http_code, \"response_time\": $response_time, \"timestamp\": $(date +%s)}"
        return 1
    fi
}

# Main monitoring function
main() {
    log_message "🔍 Starting health check cycle"
    
    local failed_services=0
    local total_services=0
    local status_data="["
    
    for endpoint in "${API_ENDPOINTS[@]}"; do
        IFS=':' read -r url name <<< "$endpoint"
        total_services=$((total_services + 1))
        
        local result=$(check_endpoint "$url" "$name")
        
        if [ $total_services -gt 1 ]; then
            status_data="$status_data,"
        fi
        status_data="$status_data$result"
        
        if ! echo "$result" | grep -q '"status": "healthy"'; then
            failed_services=$((failed_services + 1))
        fi
    done
    
    status_data="$status_data]"
    
    # Save status to file
    echo "$status_data" > "$STATUS_FILE"
    
    # Get Cloudflare analytics
    local cf_stats=$(get_cf_analytics)
    if [ -n "$cf_stats" ]; then
        log_message "📊 Cloudflare Stats: $cf_stats"
    fi
    
    # Send alerts if needed
    if [ $failed_services -gt 0 ]; then
        local message="⚠️ $failed_services/$total_services services are unhealthy!"
        log_message "$message"
        send_webhook "$message" 16711680  # Red color
    elif [ $total_services -gt 0 ]; then
        log_message "✅ All $total_services services are healthy"
        
        # Send success notification every hour
        local minute=$(date +%M)
        if [ "$minute" = "00" ]; then
            send_webhook "✅ All HFT Ninja services are healthy. $cf_stats" 65280  # Green color
        fi
    fi
    
    log_message "🏁 Health check cycle completed"
}

# Run main function
main "$@"
EOF
    
    # Replace placeholders
    sed -i "s/WEBHOOK_URL_PLACEHOLDER/$WEBHOOK_URL/g" /usr/local/bin/hft-ninja-monitor.sh
    sed -i "s/CF_TOKEN_PLACEHOLDER/$CF_TOKEN/g" /usr/local/bin/hft-ninja-monitor.sh
    sed -i "s/CF_ZONE_ID_PLACEHOLDER/$CF_ZONE_ID/g" /usr/local/bin/hft-ninja-monitor.sh
    
    chmod +x /usr/local/bin/hft-ninja-monitor.sh
    
    print_status "✅ Health monitor created"
}

# Create analytics dashboard script
create_analytics_dashboard() {
    print_status "Creating analytics dashboard script..."
    
    cat > /usr/local/bin/hft-ninja-analytics.sh << 'EOF'
#!/bin/bash

# 📊 HFT Ninja Analytics Dashboard
# Display real-time analytics from Cloudflare and local monitoring

CF_TOKEN="CF_TOKEN_PLACEHOLDER"
CF_ZONE_ID="CF_ZONE_ID_PLACEHOLDER"
STATUS_FILE="/tmp/hft-ninja-status.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Function to get Cloudflare analytics
get_analytics() {
    local period="$1"
    local since until
    
    case $period in
        "1h")
            since=$(date -d '1 hour ago' -u +%Y-%m-%dT%H:%M:%SZ)
            ;;
        "24h")
            since=$(date -d '24 hours ago' -u +%Y-%m-%dT%H:%M:%SZ)
            ;;
        "7d")
            since=$(date -d '7 days ago' -u +%Y-%m-%dT%H:%M:%SZ)
            ;;
        *)
            since=$(date -d '1 hour ago' -u +%Y-%m-%dT%H:%M:%SZ)
            ;;
    esac
    
    until=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    
    curl -s -X GET "https://api.cloudflare.com/client/v4/zones/$CF_ZONE_ID/analytics/dashboard" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        -G -d "since=$since" -d "until=$until"
}

# Function to display dashboard
display_dashboard() {
    clear
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                🥷 HFT Ninja Analytics Dashboard              ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Service Status
    echo -e "${YELLOW}📊 Service Status:${NC}"
    if [ -f "$STATUS_FILE" ]; then
        jq -r '.[] | "  • \(.name): \(if .status == "healthy" then "✅" else "❌" end) \(.status) (HTTP \(.http_code), \(.response_time)s)"' "$STATUS_FILE" 2>/dev/null || echo "  No status data available"
    else
        echo "  No status data available"
    fi
    echo ""
    
    # Cloudflare Analytics
    echo -e "${YELLOW}🌐 Cloudflare Analytics (Last 24h):${NC}"
    local analytics=$(get_analytics "24h")
    
    if [ -n "$analytics" ] && echo "$analytics" | jq -e '.success' > /dev/null 2>&1; then
        echo "$analytics" | jq -r '
            .result.totals |
            "  • Total Requests: \(.requests.all // 0)",
            "  • Cached Requests: \(.requests.cached // 0)",
            "  • Bandwidth: \((.bandwidth.all // 0) / 1024 / 1024 | floor)MB",
            "  • Unique Visitors: \(.uniques.all // 0)",
            "  • Threats Blocked: \(.threats.all // 0)",
            "  • Page Views: \(.pageviews.all // 0)"
        '
    else
        echo "  Analytics data not available"
    fi
    echo ""
    
    # Response Codes
    echo -e "${YELLOW}📈 Response Codes (Last 24h):${NC}"
    if [ -n "$analytics" ] && echo "$analytics" | jq -e '.success' > /dev/null 2>&1; then
        echo "$analytics" | jq -r '
            .result.totals.requests |
            "  • 2xx Success: \(.["200"] // 0)",
            "  • 3xx Redirect: \(.["300"] // 0)",
            "  • 4xx Client Error: \(.["400"] // 0)",
            "  • 5xx Server Error: \(.["500"] // 0)"
        '
    else
        echo "  Response code data not available"
    fi
    echo ""
    
    # System Info
    echo -e "${YELLOW}💻 System Information:${NC}"
    echo "  • Uptime: $(uptime -p)"
    echo "  • Load Average: $(uptime | awk -F'load average:' '{print $2}')"
    echo "  • Memory Usage: $(free -h | awk '/^Mem:/ {print $3"/"$2}')"
    echo "  • Disk Usage: $(df -h / | awk 'NR==2 {print $3"/"$2" ("$5")"}')"
    echo ""
    
    echo -e "${GREEN}Last updated: $(date)${NC}"
    echo -e "${BLUE}Press Ctrl+C to exit, or wait 30 seconds for refresh...${NC}"
}

# Main function
main() {
    if [ "$1" = "--once" ]; then
        display_dashboard
    else
        while true; do
            display_dashboard
            sleep 30
        done
    fi
}

main "$@"
EOF
    
    # Replace placeholders
    sed -i "s/CF_TOKEN_PLACEHOLDER/$CF_TOKEN/g" /usr/local/bin/hft-ninja-analytics.sh
    sed -i "s/CF_ZONE_ID_PLACEHOLDER/$CF_ZONE_ID/g" /usr/local/bin/hft-ninja-analytics.sh
    
    chmod +x /usr/local/bin/hft-ninja-analytics.sh
    
    print_status "✅ Analytics dashboard created"
}

# Setup cron jobs
setup_cron_jobs() {
    print_status "Setting up cron jobs..."
    
    # Add monitoring cron job (every 5 minutes)
    (crontab -l 2>/dev/null; echo "*/5 * * * * /usr/local/bin/hft-ninja-monitor.sh") | crontab -
    
    # Add log rotation
    cat > /etc/logrotate.d/hft-ninja-monitor << 'EOF'
/var/log/hft-ninja-monitor.log {
    daily
    missingok
    rotate 7
    compress
    delaycompress
    notifempty
    create 644 root root
}
EOF
    
    print_status "✅ Cron jobs configured"
}

# Main execution
main() {
    check_dependencies
    get_zone_id
    create_analytics_rules
    create_health_monitor
    create_analytics_dashboard
    setup_cron_jobs
    
    echo ""
    echo -e "${GREEN}🎉 Cloudflare Analytics integration completed!${NC}"
    echo ""
    echo -e "${BLUE}📋 Available Commands:${NC}"
    echo "  • Health Monitor: /usr/local/bin/hft-ninja-monitor.sh"
    echo "  • Analytics Dashboard: /usr/local/bin/hft-ninja-analytics.sh"
    echo "  • View Logs: tail -f /var/log/hft-ninja-monitor.log"
    echo ""
    echo -e "${BLUE}🔧 Configuration:${NC}"
    echo "  • Zone ID: $CF_ZONE_ID"
    echo "  • Domain: $DOMAIN"
    echo "  • API Domain: $API_DOMAIN"
    echo ""
    echo -e "${YELLOW}⚠️  Next Steps:${NC}"
    echo "  1. Update CF_TOKEN and WEBHOOK_URL in the scripts"
    echo "  2. Test monitoring: /usr/local/bin/hft-ninja-monitor.sh"
    echo "  3. View dashboard: /usr/local/bin/hft-ninja-analytics.sh --once"
    echo "  4. Check Cloudflare dashboard for analytics rules"
    echo ""
    echo -e "${GREEN}📊 Your API now has comprehensive monitoring and analytics!${NC}"
}

# Run main function
main "$@"
