#!/bin/bash

# Test script for PSLFS

echo "=========================================="
echo "PSLFS System Test"
echo "=========================================="
echo

# Check if executables exist
if [ ! -f "./base" ] || [ ! -f "./fbase" ]; then
    echo "Error: Executables not found. Run 'make all' first."
    exit 1
fi

echo "✓ Executables found"
echo

# Check if filesystem already exists
if [ -f "test1.psl" ]; then
    echo "⚠ Filesystem already exists"
    echo "Files found:"
    ls -lh test1.* authent 2>/dev/null
    echo
    read -p "Do you want to reinitialize? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Cleaning up old filesystem..."
        rm -f test1.* authent alpha alpha.txt
        echo "✓ Cleanup complete"
        echo
    else
        echo "Using existing filesystem"
        echo "Run './fbase' to access it"
        exit 0
    fi
fi

echo "This script will help you test the filesystem."
echo
echo "Press Ctrl+C at any time to cancel"
echo
echo "Starting filesystem initialization..."
echo "You'll be asked to create a username and password"
echo
echo "=========================================="
./base

# Check if initialization succeeded
if [ $? -eq 0 ] && [ -f "test1.psl" ]; then
    echo
    echo "=========================================="
    echo "✓ Filesystem initialized successfully!"
    echo
    echo "Files created:"
    ls -lh test1.* authent
    echo
    echo "To use the filesystem, run:"
    echo "  ./fbase"
    echo
    echo "Type 'help' once inside to see available commands"
    echo "=========================================="
else
    echo
    echo "✗ Filesystem initialization failed"
    exit 1
fi
