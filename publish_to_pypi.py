#!/usr/bin/env python3
"""
AlphaForge PyPI Publishing Script
Created by Krishna Bajpai (krishna@krishnabajpai.me) and Vedanshi Gupta (vedanshigupta158@gmail.com)

This script helps build and publish AlphaForge to PyPI.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

def run_command(cmd, description):
    """Run a command and handle errors"""
    print(f"\n🚀 {description}")
    print(f"Running: {cmd}")
    print("-" * 60)
    
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    if result.stdout:
        print("STDOUT:", result.stdout)
    if result.stderr:
        print("STDERR:", result.stderr)
    
    if result.returncode != 0:
        print(f"❌ Command failed with return code {result.returncode}")
        sys.exit(1)
    
    print(f"✅ {description} completed successfully")
    return result

def clean_build_artifacts():
    """Clean previous build artifacts"""
    print("\n🧹 Cleaning build artifacts...")
    
    dirs_to_clean = ['dist', 'build', 'target/wheels', '*.egg-info']
    
    for pattern in dirs_to_clean:
        for path in Path('.').glob(pattern):
            if path.is_dir():
                print(f"  Removing directory: {path}")
                shutil.rmtree(path, ignore_errors=True)
            elif path.is_file():
                print(f"  Removing file: {path}")
                path.unlink(missing_ok=True)
    
    print("✅ Build artifacts cleaned")

def verify_requirements():
    """Verify that required tools are installed"""
    print("\n🔍 Verifying requirements...")
    
    required_tools = [
        ('python', 'Python interpreter'),
        ('maturin', 'Maturin build tool'),
        ('twine', 'PyPI upload tool (optional)'),
    ]
    
    missing_tools = []
    
    for tool, description in required_tools:
        try:
            result = subprocess.run([tool, '--version'], 
                                  capture_output=True, text=True)
            if result.returncode == 0:
                version = result.stdout.strip().split('\n')[0]
                print(f"  ✅ {description}: {version}")
            else:
                missing_tools.append((tool, description))
        except FileNotFoundError:
            missing_tools.append((tool, description))
    
    if missing_tools:
        print("\n❌ Missing required tools:")
        for tool, description in missing_tools:
            print(f"  - {tool}: {description}")
        
        print("\n📦 Install missing tools:")
        if any(tool == 'maturin' for tool, _ in missing_tools):
            print("  pip install maturin")
        if any(tool == 'twine' for tool, _ in missing_tools):
            print("  pip install twine")
        
        sys.exit(1)
    
    print("✅ All requirements verified")

def build_package():
    """Build the Python package using maturin"""
    clean_build_artifacts()
    
    # Build source distribution
    run_command(
        "maturin sdist",
        "Building source distribution (sdist)"
    )
    
    # Build wheel for current platform
    run_command(
        "maturin build --release --strip",
        "Building wheel distribution"
    )
    
    # List built packages
    print("\n📦 Built packages:")
    dist_path = Path('dist')
    if dist_path.exists():
        for file in dist_path.iterdir():
            print(f"  - {file.name} ({file.stat().st_size:,} bytes)")
    else:
        print("  ❌ No dist/ directory found")

def test_package():
    """Test the built package"""
    print("\n🧪 Testing built package...")
    
    # Create a temporary virtual environment for testing
    run_command(
        "python -m venv test_env",
        "Creating test virtual environment"
    )
    
    # Activate and install the package
    if os.name == 'nt':  # Windows
        activate_cmd = "test_env\\Scripts\\activate"
        pip_cmd = "test_env\\Scripts\\pip"
    else:  # Unix-like
        activate_cmd = "source test_env/bin/activate"
        pip_cmd = "test_env/bin/pip"
    
    # Find the wheel file
    wheel_files = list(Path('dist').glob('*.whl'))
    if not wheel_files:
        print("❌ No wheel file found for testing")
        return
    
    wheel_file = wheel_files[0]
    
    run_command(
        f"{pip_cmd} install {wheel_file}",
        "Installing built package for testing"
    )
    
    # Test import
    test_script = """
import sys
try:
    import alphaforge
    print("✅ AlphaForge imported successfully")
    print(f"   Version: {getattr(alphaforge, '__version__', 'unknown')}")
    print(f"   Authors: {getattr(alphaforge, '__author__', 'Krishna Bajpai and Vedanshi Gupta')}")
except ImportError as e:
    print(f"❌ Import failed: {e}")
    sys.exit(1)
"""
    
    with open('test_import.py', 'w') as f:
        f.write(test_script)
    
    if os.name == 'nt':
        test_cmd = f"test_env\\Scripts\\python test_import.py"
    else:
        test_cmd = f"test_env/bin/python test_import.py"
    
    run_command(test_cmd, "Testing package import")
    
    # Cleanup
    shutil.rmtree('test_env', ignore_errors=True)
    Path('test_import.py').unlink(missing_ok=True)
    
    print("✅ Package testing completed")

def publish_to_pypi(test=True):
    """Publish to PyPI (or Test PyPI)"""
    
    if test:
        print("\n🚀 Publishing to Test PyPI...")
        repository_url = "https://test.pypi.org/legacy/"
        print("📝 You'll need Test PyPI credentials")
        print("   Create account at: https://test.pypi.org/account/register/")
    else:
        print("\n🚀 Publishing to PyPI...")
        repository_url = "https://upload.pypi.org/legacy/"
        print("📝 You'll need PyPI credentials")
        print("   Create account at: https://pypi.org/account/register/")
    
    print("\n⚠️  WARNING: This will upload to the public package index!")
    response = input("Continue? (yes/no): ").strip().lower()
    
    if response != 'yes':
        print("❌ Upload cancelled")
        return
    
    # Use twine to upload
    upload_cmd = f"twine upload --repository-url {repository_url} dist/*"
    
    print(f"\n📤 Uploading to {'Test PyPI' if test else 'PyPI'}...")
    print("You will be prompted for your username and password.")
    
    result = subprocess.run(upload_cmd, shell=True)
    
    if result.returncode == 0:
        if test:
            print("✅ Successfully uploaded to Test PyPI!")
            print("🔗 Check your package at: https://test.pypi.org/project/alphaforge/")
            print("🧪 Test installation with:")
            print("   pip install -i https://test.pypi.org/simple/ alphaforge")
        else:
            print("✅ Successfully uploaded to PyPI!")
            print("🔗 Check your package at: https://pypi.org/project/alphaforge/")
            print("📦 Install with: pip install alphaforge")
    else:
        print("❌ Upload failed")

def main():
    """Main function"""
    print("🚀 AlphaForge PyPI Publisher")
    print("=" * 60)
    print("Authors: Krishna Bajpai (krishna@krishnabajpai.me)")
    print("         Vedanshi Gupta (vedanshigupta158@gmail.com)")
    print("=" * 60)
    
    if len(sys.argv) < 2:
        print("\nUsage:")
        print("  python publish_to_pypi.py build       # Build package only")
        print("  python publish_to_pypi.py test        # Build and test")
        print("  python publish_to_pypi.py test-pypi   # Build, test, and upload to Test PyPI")
        print("  python publish_to_pypi.py pypi        # Build, test, and upload to PyPI")
        print("  python publish_to_pypi.py clean       # Clean build artifacts")
        return
    
    command = sys.argv[1].lower()
    
    if command == 'clean':
        clean_build_artifacts()
        return
    
    # Verify tools are available
    verify_requirements()
    
    if command in ['build', 'test', 'test-pypi', 'pypi']:
        build_package()
    
    if command in ['test', 'test-pypi', 'pypi']:
        test_package()
    
    if command == 'test-pypi':
        publish_to_pypi(test=True)
    elif command == 'pypi':
        publish_to_pypi(test=False)
    
    print("\n🎉 All operations completed successfully!")
    print("\n📚 Next steps:")
    if command == 'build':
        print("  - Test your package with: python publish_to_pypi.py test")
        print("  - Upload to Test PyPI with: python publish_to_pypi.py test-pypi")
    elif command == 'test':
        print("  - Upload to Test PyPI with: python publish_to_pypi.py test-pypi")
    elif command == 'test-pypi':
        print("  - Upload to PyPI with: python publish_to_pypi.py pypi")
    elif command == 'pypi':
        print("  - Install your package with: pip install alphaforge")
        print("  - Share with the world! 🌍")

if __name__ == '__main__':
    main()
