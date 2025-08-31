"""
Setup script for AlphaForge PyO3 Python extension.

This builds the Rust-powered Python extension module that provides
high-performance data processing and caching capabilities.
"""

import os
from setuptools import setup, find_packages
from setuptools_rust import Binding, RustExtension

# Get version from Cargo.toml
def get_version():
    """Extract version from Cargo.toml"""
    try:
        import toml
        with open("Cargo.toml", "r") as f:
            cargo = toml.load(f)
        return cargo["workspace"]["package"]["version"]
    except:
        return "1.0.0"

def get_long_description():
    """Get long description from README"""
    try:
        with open("README.md", "r", encoding="utf-8") as f:
            return f.read()
    except:
        return "AlphaForge - High-performance algorithmic trading system"

# Rust extension configuration
rust_extensions = [
    RustExtension(
        "alphaforge.core.rust",
        path="crates/pyo3/Cargo.toml",
        binding=Binding.PyO3,
        debug=False,  # Set to True for debug builds
        rust_version=">=1.65.0",
        features=["default"],
    ),
]

# Python package setup
setup(
    name="alphaforge",
    version=get_version(),
    description="High-performance algorithmic trading system with Rust-powered core",
    long_description=get_long_description(),
    long_description_content_type="text/markdown",
    author="AlphaForge Team",
    author_email="team@alphaforge.dev",
    url="https://github.com/alphaforge/alphaforge",
    license="MIT",
    
    # Python package configuration
    packages=find_packages(where="python"),
    package_dir={"": "python"},
    python_requires=">=3.8",
    
    # Rust extension
    rust_extensions=rust_extensions,
    zip_safe=False,
    
    # Dependencies
    install_requires=[
        "numpy>=1.21.0",
        "pandas>=1.3.0",
        "pyarrow>=5.0.0",
    ],
    
    # Development dependencies  
    extras_require={
        "dev": [
            "pytest>=6.0",
            "pytest-benchmark>=3.4.0",
            "black>=21.0.0",
            "isort>=5.0.0",
            "mypy>=0.910",
        ],
        "test": [
            "pytest>=6.0",
            "pytest-benchmark>=3.4.0",
        ],
    },
    
    # Entry points
    entry_points={
        "console_scripts": [
            "alphaforge=alphaforge.cli:main",
        ],
    },
    
    # Metadata
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Financial and Insurance Industry",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Rust",
        "Topic :: Office/Business :: Financial :: Investment",
        "Topic :: Scientific/Engineering :: Information Analysis",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    
    # Keywords
    keywords=[
        "trading", "algorithmic-trading", "quantitative-finance",
        "high-frequency-trading", "market-data", "backtesting",
        "rust", "performance", "low-latency"
    ],
    
    # Build requirements
    setup_requires=["setuptools-rust>=1.5.1", "toml>=0.10.0"],
    
    # Include package data
    include_package_data=True,
    package_data={
        "alphaforge": ["py.typed"],
    },
)
