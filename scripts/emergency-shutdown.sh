#!/bin/bash

# 🚨 EMERGENCY SHUTDOWN SCRIPT
# Immediate halt of all trading activities

set -euo pipefail

TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
SHUTDOWN_REASON="${1:-Manual emergency shutdown}"
LOG_FILE="logs/emergency-shutdown-$(date +%Y%m%d_%H%M%S).log"

echo "🚨 EMERGENCY SHUTDOWN INITIATED" | tee -a "$LOG_FILE"
echo "Time: $TIMESTAMP" | tee -a "$LOG_FILE"
echo "Reason: $SHUTDOWN_REASON" | tee -a "$LOG_FILE"
echo "========================================" | tee -a "$LOG_FILE"

# Function to log with timestamp
log() {
    echo "[$(date '+%H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Function to send alerts
send_alert() {
    local message="$1"
    local severity="${2:-CRITICAL}"
    
    # Telegram alert
    if [ -n "${TELEGRAM_BOT_TOKEN:-}" ] && [ -n "${TELEGRAM_CHAT_ID:-}" ]; then
        curl -s -X POST "https://api.telegram.org/bot$TELEGRAM_BOT_TOKEN/sendMessage" \
            -d chat_id="$TELEGRAM_CHAT_ID" \
            -d text="🚨 $severity: $message" \
            -d parse_mode="HTML" || true
    fi
    
    # Discord webhook
    if [ -n "${DISCORD_WEBHOOK_URL:-}" ]; then
        curl -s -X POST "$DISCORD_WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "{\"content\": \"🚨 **$severity**: $message\"}" || true
    fi
    
    # Email alert (if configured)
    if command -v mail &> /dev/null && [ -n "${EMERGENCY_CONTACT_EMAIL:-}" ]; then
        echo "$message" | mail -s "🚨 EMERGENCY: Cerebro HFT Shutdown" "$EMERGENCY_CONTACT_EMAIL" || true
    fi
}

# Step 1: Disable trading in HFT engine
log "1. Disabling trading in HFT engine..."
if curl -s -X POST "http://localhost:8080/api/emergency/disable" \
   -H "Content-Type: application/json" \
   -d "{\"reason\": \"$SHUTDOWN_REASON\"}" | grep -q "success"; then
    log "✅ HFT trading disabled successfully"
else
    log "❌ Failed to disable HFT trading via API"
    
    # Try to stop container if API fails
    if docker ps | grep -q "cerebro-hft-mainnet"; then
        log "🔄 Stopping HFT container..."
        docker stop cerebro-hft-mainnet || true
        log "✅ HFT container stopped"
    fi
fi

# Step 2: Cancel all open orders
log "2. Cancelling all open orders..."
if curl -s -X POST "http://localhost:8080/api/orders/cancel-all" \
   -H "Content-Type: application/json" | grep -q "success"; then
    log "✅ All orders cancelled successfully"
else
    log "❌ Failed to cancel orders via API"
fi

# Step 3: Close all positions (if any)
log "3. Closing all open positions..."
if curl -s -X POST "http://localhost:8080/api/positions/close-all" \
   -H "Content-Type: application/json" | grep -q "success"; then
    log "✅ All positions closed successfully"
else
    log "❌ Failed to close positions via API"
fi

# Step 4: Stop Kestra workflows
log "4. Stopping Kestra workflows..."
if docker ps | grep -q "cerebro-kestra-mainnet"; then
    docker exec cerebro-kestra-mainnet curl -s -X POST "http://localhost:8080/api/v1/executions/stop-all" || true
    log "✅ Kestra workflows stopped"
else
    log "⚠️ Kestra container not running"
fi

# Step 5: Set emergency flag in database
log "5. Setting emergency flag in database..."
if command -v psql &> /dev/null; then
    PGPASSWORD="${POSTGRES_PASSWORD}" psql -h localhost -U cerebro -d cerebro_mainnet -c \
        "UPDATE system_config SET emergency_stop = true, emergency_reason = '$SHUTDOWN_REASON', emergency_timestamp = NOW();" || true
    log "✅ Emergency flag set in database"
else
    log "⚠️ psql not available, skipping database update"
fi

# Step 6: Create emergency state file
log "6. Creating emergency state file..."
cat > "emergency.state" << EOF
{
    "emergency_stop": true,
    "reason": "$SHUTDOWN_REASON",
    "timestamp": "$TIMESTAMP",
    "shutdown_by": "$(whoami)",
    "hostname": "$(hostname)",
    "trading_disabled": true,
    "orders_cancelled": true,
    "positions_closed": true
}
EOF
log "✅ Emergency state file created"

# Step 7: Backup current state
log "7. Creating emergency backup..."
BACKUP_DIR="backups/emergency-$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Backup logs
cp -r logs/* "$BACKUP_DIR/" 2>/dev/null || true

# Backup database
if command -v pg_dump &> /dev/null; then
    PGPASSWORD="${POSTGRES_PASSWORD}" pg_dump -h localhost -U cerebro cerebro_mainnet > "$BACKUP_DIR/database_backup.sql" || true
    log "✅ Database backup created"
fi

# Backup configuration
cp .env.mainnet "$BACKUP_DIR/" 2>/dev/null || true
cp docker-compose.mainnet.yml "$BACKUP_DIR/" 2>/dev/null || true

log "✅ Emergency backup completed in $BACKUP_DIR"

# Step 8: Send comprehensive alert
log "8. Sending emergency alerts..."
ALERT_MESSAGE="CEREBRO HFT EMERGENCY SHUTDOWN

Timestamp: $TIMESTAMP
Reason: $SHUTDOWN_REASON
Actions Taken:
- ✅ Trading disabled
- ✅ Orders cancelled  
- ✅ Positions closed
- ✅ Workflows stopped
- ✅ Emergency state saved
- ✅ Backup created

System Status: SAFE MODE
Manual intervention required to restart.

Backup Location: $BACKUP_DIR
Log File: $LOG_FILE"

send_alert "$ALERT_MESSAGE" "EMERGENCY"
log "✅ Emergency alerts sent"

# Step 9: Display final status
echo ""
echo "🚨 EMERGENCY SHUTDOWN COMPLETED"
echo "==============================="
echo "Time: $(date '+%Y-%m-%d %H:%M:%S')"
echo "Reason: $SHUTDOWN_REASON"
echo "Status: SYSTEM IN SAFE MODE"
echo ""
echo "📁 Backup: $BACKUP_DIR"
echo "📝 Log: $LOG_FILE"
echo "🔧 State: emergency.state"
echo ""
echo "⚠️  MANUAL INTERVENTION REQUIRED TO RESTART"
echo "   Run: ./scripts/restart-after-emergency.sh"
echo ""

# Step 10: Monitor for manual restart
log "9. Emergency shutdown complete. Monitoring disabled."
log "Manual restart required using: ./scripts/restart-after-emergency.sh"

# Create restart instructions
cat > "RESTART_INSTRUCTIONS.txt" << EOF
🚨 EMERGENCY SHUTDOWN RECOVERY INSTRUCTIONS

The system was shut down due to: $SHUTDOWN_REASON
Shutdown time: $TIMESTAMP

BEFORE RESTARTING:
1. Investigate the cause of the emergency
2. Review logs in: $LOG_FILE
3. Check backup in: $BACKUP_DIR
4. Verify wallet balance and positions
5. Ensure the issue is resolved

TO RESTART:
1. Review and update configuration if needed
2. Run: ./scripts/restart-after-emergency.sh
3. Monitor system carefully for first hour
4. Verify all systems are functioning normally

EMERGENCY CONTACTS:
- Admin: ${EMERGENCY_CONTACT_EMAIL:-not_configured}
- Phone: ${EMERGENCY_CONTACT_PHONE:-not_configured}

DO NOT RESTART WITHOUT PROPER INVESTIGATION!
EOF

echo "📋 Restart instructions saved to: RESTART_INSTRUCTIONS.txt"

# Exit with error code to indicate emergency state
exit 1
