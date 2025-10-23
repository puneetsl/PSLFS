# PSLFS Quick Start Guide

Get up and running with PSLFS in under 2 minutes!

## Prerequisites

- GCC compiler
- A text editor (nano, vim, or vi)
- Linux, macOS, or Windows with MinGW/WSL

## Step 1: Build

```bash
make all
```

Or manually:
```bash
gcc -Wall -o base basic.c
gcc -Wall -o fbase fbase.c
```

## Step 2: Initialize Filesystem

```bash
./base
```

You'll be prompted to:
1. Enter a username (e.g., "admin")
2. Enter a password (will be hidden as you type)

## Step 3: Start Using PSLFS

```bash
./fbase
```

Login with the credentials you just created.

## Step 4: Try Some Commands

Once you're in the PSLFS prompt, try:

```bash
# See available commands
help

# Create a directory
md Documents

# Change into it
cd Documents

# Create a file (will open your default editor)
mf README.txt

# List directory contents
ls

# Show file contents
show README.txt

# Go back to parent directory
cd ..

# Return to root
cd /

# Exit filesystem
quit
```

## Common Issues

### "Authentication file not found"
You need to run `./base` first to create the filesystem and user.

### "Editor not found"
Set your preferred editor:
```bash
export EDITOR=nano
./fbase
```

### "Permission denied" when creating files
The directory might not be writable. Check permissions with the current directory's properties.

## Next Steps

- Read [README.md](README.md) for detailed documentation
- Check [IMPROVEMENTS.md](IMPROVEMENTS.md) to see what was fixed
- Read [CLAUDE.md](CLAUDE.md) for architecture details

## Quick Reset

To start fresh:
```bash
make clean  # Removes all filesystem files
./base      # Reinitialize
```

Enjoy exploring PSLFS!
