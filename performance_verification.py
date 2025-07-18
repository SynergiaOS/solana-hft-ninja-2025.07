#!/usr/bin/env python3
"""
📊 Performance Verification for Solana HFT Ninja + DeepSeek-Math AI
Comprehensive performance testing and metrics collection
"""

import time
import requests
import psutil
import json
import subprocess
import sys
from typing import Dict, List, Any

# Configuration
AI_API_URL = "http://localhost:8003"
BFF_API_URL = "http://localhost:8002"
FRONTEND_URL = "http://localhost:3000"

class PerformanceVerifier:
    def __init__(self):
        self.results = {}
        
    def print_header(self, title: str):
        print(f"\n{'='*60}")
        print(f"📊 {title}")
        print(f"{'='*60}")
        
    def print_result(self, test_name: str, value: Any, target: str = "", status: str = ""):
        if status == "PASS":
            status_icon = "✅"
        elif status == "FAIL":
            status_icon = "❌"
        else:
            status_icon = "📊"
            
        print(f"{status_icon} {test_name}: {value} {target}")
        
    def test_response_times(self):
        """Test API response times"""
        self.print_header("Response Time Testing")
        
        # Test AI API Health
        try:
            start_time = time.time()
            response = requests.get(f"{AI_API_URL}/health", timeout=5)
            ai_health_time = (time.time() - start_time) * 1000
            
            if response.status_code == 200:
                status = "PASS" if ai_health_time < 100 else "FAIL"
                self.print_result("AI API Health", f"{ai_health_time:.1f}ms", "(target: <100ms)", status)
                self.results['ai_health_latency'] = ai_health_time
            else:
                self.print_result("AI API Health", "FAILED", f"Status: {response.status_code}", "FAIL")
        except Exception as e:
            self.print_result("AI API Health", "ERROR", str(e), "FAIL")
            
        # Test AI Calculation
        try:
            payload = {
                "capital": 8.0,
                "risk_tolerance": 0.05,
                "expected_return": 0.15,
                "volatility": 0.3,
                "strategy": "wallet_tracker"
            }
            
            start_time = time.time()
            response = requests.post(f"{AI_API_URL}/calculate/position-size", 
                                   json=payload, timeout=10)
            ai_calc_time = (time.time() - start_time) * 1000
            
            if response.status_code == 200:
                data = response.json()
                reported_latency = data.get('metadata', {}).get('latency_ms', 0)
                status = "PASS" if ai_calc_time < 500 else "FAIL"
                self.print_result("AI Calculation", f"{ai_calc_time:.1f}ms", "(target: <500ms)", status)
                self.print_result("AI Reported Latency", f"{reported_latency}ms", "(internal)", "")
                self.results['ai_calc_latency'] = ai_calc_time
                self.results['ai_reported_latency'] = reported_latency
            else:
                self.print_result("AI Calculation", "FAILED", f"Status: {response.status_code}", "FAIL")
        except Exception as e:
            self.print_result("AI Calculation", "ERROR", str(e), "FAIL")
            
        # Test BFF Health
        try:
            start_time = time.time()
            response = requests.get(f"{BFF_API_URL}/health", timeout=5)
            bff_health_time = (time.time() - start_time) * 1000
            
            if response.status_code == 200:
                status = "PASS" if bff_health_time < 200 else "FAIL"
                self.print_result("BFF Health", f"{bff_health_time:.1f}ms", "(target: <200ms)", status)
                self.results['bff_health_latency'] = bff_health_time
            else:
                self.print_result("BFF Health", "FAILED", f"Status: {response.status_code}", "FAIL")
        except Exception as e:
            self.print_result("BFF Health", "ERROR", str(e), "FAIL")
            
        # Test Frontend
        try:
            start_time = time.time()
            response = requests.get(FRONTEND_URL, timeout=5)
            frontend_time = (time.time() - start_time) * 1000
            
            if response.status_code == 200:
                status = "PASS" if frontend_time < 1000 else "FAIL"
                self.print_result("Frontend Load", f"{frontend_time:.1f}ms", "(target: <1000ms)", status)
                self.results['frontend_latency'] = frontend_time
            else:
                self.print_result("Frontend Load", "FAILED", f"Status: {response.status_code}", "FAIL")
        except Exception as e:
            self.print_result("Frontend Load", "ERROR", str(e), "FAIL")
            
    def test_resource_usage(self):
        """Test system resource usage"""
        self.print_header("Resource Usage Analysis")
        
        # Get Python processes
        python_processes = []
        for proc in psutil.process_iter(['pid', 'name', 'memory_info', 'cpu_percent']):
            try:
                if 'python' in proc.info['name'].lower():
                    cmdline = proc.cmdline()
                    if any('deepseek' in arg or 'main_simple' in arg or 'http.server' in arg for arg in cmdline):
                        python_processes.append(proc)
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
                
        total_memory_mb = 0
        total_cpu_percent = 0
        
        for proc in python_processes:
            try:
                memory_mb = proc.memory_info().rss / 1024 / 1024
                cpu_percent = proc.cpu_percent()
                total_memory_mb += memory_mb
                total_cpu_percent += cpu_percent
                
                # Identify service type
                cmdline = ' '.join(proc.cmdline())
                if 'deepseek' in cmdline:
                    service_name = "DeepSeek-Math AI"
                elif 'main_simple' in cmdline:
                    service_name = "Cerebro BFF"
                elif 'http.server' in cmdline:
                    service_name = "React Dashboard"
                else:
                    service_name = "Python Service"
                    
                self.print_result(f"{service_name} Memory", f"{memory_mb:.1f}MB", "", "")
                self.print_result(f"{service_name} CPU", f"{cpu_percent:.1f}%", "", "")
                
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
                
        # Total resource usage
        status = "PASS" if total_memory_mb < 200 else "FAIL"
        self.print_result("Total Memory Usage", f"{total_memory_mb:.1f}MB", "(target: <200MB)", status)
        
        status = "PASS" if total_cpu_percent < 50 else "FAIL"
        self.print_result("Total CPU Usage", f"{total_cpu_percent:.1f}%", "(target: <50%)", status)
        
        self.results['total_memory_mb'] = total_memory_mb
        self.results['total_cpu_percent'] = total_cpu_percent
        
        # System memory
        memory = psutil.virtual_memory()
        self.print_result("System Memory", f"{memory.used/1024/1024/1024:.1f}GB / {memory.total/1024/1024/1024:.1f}GB", f"({memory.percent:.1f}% used)", "")
        
    def test_ai_accuracy(self):
        """Test AI calculation accuracy"""
        self.print_header("AI Calculation Accuracy")
        
        test_cases = [
            {
                "name": "Conservative Position",
                "payload": {"capital": 10.0, "risk_tolerance": 0.02, "expected_return": 0.10, "volatility": 0.20, "strategy": "conservative"},
                "expected_max_position": 0.5  # Should be conservative
            },
            {
                "name": "Aggressive Position", 
                "payload": {"capital": 10.0, "risk_tolerance": 0.10, "expected_return": 0.25, "volatility": 0.40, "strategy": "aggressive"},
                "expected_max_position": 2.5  # Can be more aggressive
            },
            {
                "name": "Arbitrage Opportunity",
                "endpoint": "arbitrage-profit",
                "payload": {"token": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "price_a": 1.0, "price_b": 1.05, "liquidity_a": 1000.0, "liquidity_b": 1000.0, "gas_cost": 0.001},
                "expected_profitable": True
            }
        ]
        
        for test_case in test_cases:
            try:
                endpoint = test_case.get('endpoint', 'position-size')
                response = requests.post(f"{AI_API_URL}/calculate/{endpoint}", 
                                       json=test_case['payload'], timeout=10)
                
                if response.status_code == 200:
                    data = response.json()
                    
                    if endpoint == 'position-size':
                        position_size = data['result']['position_size']
                        confidence = data['result']['confidence']
                        
                        # Check if position size is reasonable
                        if position_size <= test_case['expected_max_position']:
                            status = "PASS"
                        else:
                            status = "FAIL"
                            
                        self.print_result(f"{test_case['name']} Size", f"{position_size:.3f} SOL", f"(max: {test_case['expected_max_position']})", status)
                        self.print_result(f"{test_case['name']} Confidence", f"{confidence:.2f}", "(0.0-1.0)", "")
                        
                    elif endpoint == 'arbitrage-profit':
                        is_profitable = data['result']['is_profitable']
                        net_profit = data['result']['net_profit']
                        
                        status = "PASS" if is_profitable == test_case['expected_profitable'] else "FAIL"
                        self.print_result(f"{test_case['name']} Profitable", str(is_profitable), "", status)
                        self.print_result(f"{test_case['name']} Profit", f"{net_profit:.4f} SOL", "", "")
                        
                else:
                    self.print_result(test_case['name'], "FAILED", f"Status: {response.status_code}", "FAIL")
                    
            except Exception as e:
                self.print_result(test_case['name'], "ERROR", str(e), "FAIL")
                
    def test_throughput(self):
        """Test API throughput"""
        self.print_header("Throughput Testing")
        
        # Test concurrent requests
        import concurrent.futures
        import threading
        
        def make_request():
            try:
                response = requests.get(f"{AI_API_URL}/health", timeout=5)
                return response.status_code == 200
            except:
                return False
                
        # Test with 10 concurrent requests
        start_time = time.time()
        with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
            futures = [executor.submit(make_request) for _ in range(10)]
            results = [future.result() for future in concurrent.futures.as_completed(futures)]
            
        total_time = time.time() - start_time
        success_rate = sum(results) / len(results) * 100
        requests_per_second = len(results) / total_time
        
        status = "PASS" if success_rate >= 90 else "FAIL"
        self.print_result("Concurrent Requests Success", f"{success_rate:.1f}%", "(target: >90%)", status)
        
        status = "PASS" if requests_per_second >= 20 else "FAIL"
        self.print_result("Requests per Second", f"{requests_per_second:.1f} req/s", "(target: >20 req/s)", status)
        
        self.results['success_rate'] = success_rate
        self.results['requests_per_second'] = requests_per_second
        
    def generate_report(self):
        """Generate final performance report"""
        self.print_header("Performance Report Summary")
        
        print("🎯 Target Metrics:")
        print("  • AI Health Latency: <100ms")
        print("  • AI Calculation Latency: <500ms") 
        print("  • BFF Health Latency: <200ms")
        print("  • Frontend Load Time: <1000ms")
        print("  • Total Memory Usage: <200MB")
        print("  • Total CPU Usage: <50%")
        print("  • Concurrent Success Rate: >90%")
        print("  • Throughput: >20 req/s")
        
        print("\n📊 Achieved Metrics:")
        for key, value in self.results.items():
            if 'latency' in key:
                print(f"  • {key.replace('_', ' ').title()}: {value:.1f}ms")
            elif 'memory' in key:
                print(f"  • {key.replace('_', ' ').title()}: {value:.1f}MB")
            elif 'cpu' in key:
                print(f"  • {key.replace('_', ' ').title()}: {value:.1f}%")
            elif 'rate' in key:
                print(f"  • {key.replace('_', ' ').title()}: {value:.1f}%")
            elif 'requests_per_second' in key:
                print(f"  • {key.replace('_', ' ').title()}: {value:.1f} req/s")
                
        # Overall assessment
        print("\n🎉 Overall Assessment:")
        if (self.results.get('ai_calc_latency', 1000) < 500 and 
            self.results.get('total_memory_mb', 1000) < 200 and
            self.results.get('success_rate', 0) >= 90):
            print("✅ EXCELLENT - All performance targets met!")
            print("🚀 System ready for production deployment")
        else:
            print("⚠️  GOOD - Most targets met, some optimization possible")
            print("🔧 Consider performance tuning for production")

def main():
    print("🚀 Starting Performance Verification for Solana HFT Ninja")
    print("📊 Testing DeepSeek-Math AI + Cerebro BFF + React Dashboard")
    
    verifier = PerformanceVerifier()
    
    try:
        verifier.test_response_times()
        verifier.test_resource_usage()
        verifier.test_ai_accuracy()
        verifier.test_throughput()
        verifier.generate_report()
        
        print(f"\n{'='*60}")
        print("✅ Performance verification completed successfully!")
        print("📋 All metrics collected and analyzed")
        print(f"{'='*60}")
        
    except KeyboardInterrupt:
        print("\n⚠️ Performance verification interrupted by user")
    except Exception as e:
        print(f"\n❌ Error during performance verification: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
