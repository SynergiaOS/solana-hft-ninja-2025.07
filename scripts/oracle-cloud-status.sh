#!/bin/bash

# Oracle Cloud Status Check for Solana HFT Ninja
# Quick diagnostics for 10.0.0.59 deployment

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo -e "${BLUE}🌐 Oracle Cloud Status Check - Solana HFT Ninja${NC}"
    echo "=================================================="
    echo "📍 Target: 10.0.0.59 (subnet07161247.vcn07161247.oraclevcn.com)"
    echo ""
}

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

# Check Oracle Cloud metadata
check_oracle_metadata() {
    print_info "Checking Oracle Cloud metadata..."
    
    if curl -s --connect-timeout 2 http://169.254.169.254/opc/v2/instance/ > /dev/null 2>&1; then
        INSTANCE_ID=$(curl -s http://169.254.169.254/opc/v2/instance/id)
        INSTANCE_NAME=$(curl -s http://169.254.169.254/opc/v2/instance/displayName)
        INSTANCE_SHAPE=$(curl -s http://169.254.169.254/opc/v2/instance/shape)
        REGION=$(curl -s http://169.254.169.254/opc/v2/instance/region)
        
        print_status "Oracle Cloud Instance Detected"
        echo "  Instance ID: $INSTANCE_ID"
        echo "  Instance Name: $INSTANCE_NAME"
        echo "  Shape: $INSTANCE_SHAPE"
        echo "  Region: $REGION"
    else
        print_warning "Oracle Cloud metadata not accessible"
    fi
    echo ""
}

# Check network configuration
check_network() {
    print_info "Checking network configuration..."
    
    # Check IP configuration
    IP_ADDRESS=$(ip route get 1 | awk '{print $7; exit}')
    if [ "$IP_ADDRESS" = "10.0.0.59" ]; then
        print_status "Correct IP address: $IP_ADDRESS"
    else
        print_warning "IP address mismatch. Expected: 10.0.0.59, Got: $IP_ADDRESS"
    fi
    
    # Check DNS resolution
    if nslookup subnet07161247.vcn07161247.oraclevcn.com > /dev/null 2>&1; then
        print_status "DNS resolution working"
    else
        print_warning "DNS resolution issues"
    fi
    
    # Check internet connectivity
    if ping -c 1 8.8.8.8 > /dev/null 2>&1; then
        print_status "Internet connectivity OK"
    else
        print_error "No internet connectivity"
    fi
    
    echo ""
}

# Check HFT application status
check_hft_application() {
    print_info "Checking HFT application status..."
    
    # Check if process is running
    if pgrep -f hft_main > /dev/null; then
        print_status "HFT application is running"
        PID=$(pgrep -f hft_main)
        echo "  Process ID: $PID"
        
        # Check CPU and memory usage
        CPU_USAGE=$(ps -p $PID -o %cpu --no-headers | tr -d ' ')
        MEM_USAGE=$(ps -p $PID -o %mem --no-headers | tr -d ' ')
        echo "  CPU Usage: ${CPU_USAGE}%"
        echo "  Memory Usage: ${MEM_USAGE}%"
    else
        print_error "HFT application not running"
    fi
    
    # Check systemd service
    if systemctl is-active --quiet solana-hft-ninja 2>/dev/null; then
        print_status "Systemd service active"
    else
        print_warning "Systemd service not active"
    fi
    
    echo ""
}

# Check application endpoints
check_endpoints() {
    print_info "Checking application endpoints..."
    
    # Health check
    if curl -s --connect-timeout 5 http://localhost:8080/health > /dev/null 2>&1; then
        print_status "Health endpoint responding"
        HEALTH_STATUS=$(curl -s http://localhost:8080/health | jq -r '.status' 2>/dev/null || echo "unknown")
        echo "  Health Status: $HEALTH_STATUS"
    else
        print_error "Health endpoint not responding"
    fi
    
    # Metrics endpoint
    if curl -s --connect-timeout 5 http://localhost:8080/metrics > /dev/null 2>&1; then
        print_status "Metrics endpoint responding"
        METRIC_COUNT=$(curl -s http://localhost:8080/metrics | grep -c "^[a-zA-Z]" || echo "0")
        echo "  Metrics Count: $METRIC_COUNT"
    else
        print_error "Metrics endpoint not responding"
    fi
    
    echo ""
}

# Check Infisical connection
check_infisical() {
    print_info "Checking Infisical connection..."
    
    if [ -f ".env.local" ]; then
        source .env.local
        
        if [ -n "$INFISICAL_TOKEN" ]; then
            print_status "Infisical token found"
            
            # Test Infisical connection
            if infisical secrets --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENVIRONMENT" --token="$INFISICAL_TOKEN" > /dev/null 2>&1; then
                print_status "Infisical connection working"
                SECRET_COUNT=$(infisical secrets --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENVIRONMENT" --token="$INFISICAL_TOKEN" --plain | wc -l)
                echo "  Secrets Available: $SECRET_COUNT"
            else
                print_error "Infisical connection failed"
            fi
        else
            print_error "Infisical token not found"
        fi
    else
        print_warning "Infisical configuration file not found"
    fi
    
    echo ""
}

# Check system performance
check_performance() {
    print_info "Checking system performance..."
    
    # CPU usage
    CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    echo "  CPU Usage: ${CPU_USAGE}%"
    
    # Memory usage
    MEM_USAGE=$(free | grep Mem | awk '{printf "%.1f", $3/$2 * 100.0}')
    echo "  Memory Usage: ${MEM_USAGE}%"
    
    # Disk usage
    DISK_USAGE=$(df / | tail -1 | awk '{print $5}' | cut -d'%' -f1)
    echo "  Disk Usage: ${DISK_USAGE}%"
    
    # Load average
    LOAD_AVG=$(uptime | awk -F'load average:' '{print $2}')
    echo "  Load Average:$LOAD_AVG"
    
    # Network latency to Solana
    SOLANA_LATENCY=$(ping -c 1 api.mainnet-beta.solana.com 2>/dev/null | grep 'time=' | awk -F'time=' '{print $2}' | awk '{print $1}' || echo "N/A")
    echo "  Solana RPC Latency: ${SOLANA_LATENCY}"
    
    echo ""
}

# Check logs for errors
check_logs() {
    print_info "Checking recent logs for errors..."
    
    # Check systemd logs
    if systemctl is-active --quiet solana-hft-ninja 2>/dev/null; then
        ERROR_COUNT=$(journalctl -u solana-hft-ninja --since "1 hour ago" | grep -i error | wc -l)
        WARNING_COUNT=$(journalctl -u solana-hft-ninja --since "1 hour ago" | grep -i warning | wc -l)
        
        echo "  Errors (last hour): $ERROR_COUNT"
        echo "  Warnings (last hour): $WARNING_COUNT"
        
        if [ "$ERROR_COUNT" -gt 0 ]; then
            print_warning "Recent errors found in logs"
            echo "  Run: journalctl -u solana-hft-ninja --since '1 hour ago' | grep -i error"
        else
            print_status "No recent errors in logs"
        fi
    else
        print_warning "Cannot check systemd logs - service not active"
    fi
    
    echo ""
}

# Generate summary report
generate_summary() {
    echo -e "${BLUE}📊 Summary Report${NC}"
    echo "=================="
    
    # Overall status
    if pgrep -f hft_main > /dev/null && curl -s http://localhost:8080/health > /dev/null 2>&1; then
        print_status "System Status: OPERATIONAL"
    else
        print_error "System Status: DEGRADED"
    fi
    
    echo ""
    echo "🔗 Quick Access Links:"
    echo "  • Health: http://10.0.0.59:8080/health"
    echo "  • Metrics: http://10.0.0.59:8080/metrics"
    echo "  • DNS: http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080"
    echo ""
    echo "🛠️  Management Commands:"
    echo "  • Status: sudo systemctl status solana-hft-ninja"
    echo "  • Logs: sudo journalctl -u solana-hft-ninja -f"
    echo "  • Restart: sudo systemctl restart solana-hft-ninja"
    echo ""
}

# Main function
main() {
    print_header
    check_oracle_metadata
    check_network
    check_hft_application
    check_endpoints
    check_infisical
    check_performance
    check_logs
    generate_summary
}

# Run main function
main "$@"
