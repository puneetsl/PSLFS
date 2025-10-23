# PSLFS - Virtual File System

A virtual file system implementation in C that simulates a complete filesystem with user authentication, file/folder management, and permissions.

## Features

- User authentication with encrypted passwords
- Hierarchical folder structure
- File creation, editing, and deletion
- Permission-based access control (read/write/visibility)
- Sector-based storage in binary files
- Cross-platform support (Linux, macOS, Windows with MinGW)

## Building

### Using Make (Recommended)

```bash
make all          # Build both base and fbase executables
make clean        # Clean build artifacts and filesystem files
```

### Manual Compilation

```bash
gcc -Wall -o base basic.c
gcc -Wall -o fbase fbase.c
```

## Usage

### 1. Initialize the Filesystem

First, run `base` to create a new filesystem and register a user:

```bash
./base
```

This creates:
- `test1.psl` - Main partition containing folder/file metadata
- `test1.fol` - Folder free sector list
- `test1.fil` - File free sector list
- `test1.fs` - File content free sector list
- `authent` - User authentication file

### 2. Use the Filesystem

Run `fbase` to interact with the filesystem:

```bash
./fbase
```

Enter the username and password you created during initialization.

## Available Commands

- `ls` or `showdir` - List directory contents
- `cd <dir>` - Change directory (supports `..` for parent, `/` for root, `-n`/`-p` for next/prev sibling)
- `md <name>` - Create directory
- `mf <name>` - Create file (opens editor)
- `show <file>` - Display file contents
- `edit <file>` - Edit file
- `del <name>` - Delete file or folder
- `df <file>` - Delete file specifically
- `cfmode <file>` - Change file permissions
- `cdmode <dir>` - Change directory permissions
- `web <url>` - Open URL in default browser
- `clear` - Clear screen
- `help` - Show available commands
- `quit` - Exit filesystem

## Editor Support

The filesystem uses your system's text editor:
- Checks `$EDITOR` environment variable first
- Falls back to `nano`, `vi`, or `vim` (whichever is available)

## Mounting as a Real Linux Filesystem (FUSE)

You can mount PSLFS as a real filesystem using FUSE! See [FUSE_MOUNT_GUIDE.md](FUSE_MOUNT_GUIDE.md) for details.

```bash
# Quick start:
sudo apt-get install fuse3 libfuse3-dev
gcc -o pslfs_fuse pslfs_fuse_simple.c `pkg-config fuse3 --cflags --libs`
mkdir /tmp/pslfs_mount
./pslfs_fuse /tmp/pslfs_mount

# Now use it like any filesystem:
ls /tmp/pslfs_mount
echo "Hello" > /tmp/pslfs_mount/test.txt

# Unmount:
fusermount3 -u /tmp/pslfs_mount
```

### Performance

PSLFS is 20-100x slower than ext4 due to:
- No caching
- Linked list traversal (O(n) operations)
- Small 16-byte blocks
- High syscall overhead

But it's a great learning tool! Run `./benchmark_pslfs.sh` to see detailed performance analysis.

## Security Note

This is an educational project. The password encryption is simple (character shift) and not suitable for production use.

