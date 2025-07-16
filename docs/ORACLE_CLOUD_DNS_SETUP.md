# 🌐 Oracle Cloud DNS Setup for HFT Ninja

## 🎯 Quick DNS Configuration

### Current Setup
- **Private IP**: `10.0.0.59`
- **Internal DNS**: `subnet07161247.vcn07161247.oraclevcn.com`
- **Target URL**: `http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080`

## 🔧 VCN Security List Configuration

### Required Inbound Rules

```bash
# Add these rules to your VCN Security List:

Rule 1: HFT Dashboard Access
- Source Type: CIDR
- Source CIDR: 0.0.0.0/0
- IP Protocol: TCP
- Destination Port Range: 8080
- Description: "HFT Ninja Dashboard Access"

Rule 2: SSH Access (if needed)
- Source Type: CIDR
- Source CIDR: YOUR_IP/32  # Replace with your IP
- IP Protocol: TCP
- Destination Port Range: 22
- Description: "SSH Access"

Rule 3: Prometheus Metrics (internal)
- Source Type: CIDR
- Source CIDR: 10.0.0.0/16
- IP Protocol: TCP
- Destination Port Range: 9100
- Description: "Prometheus Metrics"
```

### Oracle Cloud Console Steps

1. **Navigate to VCN**:
   ```
   Oracle Cloud Console → Networking → Virtual Cloud Networks
   → Select your VCN → Security Lists → Default Security List
   ```

2. **Add Inbound Rules**:
   ```
   Click "Add Ingress Rules"
   → Fill in the rules above
   → Click "Add Ingress Rules"
   ```

3. **Verify Configuration**:
   ```bash
   # Test from external network
   curl -I http://10.0.0.59:8080/health
   
   # Should return HTTP 200 OK
   ```

## 🚀 DNS Subdomain Setup (Optional)

### Option 1: Oracle Cloud DNS Zone

```bash
# Create DNS zone for custom subdomain
oci dns zone create \
  --name "hft-ninja.oraclevcn.com" \
  --zone-type PRIMARY \
  --compartment-id <your-compartment-id>

# Add A record
oci dns record rrset update \
  --zone-name-or-id "hft-ninja.oraclevcn.com" \
  --domain "ninja.hft-ninja.oraclevcn.com" \
  --rtype "A" \
  --items '[{"domain": "ninja.hft-ninja.oraclevcn.com", "rdata": "10.0.0.59", "rtype": "A", "ttl": 300}]'
```

### Option 2: Use Existing VCN DNS

The existing DNS `subnet07161247.vcn07161247.oraclevcn.com` should work automatically for:
- `http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080`

## 🔍 Testing DNS Configuration

### Basic Connectivity Test

```bash
# Test from within Oracle Cloud
curl -v http://10.0.0.59:8080/health

# Test DNS resolution
nslookup subnet07161247.vcn07161247.oraclevcn.com

# Test full URL
curl -v http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080/health
```

### Advanced Network Testing

```bash
# Test from external network (if rules allow)
telnet 10.0.0.59 8080

# Check port accessibility
nmap -p 8080 10.0.0.59

# Test WebSocket connection (if using)
wscat -c ws://10.0.0.59:8080/ws
```

## 🛡️ Security Considerations

### Recommended Security Rules

```bash
# Restrict dashboard access to specific IPs
Source CIDR: YOUR_OFFICE_IP/32  # Instead of 0.0.0.0/0

# Use VPN or bastion host for production
Source CIDR: VPN_SUBNET/24

# Internal monitoring only
Source CIDR: 10.0.0.0/16  # VCN internal only
```

### Production Security Setup

```bash
# 1. Create bastion host
oci compute instance launch \
  --availability-domain <AD> \
  --compartment-id <compartment-id> \
  --shape VM.Standard2.1 \
  --subnet-id <public-subnet-id> \
  --display-name "hft-bastion"

# 2. Configure SSH tunneling
ssh -L 8080:10.0.0.59:8080 opc@<bastion-public-ip>

# 3. Access via tunnel
curl http://localhost:8080/health
```

## 📊 Monitoring URLs

### Direct Access (if security rules allow)
```
Health Check: http://10.0.0.59:8080/health
Metrics: http://10.0.0.59:8080/metrics
Dashboard: http://10.0.0.59:8080/
```

### DNS Access (if configured)
```
Health Check: http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080/health
Metrics: http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080/metrics
Dashboard: http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080/
```

## 🔧 Troubleshooting

### Common Issues

1. **Connection Refused**
   ```bash
   # Check if service is running
   sudo systemctl status solana-hft-ninja
   
   # Check if port is listening
   netstat -tulpn | grep :8080
   ```

2. **DNS Resolution Failed**
   ```bash
   # Check DNS configuration
   cat /etc/resolv.conf
   
   # Test DNS resolution
   dig subnet07161247.vcn07161247.oraclevcn.com
   ```

3. **Security List Issues**
   ```bash
   # Check current rules
   oci network security-list list --compartment-id <compartment-id>
   
   # Verify rule configuration
   oci network security-list get --security-list-id <security-list-id>
   ```

### Debug Commands

```bash
# Check Oracle Cloud metadata
curl http://169.254.169.254/opc/v2/instance/

# Test internal connectivity
ping 10.0.0.1  # VCN gateway

# Check firewall rules
sudo iptables -L -n

# Monitor network traffic
sudo tcpdump -i any port 8080
```

## 🎯 Quick Setup Commands

### For Oracle Cloud Console
1. Go to: **Networking → Virtual Cloud Networks**
2. Select your VCN
3. Click **Security Lists → Default Security List**
4. Click **Add Ingress Rules**
5. Add rule: `Source: 0.0.0.0/0, Protocol: TCP, Port: 8080`
6. Click **Add Ingress Rules**

### For Command Line
```bash
# Get security list ID
SECURITY_LIST_ID=$(oci network security-list list \
  --compartment-id <compartment-id> \
  --query 'data[0].id' --raw-output)

# Add ingress rule
oci network security-list update \
  --security-list-id $SECURITY_LIST_ID \
  --ingress-security-rules '[{
    "source": "0.0.0.0/0",
    "protocol": "6",
    "tcpOptions": {
      "destinationPortRange": {
        "min": 8080,
        "max": 8080
      }
    }
  }]'
```

## ✅ Verification Checklist

- [ ] Security List rules added
- [ ] HFT Ninja service running
- [ ] Port 8080 accessible
- [ ] Health endpoint responding
- [ ] Metrics endpoint working
- [ ] DNS resolution functional
- [ ] External access tested

---

**🥷 Your HFT Ninja is now accessible via Oracle Cloud DNS!**
