#!/bin/bash

# 🛡️ SECURITY AUDIT SCRIPT
# Comprehensive security check before mainnet deployment

set -euo pipefail

echo "🛡️  SECURITY AUDIT - MAINNET DEPLOYMENT"
echo "======================================="

AUDIT_PASSED=true
WARNINGS=0
ERRORS=0

# Function to log results
log_check() {
    local status=$1
    local message=$2
    local severity=${3:-"INFO"}
    
    case $status in
        "PASS")
            echo "✅ $message"
            ;;
        "WARN")
            echo "⚠️  $message"
            ((WARNINGS++))
            ;;
        "FAIL")
            echo "❌ $message"
            ((ERRORS++))
            AUDIT_PASSED=false
            ;;
    esac
}

echo "🔍 Starting security audit..."
echo ""

# 1. File Permissions Check
echo "📁 CHECKING FILE PERMISSIONS"
echo "----------------------------"

if [ -f ".env.mainnet" ]; then
    PERM=$(stat -c "%a" .env.mainnet)
    if [ "$PERM" = "600" ]; then
        log_check "PASS" ".env.mainnet has secure permissions (600)"
    else
        log_check "FAIL" ".env.mainnet has insecure permissions ($PERM), should be 600"
    fi
else
    log_check "FAIL" ".env.mainnet not found"
fi

# Check wallet files
WALLET_DIR="$HOME/.solana-mainnet"
if [ -d "$WALLET_DIR" ]; then
    WALLET_PERM=$(stat -c "%a" "$WALLET_DIR")
    if [ "$WALLET_PERM" = "700" ]; then
        log_check "PASS" "Wallet directory has secure permissions (700)"
    else
        log_check "FAIL" "Wallet directory has insecure permissions ($WALLET_PERM)"
    fi
    
    if [ -f "$WALLET_DIR/trading-wallet.json" ]; then
        WALLET_FILE_PERM=$(stat -c "%a" "$WALLET_DIR/trading-wallet.json")
        if [ "$WALLET_FILE_PERM" = "600" ]; then
            log_check "PASS" "Wallet file has secure permissions (600)"
        else
            log_check "FAIL" "Wallet file has insecure permissions ($WALLET_FILE_PERM)"
        fi
    else
        log_check "FAIL" "Wallet file not found"
    fi
else
    log_check "FAIL" "Wallet directory not found"
fi

echo ""

# 2. Environment Configuration Check
echo "⚙️  CHECKING ENVIRONMENT CONFIGURATION"
echo "-------------------------------------"

if [ -f ".env.mainnet" ]; then
    source .env.mainnet
    
    # Check critical variables
    if [ -n "${HELIUS_API_KEY:-}" ] && [ "$HELIUS_API_KEY" != "your_helius_key_here" ]; then
        log_check "PASS" "Helius API key configured"
    else
        log_check "FAIL" "Helius API key not configured"
    fi
    
    if [ -n "${QUICKNODE_ENDPOINT:-}" ] && [ "$QUICKNODE_ENDPOINT" != "your_quicknode_endpoint_here" ]; then
        log_check "PASS" "QuickNode endpoint configured"
    else
        log_check "WARN" "QuickNode fallback endpoint not configured"
    fi
    
    if [ "${TRADING_ENABLED:-}" = "false" ]; then
        log_check "PASS" "Trading disabled by default (safe)"
    else
        log_check "WARN" "Trading enabled - ensure this is intentional"
    fi
    
    if [ -n "${MAX_POSITION_SIZE_SOL:-}" ]; then
        if (( $(echo "$MAX_POSITION_SIZE_SOL <= 2.0" | bc -l) )); then
            log_check "PASS" "Position size limit reasonable ($MAX_POSITION_SIZE_SOL SOL)"
        else
            log_check "WARN" "Position size limit high ($MAX_POSITION_SIZE_SOL SOL)"
        fi
    else
        log_check "FAIL" "Position size limit not set"
    fi
    
    if [ "${CIRCUIT_BREAKER_ENABLED:-}" = "true" ]; then
        log_check "PASS" "Circuit breaker enabled"
    else
        log_check "FAIL" "Circuit breaker disabled - high risk"
    fi
fi

echo ""

# 3. Git Security Check
echo "📝 CHECKING GIT SECURITY"
echo "------------------------"

if [ -f ".gitignore" ]; then
    if grep -q ".env.mainnet" .gitignore; then
        log_check "PASS" ".env.mainnet in .gitignore"
    else
        log_check "FAIL" ".env.mainnet not in .gitignore"
    fi
    
    if grep -q "*.json" .gitignore; then
        log_check "PASS" "JSON files in .gitignore"
    else
        log_check "WARN" "JSON files not in .gitignore"
    fi
else
    log_check "WARN" ".gitignore not found"
fi

# Check for committed secrets
if git rev-parse --git-dir > /dev/null 2>&1; then
    if git log --all --full-history -- "*.json" | grep -q "commit"; then
        log_check "WARN" "JSON files found in git history - check for leaked keys"
    else
        log_check "PASS" "No JSON files in git history"
    fi
fi

echo ""

# 4. Network Security Check
echo "🌐 CHECKING NETWORK SECURITY"
echo "----------------------------"

# Check RPC endpoints
if [ -n "${SOLANA_RPC_URL:-}" ]; then
    if [[ "$SOLANA_RPC_URL" == *"mainnet"* ]]; then
        log_check "PASS" "RPC URL points to mainnet"
    else
        log_check "FAIL" "RPC URL does not point to mainnet"
    fi
    
    if [[ "$SOLANA_RPC_URL" == *"https"* ]]; then
        log_check "PASS" "RPC URL uses HTTPS"
    else
        log_check "FAIL" "RPC URL does not use HTTPS"
    fi
else
    log_check "FAIL" "RPC URL not configured"
fi

echo ""

# 5. Code Security Check
echo "💻 CHECKING CODE SECURITY"
echo "-------------------------"

# Check for hardcoded secrets
if grep -r "private_key\|secret\|password" src/ --include="*.rs" --include="*.ts" --include="*.js" 2>/dev/null | grep -v "// TODO\|// FIXME"; then
    log_check "WARN" "Potential hardcoded secrets found in code"
else
    log_check "PASS" "No hardcoded secrets found"
fi

# Check for debug prints
if grep -r "println!\|console.log\|dbg!" src/ --include="*.rs" --include="*.ts" --include="*.js" 2>/dev/null; then
    log_check "WARN" "Debug prints found - may leak sensitive data"
else
    log_check "PASS" "No debug prints found"
fi

echo ""

# 6. Dependencies Check
echo "📦 CHECKING DEPENDENCIES"
echo "------------------------"

# Check for known vulnerable packages
if [ -f "Cargo.toml" ]; then
    if command -v cargo-audit &> /dev/null; then
        if cargo audit --quiet; then
            log_check "PASS" "No known vulnerabilities in Rust dependencies"
        else
            log_check "WARN" "Vulnerabilities found in Rust dependencies"
        fi
    else
        log_check "WARN" "cargo-audit not installed - cannot check Rust dependencies"
    fi
fi

if [ -f "package.json" ]; then
    if command -v npm &> /dev/null; then
        if npm audit --audit-level=high --silent; then
            log_check "PASS" "No high-severity vulnerabilities in npm dependencies"
        else
            log_check "WARN" "High-severity vulnerabilities found in npm dependencies"
        fi
    fi
fi

echo ""

# 7. Backup Check
echo "💾 CHECKING BACKUPS"
echo "------------------"

BACKUP_DIR="$HOME/.solana-backups"
if [ -d "$BACKUP_DIR" ]; then
    BACKUP_COUNT=$(find "$BACKUP_DIR" -name "trading-wallet-backup-*.json" | wc -l)
    if [ "$BACKUP_COUNT" -gt 0 ]; then
        log_check "PASS" "$BACKUP_COUNT wallet backup(s) found"
    else
        log_check "WARN" "No wallet backups found"
    fi
else
    log_check "WARN" "Backup directory not found"
fi

echo ""

# Final Report
echo "📊 AUDIT SUMMARY"
echo "================"
echo "Errors: $ERRORS"
echo "Warnings: $WARNINGS"

if [ "$AUDIT_PASSED" = true ]; then
    if [ "$WARNINGS" -eq 0 ]; then
        echo "🎉 AUDIT PASSED - System ready for mainnet deployment!"
        exit 0
    else
        echo "⚠️  AUDIT PASSED WITH WARNINGS - Review warnings before deployment"
        exit 1
    fi
else
    echo "❌ AUDIT FAILED - Fix errors before mainnet deployment!"
    exit 2
fi
