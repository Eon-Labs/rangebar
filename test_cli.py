#!/usr/bin/env python3
"""
Test script for the CLI interface.
Tests complete end-to-end workflow from data fetch to range bar generation.
"""

import sys
import tempfile
from pathlib import Path
import subprocess
import os

def run_cli_command(command_args, env_vars=None):
    """Run CLI command and return result."""
    env = os.environ.copy()
    if env_vars:
        env.update(env_vars)
    
    # Add src to Python path
    env['PYTHONPATH'] = str(Path(__file__).parent / "src")
    
    cmd = ["uv", "run", "--active", "python", "-m", "rangebar.cli"] + command_args
    
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            env=env,
            cwd=Path(__file__).parent
        )
        return result
    except Exception as e:
        print(f"Error running command {' '.join(cmd)}: {e}")
        return None


def test_cli_help():
    """Test CLI help commands."""
    print("🧪 Testing CLI help commands...")
    
    # Test main help
    result = run_cli_command(["--help"])
    if not result or result.returncode != 0:
        print(f"❌ Main help failed: {result.stderr if result else 'No result'}")
        return False
    
    if "Range Bar CLI" not in result.stdout:
        print("❌ Main help missing expected content")
        return False
    
    # Test command helps
    commands = ["fetch", "generate", "inspect", "cleanup"]
    for cmd in commands:
        result = run_cli_command([cmd, "--help"])
        if not result or result.returncode != 0:
            print(f"❌ Help for {cmd} failed: {result.stderr if result else 'No result'}")
            return False
    
    print("✅ All CLI help commands work")
    return True


def test_cli_fetch():
    """Test CLI data fetch functionality."""
    print("🧪 Testing CLI fetch command...")
    
    with tempfile.TemporaryDirectory() as temp_dir:
        try:
            # Test fetch with small date range
            result = run_cli_command([
                "fetch", 
                "BTCUSDT", 
                "2024-01-01", 
                "2024-01-01",
                "--output-dir", temp_dir,
                "--format", "parquet"
            ])
            
            if not result:
                print("❌ Fetch command failed to run")
                return False
            
            if result.returncode != 0:
                print(f"❌ Fetch command failed: {result.stderr}")
                return False
            
            # Check if output file was created
            output_files = list(Path(temp_dir).glob("*.parquet"))
            if not output_files:
                print("❌ No output files created")
                return False
            
            output_file = output_files[0]
            if not output_file.exists():
                print(f"❌ Output file not found: {output_file}")
                return False
            
            file_size = output_file.stat().st_size
            if file_size == 0:
                print("❌ Output file is empty")
                return False
            
            print(f"✅ Fetch created: {output_file.name} ({file_size:,} bytes)")
            return True, output_file
            
        except Exception as e:
            print(f"❌ Fetch test failed: {e}")
            return False


def test_cli_generate():
    """Test CLI range bar generation."""
    print("🧪 Testing CLI generate command...")
    
    with tempfile.TemporaryDirectory() as temp_dir:
        try:
            # Test generate with direct fetch
            result = run_cli_command([
                "generate",
                "BTCUSDT",
                "2024-01-01",
                "2024-01-01", 
                "--threshold", "0.8",
                "--output-dir", temp_dir,
                "--max-trades", "1000"  # Limit for faster testing
            ])
            
            if not result:
                print("❌ Generate command failed to run")
                return False
            
            if result.returncode != 0:
                print(f"❌ Generate command failed: {result.stderr}")
                return False
            
            # Check if range bar file was created
            output_files = list(Path(temp_dir).glob("*range_bars*.parquet"))
            if not output_files:
                print("❌ No range bar files created")
                return False
            
            output_file = output_files[0]
            file_size = output_file.stat().st_size
            
            if file_size == 0:
                print("❌ Range bar file is empty")
                return False
            
            print(f"✅ Generate created: {output_file.name} ({file_size:,} bytes)")
            return True, output_file
            
        except Exception as e:
            print(f"❌ Generate test failed: {e}")
            return False


def test_cli_inspect():
    """Test CLI inspect functionality."""
    print("🧪 Testing CLI inspect command...")
    
    # First create a test file
    fetch_result = test_cli_fetch()
    if not fetch_result or not isinstance(fetch_result, tuple):
        print("❌ Cannot test inspect without test file")
        return False
    
    success, test_file = fetch_result
    if not success:
        return False
    
    try:
        # Test inspect command
        result = run_cli_command([
            "inspect",
            str(test_file),
            "--limit", "5"
        ])
        
        if not result:
            print("❌ Inspect command failed to run")
            return False
        
        if result.returncode != 0:
            print(f"❌ Inspect command failed: {result.stderr}")
            return False
        
        if "Inspecting:" not in result.stdout:
            print("❌ Inspect output missing expected content")
            return False
        
        print("✅ Inspect command works")
        return True
        
    except Exception as e:
        print(f"❌ Inspect test failed: {e}")
        return False


def test_cli_cleanup():
    """Test CLI cleanup functionality."""
    print("🧪 Testing CLI cleanup command...")
    
    with tempfile.TemporaryDirectory() as temp_dir:
        try:
            # Create a test file
            test_file = Path(temp_dir) / "test.parquet"
            test_file.write_text("test content")
            
            # Test dry-run cleanup
            result = run_cli_command([
                "cleanup",
                "--data-dir", temp_dir,
                "--older-than", "0",  # Everything is older than 0 days
                "--dry-run"
            ])
            
            if not result:
                print("❌ Cleanup dry-run failed to run")
                return False
            
            if result.returncode != 0:
                print(f"❌ Cleanup dry-run failed: {result.stderr}")
                return False
            
            # File should still exist after dry-run
            if not test_file.exists():
                print("❌ File was deleted during dry-run")
                return False
            
            print("✅ Cleanup dry-run works")
            return True
            
        except Exception as e:
            print(f"❌ Cleanup test failed: {e}")
            return False


def test_error_handling():
    """Test CLI error handling."""
    print("🧪 Testing CLI error handling...")
    
    try:
        # Test invalid symbol
        result = run_cli_command([
            "fetch",
            "INVALID",
            "2024-01-01",
            "2024-01-01"
        ])
        
        # Should fail gracefully
        if not result or result.returncode == 0:
            print("❌ Invalid symbol should have failed")
            return False
        
        # Test invalid date format
        result = run_cli_command([
            "fetch", 
            "BTCUSDT",
            "invalid-date",
            "2024-01-01"
        ])
        
        # Should fail gracefully
        if not result or result.returncode == 0:
            print("❌ Invalid date should have failed")
            return False
        
        print("✅ Error handling works")
        return True
        
    except Exception as e:
        print(f"❌ Error handling test failed: {e}")
        return False


async def main():
    """Run all CLI tests."""
    print("🚀 Starting CLI tests...")
    print()
    
    tests = [
        ("CLI Help Commands", test_cli_help),
        ("CLI Fetch Command", test_cli_fetch),
        ("CLI Generate Command", test_cli_generate), 
        ("CLI Inspect Command", test_cli_inspect),
        ("CLI Cleanup Command", test_cli_cleanup),
        ("CLI Error Handling", test_error_handling),
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        print(f"Running: {test_name}")
        try:
            result = test_func()
            if isinstance(result, tuple):
                result = result[0]  # Extract boolean from tuple
            
            if result:
                passed += 1
                print("✅ PASSED\n")
            else:
                print("❌ FAILED\n")
        except Exception as e:
            print(f"❌ FAILED - Exception: {e}\n")
    
    print(f"🎯 Test Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 ALL CLI TESTS PASSED!")
        print()
        print("SUCCESS SUMMARY:")
        print("✅ CLI interface fully implemented")
        print("✅ Data fetch command working")
        print("✅ Range bar generation command working") 
        print("✅ Parquet inspection working")
        print("✅ File cleanup working")
        print("✅ Error handling working properly")
        print()
        print("CLI COMMANDS AVAILABLE:")
        print("• rangebar fetch SYMBOL START END     - Fetch aggTrades data")
        print("• rangebar generate SYMBOL START END  - Generate range bars")
        print("• rangebar inspect FILE               - Inspect Parquet files")
        print("• rangebar cleanup                   - Clean old files")
        return True
    else:
        print(f"❌ {total - passed} tests failed")
        return False


if __name__ == "__main__":
    import asyncio
    success = asyncio.run(main())
    sys.exit(0 if success else 1)