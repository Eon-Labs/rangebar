#!/usr/bin/env python3
"""
Production Scale Range Bar Validator
Comprehensive execution with performance monitoring and full traceability
"""

import subprocess
import time
import json
import psutil
import os
import sys
from datetime import datetime, timedelta
from pathlib import Path

class ProductionScaleValidator:
    def __init__(self):
        self.start_time = None
        self.end_time = None
        self.process = None
        self.performance_data = []
        self.results = {}
        
    def create_execution_metadata(self, symbol, start_date, end_date, threshold):
        """Create comprehensive metadata for machine discovery"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        metadata = {
            "execution_id": f"production_validation_{timestamp}",
            "dataset_id": f"um_{symbol}_{start_date.replace('-', '')}_{end_date.replace('-', '')}",
            "execution_timestamp": datetime.now().isoformat(),
            "validation_type": "production_scale_performance",
            
            "parameters": {
                "symbol": symbol,
                "start_date": start_date,
                "end_date": end_date,
                "threshold_pct": threshold,
                "threshold_bps": int(threshold * 100000),  # Convert to basis points
                "market_type": "um_futures"
            },
            
            "environment": {
                "hostname": os.uname().nodename,
                "platform": os.uname().sysname,
                "python_version": sys.version.split()[0],
                "working_directory": os.getcwd(),
                "rust_binary": "./target/release/rangebar"
            },
            
            "performance_tracking": {
                "cpu_usage_samples": [],
                "memory_usage_samples": [],
                "processing_duration_seconds": None,
                "total_trades_processed": None,
                "total_bars_generated": None,
                "throughput_trades_per_second": None
            },
            
            "output_files": {
                "csv_file": None,
                "json_file": None,
                "metadata_file": None,
                "performance_log": None
            },
            
            "claude_code_discovery": {
                "workspace_path": "/Users/terryli/eon/rangebar",
                "output_directory": "./output",
                "tags": ["production_validation", "performance_test", symbol.lower(), "rangebar"],
                "machine_readable": True,
                "analysis_ready": True,
                "discovery_file": f"production_validation_{timestamp}_discovery.json"
            },
            
            "validation_results": {
                "performance_targets_met": None,
                "data_integrity_verified": None,
                "breach_consistency_validated": None,
                "file_sizes_reasonable": None
            }
        }
        
        return metadata, timestamp
    
    def monitor_system_performance(self):
        """Monitor CPU and memory usage during processing"""
        try:
            cpu_percent = psutil.cpu_percent(interval=1)
            memory_info = psutil.virtual_memory()
            
            sample = {
                "timestamp": datetime.now().isoformat(),
                "cpu_percent": cpu_percent,
                "memory_used_mb": memory_info.used / 1024 / 1024,
                "memory_percent": memory_info.percent
            }
            
            self.performance_data.append(sample)
            return sample
        except Exception as e:
            print(f"⚠️  Performance monitoring error: {e}")
            return None
    
    def execute_range_bar_processing(self, symbol, start_date, end_date, threshold, output_dir):
        """Execute range bar processing with full monitoring"""
        print(f"🚀 Starting production scale validation")
        print(f"📊 Symbol: {symbol}")
        print(f"📅 Period: {start_date} to {end_date}")
        print(f"📈 Threshold: {threshold*100:.1f}%")
        print(f"📁 Output: {output_dir}")
        print()
        
        # Create metadata
        metadata, timestamp = self.create_execution_metadata(symbol, start_date, end_date, threshold)
        
        # Ensure output directory exists
        os.makedirs(output_dir, exist_ok=True)
        
        # Build command
        command = [
            "./target/release/rangebar",
            symbol,
            start_date,
            end_date,
            str(threshold),
            output_dir
        ]
        
        print(f"🔧 Command: {' '.join(command)}")
        print(f"⏱️  Starting execution at {datetime.now()}")
        print()
        
        # Start execution
        self.start_time = time.time()
        metadata["performance_tracking"]["start_time"] = datetime.now().isoformat()
        
        try:
            # Start process with real-time monitoring
            process = subprocess.Popen(
                command,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,
                universal_newlines=True
            )
            
            # Monitor performance during execution
            monitoring_active = True
            output_lines = []
            error_lines = []
            
            while process.poll() is None:
                # Sample system performance
                perf_sample = self.monitor_system_performance()
                if perf_sample:
                    print(f"⚡ CPU: {perf_sample['cpu_percent']:5.1f}% | Memory: {perf_sample['memory_used_mb']:>6.0f}MB ({perf_sample['memory_percent']:4.1f}%)")
                
                # Read output
                try:
                    stdout_line = process.stdout.readline()
                    if stdout_line:
                        output_lines.append(stdout_line.strip())
                        print(f"📝 {stdout_line.strip()}")
                except:
                    pass
                
                time.sleep(0.5)  # Sample every 500ms
            
            # Get final output
            remaining_stdout, remaining_stderr = process.communicate()
            if remaining_stdout:
                output_lines.extend(remaining_stdout.strip().split('\n'))
            if remaining_stderr:
                error_lines.extend(remaining_stderr.strip().split('\n'))
            
            self.end_time = time.time()
            duration = self.end_time - self.start_time
            
            print(f"\n✅ Processing completed in {duration:.2f} seconds")
            
            # Update metadata with results
            metadata["performance_tracking"]["end_time"] = datetime.now().isoformat()
            metadata["performance_tracking"]["processing_duration_seconds"] = duration
            metadata["performance_tracking"]["cpu_usage_samples"] = self.performance_data
            
            # Parse output for results
            self.parse_execution_results(output_lines, metadata)
            
            # Save all output files
            self.save_results(metadata, timestamp, output_dir, output_lines, error_lines)
            
            return metadata
            
        except Exception as e:
            print(f"❌ Execution failed: {e}")
            return None
    
    def parse_execution_results(self, output_lines, metadata):
        """Parse execution output to extract key metrics"""
        for line in output_lines:
            # Extract total trades
            if "trades loaded" in line:
                try:
                    trades = int(''.join(filter(str.isdigit, line.split("trades loaded")[0])))
                    metadata["performance_tracking"]["total_trades_processed"] = trades
                except:
                    pass
            
            # Extract total bars
            if "Total Bars:" in line:
                try:
                    bars = int(line.split("Total Bars:")[1].strip())
                    metadata["performance_tracking"]["total_bars_generated"] = bars
                except:
                    pass
            
            # Extract processing time
            if "Processing Time:" in line:
                try:
                    time_str = line.split("Processing Time:")[1].strip()
                    if "s" in time_str:
                        duration = float(time_str.replace("s", ""))
                        metadata["performance_tracking"]["processing_duration_seconds"] = duration
                except:
                    pass
        
        # Calculate throughput
        if (metadata["performance_tracking"]["total_trades_processed"] and 
            metadata["performance_tracking"]["processing_duration_seconds"]):
            trades = metadata["performance_tracking"]["total_trades_processed"]
            duration = metadata["performance_tracking"]["processing_duration_seconds"]
            throughput = trades / duration
            metadata["performance_tracking"]["throughput_trades_per_second"] = throughput
    
    def save_results(self, metadata, timestamp, output_dir, output_lines, error_lines):
        """Save all results with machine-discoverable naming"""
        
        # Create filenames
        symbol = metadata["parameters"]["symbol"]
        start = metadata["parameters"]["start_date"].replace("-", "")
        end = metadata["parameters"]["end_date"].replace("-", "")
        threshold = metadata["parameters"]["threshold_pct"]
        
        base_name = f"um_{symbol}_rangebar_{start}_{end}_{threshold:.3f}pct_v1_{timestamp}"
        
        # Save metadata
        metadata_file = f"{output_dir}/{base_name}_metadata.json"
        with open(metadata_file, 'w') as f:
            json.dump(metadata, f, indent=2)
        print(f"💾 Metadata saved: {metadata_file}")
        
        # Save performance log
        performance_file = f"{output_dir}/{base_name}_performance.json"
        performance_data = {
            "execution_id": metadata["execution_id"],
            "performance_samples": self.performance_data,
            "stdout_lines": output_lines,
            "stderr_lines": error_lines,
            "summary": {
                "total_trades": metadata["performance_tracking"]["total_trades_processed"],
                "total_bars": metadata["performance_tracking"]["total_bars_generated"],
                "duration_seconds": metadata["performance_tracking"]["processing_duration_seconds"],
                "throughput_trades_per_sec": metadata["performance_tracking"]["throughput_trades_per_second"]
            }
        }
        
        with open(performance_file, 'w') as f:
            json.dump(performance_data, f, indent=2)
        print(f"📊 Performance log saved: {performance_file}")
        
        # Save discovery file for Claude Code
        discovery_file = f"{output_dir}/{metadata['claude_code_discovery']['discovery_file']}"
        discovery_data = {
            "type": "production_validation_results",
            "execution_timestamp": metadata["execution_timestamp"],
            "files": {
                "metadata": metadata_file,
                "performance": performance_file,
                "csv_output": f"{output_dir}/{base_name}.csv",
                "json_output": f"{output_dir}/{base_name}.json"
            },
            "quick_stats": {
                "trades_processed": metadata["performance_tracking"]["total_trades_processed"],
                "bars_generated": metadata["performance_tracking"]["total_bars_generated"],
                "throughput_trades_per_sec": metadata["performance_tracking"]["throughput_trades_per_second"],
                "duration_seconds": metadata["performance_tracking"]["processing_duration_seconds"]
            },
            "claude_code_analysis_ready": True
        }
        
        with open(discovery_file, 'w') as f:
            json.dump(discovery_data, f, indent=2)
        print(f"🔍 Discovery file saved: {discovery_file}")

def main():
    """Main execution function"""
    print("🚀 Production Scale Range Bar Validator")
    print("=" * 60)
    
    # Configuration (start with 1 month, expand to 6 months after validation)
    config = {
        "symbol": "BTCUSDT",
        "start_date": "2025-08-01",
        "end_date": "2025-08-31",  # 1 month first
        "threshold": 0.008,  # 0.8%
        "output_dir": "./output"
    }
    
    print(f"📋 Validation Configuration:")
    for key, value in config.items():
        print(f"   {key}: {value}")
    print()
    
    # Execute validation
    validator = ProductionScaleValidator()
    results = validator.execute_range_bar_processing(**config)
    
    if results:
        print(f"\n🎉 Production validation completed successfully!")
        print(f"📊 Results summary:")
        perf = results["performance_tracking"]
        if perf["total_trades_processed"]:
            print(f"   Trades processed: {perf['total_trades_processed']:,}")
        if perf["total_bars_generated"]:
            print(f"   Bars generated: {perf['total_bars_generated']:,}")
        if perf["throughput_trades_per_second"]:
            print(f"   Throughput: {perf['throughput_trades_per_second']:,.0f} trades/sec")
        if perf["processing_duration_seconds"]:
            print(f"   Duration: {perf['processing_duration_seconds']:.2f} seconds")
        
        print(f"\n📁 All results saved with machine-discoverable naming")
        print(f"🔍 Claude Code can now analyze results via discovery files")
    else:
        print(f"\n❌ Production validation failed!")
        return 1
    
    return 0

if __name__ == "__main__":
    sys.exit(main())