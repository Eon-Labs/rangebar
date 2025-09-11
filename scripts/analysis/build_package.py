#!/usr/bin/env python3
"""
Test script to build the package for PyPI distribution.
"""

import subprocess
import sys
from pathlib import Path

def run_command(cmd, description):
    """Run a command and handle errors."""
    print(f"🔄 {description}...")
    try:
        result = subprocess.run(cmd, shell=True, check=True, capture_output=True, text=True)
        print(f"✅ {description} completed")
        if result.stdout:
            print(f"Output: {result.stdout.strip()}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ {description} failed")
        print(f"Error: {e.stderr.strip()}")
        return False

def main():
    """Build and validate the package."""
    print("🚀 Building RangeBar package for PyPI")
    print("=" * 50)
    
    # Clean previous builds
    if not run_command("rm -rf dist/ build/ target/wheels/", "Cleaning previous builds"):
        return False
    
    # Build with maturin (for Rust extension)
    if not run_command("uv run maturin build --release", "Building Rust extension with maturin"):
        return False
    
    # Check if wheel was created
    wheels = list(Path("target/wheels").glob("*.whl")) if Path("target/wheels").exists() else []
    if not wheels:
        print("❌ No wheel file found after build")
        return False
    
    wheel_file = wheels[0]
    print(f"✅ Wheel built: {wheel_file}")
    print(f"   Size: {wheel_file.stat().st_size:,} bytes")
    
    # Validate package metadata
    if not run_command(f"uv run python -m pip show rangebar || echo 'Not installed yet'", "Checking current installation"):
        pass  # This is expected to fail if not installed
    
    print("\n🎉 Package build completed successfully!")
    print("\nNext steps for PyPI publication:")
    print("1. Test install: uv pip install target/wheels/*.whl")
    print("2. Upload to PyPI: uv run maturin publish")
    print("3. Or upload manually: twine upload target/wheels/*.whl")
    
    return True

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)