# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PSLFS is a virtual file system implementation in C that simulates a complete filesystem with user authentication, file/folder management, and permissions. The system stores its data in binary files with custom sector-based allocation.

## Build and Run Commands

### Compilation
The project has been updated for cross-platform compatibility (Linux, macOS, Windows with MinGW):

```bash
# Using Makefile (recommended)
make all          # Build both executables
make clean        # Clean build artifacts

# Manual compilation
gcc -Wall -o base basic.c
gcc -Wall -o fbase fbase.c
```

### Running the System

1. **Initialize the filesystem**: Run `base` first to create a new filesystem and register a user
   ```bash
   ./base
   ```
   This creates four binary files: `test1.psl` (main partition), `test1.fol` (folder free sectors), `test1.fil` (file free sectors), `test1.fs` (file content sectors), plus `authent` (user credentials)

2. **Use the filesystem**: Run `fbase` to interact with the filesystem
   ```bash
   ./fbase
   ```
   Login with the username/password created during initialization

## Core Architecture

### Binary File Storage Structure

The filesystem uses four binary files for storage:
- **`.psl` file**: Main partition containing all folder and file metadata structures
- **`.fol` file**: Free sector list for folders
- **`.fil` file**: Free sector list for files
- **`.fs` file**: Free sector list for file content blocks

### Key Data Structures

**ffolder** (folder structure in `fstest1.h`, `fstest.h`, `testio.h`):
- Linked list of folders with `next`/`prev` pointers for siblings
- `insector`: points to first child folder
- `filesector`: points to first file in this folder
- `upsector`: points to parent folder
- `properties[3]`: read/write/visibility permissions
- `sector`: location in `.psl` file

**ffile** (file structure):
- Linked list with `next`/`prev` pointers
- `fsector`: points to first content block in `.fs` file
- `properties[4]`: file permissions
- `sector`: location in `.psl` file

**wfile** (file content blocks):
- 16-byte chunks linked together
- Forms linked list for file contents using `next`/`prev`

**freesec** (free sector management):
- Linked list tracking available sectors in each binary file
- Used for allocation/deallocation

### Memory Management

The system maintains three free sector lists in memory:
- `foldsec`: free sectors for folder metadata
- `filesec`: free sectors for file metadata
- `fsec`: free sectors for file content blocks

These are initialized by `invokeFreeSectors()` in fbase.c:32 and used throughout to allocate/deallocate space.

### User Authentication

- User credentials stored in binary file header (`diskheader` in partition.h:2-12)
- Password encryption using simple character shift based on first 3 chars of username (partition.h:79-102)
- `Authenticate()` function validates user before filesystem access
- `makeUser()` creates initial user during filesystem setup

### File/Folder Operations

All operations go through sector-based I/O:
- `readFolder()`: reads folder metadata from sector position
- `writeFolder()`: allocates sector and writes folder metadata
- `foldTravel()`: traverses folder linked list
- Similar patterns for file operations

Navigation uses sector pointers rather than paths internally, enabling efficient traversal.

## Command Line Interface (fbase.c)

The main loop (fbase.c:33-273) implements a shell-like interface:

**Directory commands**:
- `ls` / `showdir`: list directory contents
- `cd <name>`: change directory (supports `/`, `..`, `-n`/`-p` for next/prev sibling)
- `md <name>`: create directory

**File commands**:
- `mf <name>`: create file (opens external editor)
- `show <name>`: display file contents
- `edit <name>`: edit file (choice of editors)
- `del <name>`: delete file or folder
- `df <name>`: delete file specifically

**Permission commands**:
- `cfmode <name>`: change file mode
- `cdmode <name>`: change directory mode

**Other**:
- `quit`: exit filesystem
- `clear`: clear screen
- `help`: show available commands
- `web <url>`: open URL in default browser

## Important Implementation Notes

### Multiple Header Versions

There are three versions of the filesystem core:
- `fstest.h`: original implementation
- `fstest1.h`: enhanced version with authentication and free sector management (used by fbase.c)
- `testio.h`: alternative implementation
- `conpsl.h`: contains basic data structure definitions and in-memory folder/file tree

The current system uses `fstest1.h` (included in basic.c:6 and fbase.c:8).

### Platform Compatibility

The code has been modernized for cross-platform support:
- Removed Windows-specific dependencies (`windows.h`, `ShellExecute`)
- `ds.h` provides Unix terminal compatibility (`getch()`, `clrscr()`)
- Editor integration respects `$EDITOR` environment variable, falls back to `nano`/`vi`/`vim`
- URL opening uses `xdg-open` (Linux) or `open` (macOS)
- Replaced unsafe `gets()` with `fgets()` throughout
- Improved error handling with null checks and proper file closures

### Hardcoded Paths

The default filesystem name is hardcoded in fbase.c:14-24 as `"test1"` with extensions `.psl`, `.fol`, `.fil`, `.fs`. To use different filesystem names, modify these path variables.

### Sector-Based Architecture

The entire system uses file offsets as "sectors" - essentially treating binary files as a custom block device. All pointers between structures are file offsets, enabling direct fseek/fread/fwrite operations.
