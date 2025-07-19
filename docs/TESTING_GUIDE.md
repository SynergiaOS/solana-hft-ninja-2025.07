# 🧪 Testing Guide - Solana HFT Ninja 2025.07

## 🎯 Quick Start

### **Run All Tests**
```bash
cargo test
```

### **Run Specific Test Suite**
```bash
cargo test --test ai_brain_tests
cargo test --test integration_jito_test
cargo test --lib
```

### **Debug Mode**
```bash
RUST_LOG=debug cargo test -- --nocapture
```

## 📚 Test Architecture

### **Test Organization**
```
tests/
├── ai_brain_tests.rs          # AI engine integration
├── integration_bridge_test.rs # Bridge communication
├── integration_jito_test.rs   # Jito MEV operations
└── mempool_integration.rs     # Transaction processing

src/
└── lib.rs                     # Core unit tests
```

### **Test Categories**

#### **🔧 Unit Tests** (`src/lib.rs`)
- Test individual components in isolation
- Fast execution (<1ms per test)
- Mock external dependencies
- Focus on business logic

#### **🔗 Integration Tests** (`tests/`)
- Test component interactions
- Real-world scenarios
- End-to-end workflows
- Performance validation

## 🛠️ Writing Tests

### **Test Naming Convention**
```rust
#[tokio::test]
async fn test_component_functionality_scenario() {
    // Test implementation
}
```

**Examples:**
- `test_jito_tip_calculation` - Tests tip calculation logic
- `test_bundle_timeout_handling` - Tests timeout scenarios
- `test_ai_coordinator_multi_agent` - Tests AI coordination

### **Test Structure**
```rust
#[tokio::test]
async fn test_example() {
    // 1. Setup
    let config = create_test_config();
    let engine = TestEngine::new(config).await.unwrap();
    
    // 2. Execute
    let result = engine.process_data(test_data).await;
    
    // 3. Verify
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, ExpectedStatus::Success);
    
    // 4. Cleanup (automatic with Drop trait)
}
```

### **Test Data Management**
```rust
// Use unique file names to avoid conflicts
fn create_test_wallet(test_name: &str) -> String {
    let filename = format!("test_wallet_{}.json", test_name);
    // Create wallet file
    filename
}

// Cleanup in test
#[tokio::test]
async fn test_wallet_operations() {
    let wallet_file = create_test_wallet("operations");
    
    // Test logic here
    
    // Cleanup
    let _ = std::fs::remove_file(&wallet_file);
}
```

## 🎯 Test Patterns

### **1. Mock Services**
```rust
struct MockJitoClient {
    should_fail: bool,
    delay_ms: u64,
}

impl MockJitoClient {
    async fn submit_bundle(&self, bundle: Bundle) -> Result<String> {
        if self.should_fail {
            return Err(anyhow!("Network error"));
        }
        
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        Ok("bundle_id_123".to_string())
    }
}
```

### **2. Performance Testing**
```rust
#[tokio::test]
async fn test_performance_under_load() {
    let start = Instant::now();
    
    // Execute operation
    let result = heavy_operation().await;
    
    let duration = start.elapsed();
    
    // Verify performance
    assert!(duration < Duration::from_millis(100));
    assert!(result.is_ok());
}
```

### **3. Error Scenarios**
```rust
#[tokio::test]
async fn test_network_failure_recovery() {
    let mut client = MockClient::new();
    client.set_failure_mode(true);
    
    // Should handle failure gracefully
    let result = client.submit_with_retry().await;
    
    assert!(result.is_err());
    assert_eq!(client.retry_count(), 3); // Verify retry logic
}
```

### **4. Concurrent Testing**
```rust
#[tokio::test]
async fn test_concurrent_operations() {
    let engine = Arc::new(TestEngine::new().await.unwrap());
    
    let tasks: Vec<_> = (0..10).map(|i| {
        let engine = Arc::clone(&engine);
        tokio::spawn(async move {
            engine.process_transaction(i).await
        })
    }).collect();
    
    let results = futures::future::join_all(tasks).await;
    
    // Verify all succeeded
    for result in results {
        assert!(result.unwrap().is_ok());
    }
}
```

## 🔍 Debugging Tests

### **Common Issues**

#### **1. File Conflicts**
```bash
# Error: File already exists
# Solution: Use unique file names
let filename = format!("test_{}_{}.json", test_name, thread_id);
```

#### **2. Timeout Issues**
```bash
# Error: Test timeout
# Solution: Increase timeout or optimize test
tokio::time::timeout(Duration::from_secs(30), test_operation()).await
```

#### **3. Memory Limits**
```bash
# Error: Memory limit exceeded
# Solution: Check memory usage in test
assert!(memory_usage < 16 * 1024 * 1024); // 16MB limit
```

### **Debug Commands**
```bash
# Single test with full logging
RUST_LOG=trace cargo test test_name -- --nocapture --test-threads=1

# Test with backtrace
RUST_BACKTRACE=full cargo test test_name

# Test timing information
cargo test -- --report-time

# Run specific test pattern
cargo test jito -- --nocapture
```

## 📊 Performance Benchmarks

### **Latency Targets**
```rust
// Bundle operations
assert!(bundle_time < Duration::from_millis(100));

// Transaction parsing  
assert!(parse_time < Duration::from_micros(1000));

// AI inference
assert!(ai_time < Duration::from_millis(200));
```

### **Throughput Targets**
```rust
// Mempool processing
assert!(transactions_per_second > 1000);

// Bundle creation
assert!(bundles_per_second > 100);

// Event processing
assert!(events_per_second > 5000);
```

## 🛡️ Test Quality Guidelines

### **Test Independence**
- Each test should run independently
- No shared state between tests
- Clean up resources after each test
- Use unique identifiers for test data

### **Test Reliability**
- Tests should be deterministic
- No flaky tests allowed
- Handle timing issues properly
- Mock external dependencies

### **Test Maintainability**
- Clear test names and documentation
- Reusable test utilities
- Consistent test patterns
- Regular test review and cleanup

## 🚀 Continuous Integration

### **Pre-commit Checks**
```bash
# Run before committing
cargo test
cargo clippy
cargo fmt --check
```

### **CI Pipeline**
```yaml
# .github/workflows/test.yml
- name: Run tests
  run: cargo test --all-features
  
- name: Check performance
  run: cargo test --release -- --ignored
```

## 📋 Test Maintenance

### **Adding New Tests**
1. Identify test category (unit vs integration)
2. Follow naming conventions
3. Use appropriate test patterns
4. Add performance assertions
5. Include error scenarios
6. Update documentation

### **Updating Existing Tests**
1. Maintain backward compatibility
2. Update performance expectations
3. Add new error scenarios
4. Refactor common patterns
5. Update documentation

### **Test Review Checklist**
- [ ] Test names are descriptive
- [ ] Tests are independent
- [ ] Performance assertions included
- [ ] Error scenarios covered
- [ ] Resources cleaned up
- [ ] Documentation updated

---

**Guide Version:** 2025.07  
**Last Updated:** 2025-07-19  
**Status:** Production Ready ✅
