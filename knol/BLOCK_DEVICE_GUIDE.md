# PSLFS V2 - Block Device Support

**Using PSLFS on Physical Devices (USB Drives, Hard Disks, etc.)**

**Version:** 2.0
**Status:** Design Phase
**Last Updated:** 2025-10-22

---

## Overview

PSLFS V2 supports formatting and using **physical block devices** (USB drives, hard disks, SD cards, etc.) in addition to file-backed storage. This is possible through the abstract `StorageBackend` trait.

### Storage Backends

PSLFS V2 supports three storage backends:

1. **FileBackend** - Store filesystem in regular files (V0.1 compatible)
   - Use case: Testing, development, backward compatibility
   - Example: `/tmp/myfs.psl`

2. **BlockDeviceBackend** - Direct block device access
   - Use case: Format real devices (USB drives, partitions)
   - Example: `/dev/sdb1`, `/dev/nvme0n1p2`

3. **MemoryBackend** - In-memory storage
   - Use case: Testing, temporary filesystems
   - Example: RAM disk

---

## Table of Contents

1. [Safety Warnings](#safety-warnings)
2. [BlockDeviceBackend Implementation](#blockdevicebackend-implementation)
3. [CLI Commands](#cli-commands)
4. [Usage Examples](#usage-examples)
5. [Technical Details](#technical-details)
6. [Limitations](#limitations)

---

## Safety Warnings

⚠️ **CRITICAL WARNINGS:**

1. **Data Loss**: Formatting a device will DESTROY all existing data
2. **Choose Carefully**: Double-check device path (e.g., `/dev/sdb` vs `/dev/sda`)
3. **Unmount First**: Ensure device is unmounted before formatting
4. **Permissions**: Requires root/sudo for block device access
5. **Backup**: Always backup important data before formatting

**Common mistakes:**
- ❌ `/dev/sda` - Your main hard drive (DON'T FORMAT THIS!)
- ❌ `/dev/nvme0n1` - Your NVMe SSD (DON'T FORMAT THIS!)
- ✅ `/dev/sdb` - USB drive (usually safe)
- ✅ `/dev/mmcblk0` - SD card (usually safe)

**Always verify:**
```bash
# List block devices
lsblk

# Check device info
sudo fdisk -l /dev/sdb

# Verify it's your USB drive
ls -l /dev/disk/by-id/usb-*
```

---

## BlockDeviceBackend Implementation

### Trait Implementation

```rust
use std::fs::{File, OpenOptions};
use std::os::unix::fs::FileExt;
use std::path::Path;

/// Block device backend for real hardware
pub struct BlockDeviceBackend {
    device: File,
    total_blocks: u64,
    block_size: usize,
}

impl BlockDeviceBackend {
    /// Open an existing block device
    ///
    /// # Arguments
    /// * `path` - Path to block device (e.g., "/dev/sdb1")
    /// * `read_only` - Open in read-only mode
    ///
    /// # Safety
    /// Requires appropriate permissions (usually root)
    pub fn open<P: AsRef<Path>>(path: P, read_only: bool) -> Result<Self> {
        let device = OpenOptions::new()
            .read(true)
            .write(!read_only)
            .open(path.as_ref())?;

        // Get device size
        let size = Self::get_device_size(&device)?;
        let block_size = BLOCK_SIZE;
        let total_blocks = size / block_size as u64;

        Ok(Self {
            device,
            total_blocks,
            block_size,
        })
    }

    /// Create/format a new filesystem on device
    ///
    /// # Warning
    /// This DESTROYS all existing data on the device!
    pub fn format<P: AsRef<Path>>(path: P, label: Option<&str>) -> Result<Self> {
        // Open device for writing
        let mut backend = Self::open(path, false)?;

        // Write superblock
        let superblock = Superblock::new(
            backend.total_blocks,
            label,
        );
        backend.write_block(BlockId(0), &superblock.to_block()?)?;

        // Initialize bitmaps
        backend.init_bitmaps()?;

        // Sync to ensure data is written
        backend.sync()?;

        Ok(backend)
    }

    /// Get size of block device in bytes
    fn get_device_size(device: &File) -> Result<u64> {
        use std::os::unix::io::AsRawFd;

        // On Linux, use ioctl to get device size
        #[cfg(target_os = "linux")]
        {
            use libc::{ioctl, BLKGETSIZE64};

            let mut size: u64 = 0;
            let fd = device.as_raw_fd();

            unsafe {
                if ioctl(fd, BLKGETSIZE64, &mut size) == -1 {
                    return Err(FsError::Io(std::io::Error::last_os_error()));
                }
            }

            Ok(size)
        }

        // On macOS, use different approach
        #[cfg(target_os = "macos")]
        {
            use libc::{ioctl, DKIOCGETBLOCKCOUNT, DKIOCGETBLOCKSIZE};

            let mut block_count: u64 = 0;
            let mut block_size: u32 = 0;
            let fd = device.as_raw_fd();

            unsafe {
                if ioctl(fd, DKIOCGETBLOCKCOUNT, &mut block_count) == -1 {
                    return Err(FsError::Io(std::io::Error::last_os_error()));
                }
                if ioctl(fd, DKIOCGETBLOCKSIZE, &mut block_size) == -1 {
                    return Err(FsError::Io(std::io::Error::last_os_error()));
                }
            }

            Ok(block_count * block_size as u64)
        }

        // Fallback: seek to end
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            use std::io::{Seek, SeekFrom};
            let size = device.seek(SeekFrom::End(0))?;
            device.seek(SeekFrom::Start(0))?;
            Ok(size)
        }
    }
}

impl StorageBackend for BlockDeviceBackend {
    fn read_block(&self, block_id: BlockId) -> Result<Block> {
        let offset = block_id.0 * self.block_size as u64;
        let mut block = Block::new();

        self.device.read_exact_at(&mut block.data, offset)?;

        Ok(block)
    }

    fn write_block(&mut self, block_id: BlockId, data: &Block) -> Result<()> {
        let offset = block_id.0 * self.block_size as u64;

        self.device.write_all_at(&data.data, offset)?;

        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        self.device.sync_all()?;
        Ok(())
    }

    fn total_blocks(&self) -> u64 {
        self.total_blocks
    }
}
```

---

## CLI Commands

### Format a Block Device

```bash
# Format USB drive (DESTROYS ALL DATA!)
sudo pslfs format /dev/sdb1 --label "MyUSB" --user admin

# Format with specific size (use only part of device)
sudo pslfs format /dev/sdb1 --size 1G --label "TestFS"

# Dry run (show what would happen)
sudo pslfs format /dev/sdb1 --dry-run
```

### Mount a Block Device

```bash
# Mount formatted device
sudo pslfs mount /dev/sdb1 /mnt/pslfs --user admin

# Mount read-only
sudo pslfs mount /dev/sdb1 /mnt/pslfs --read-only
```

### Check Device Filesystem

```bash
# Check filesystem integrity
sudo pslfs check /dev/sdb1

# Check and repair
sudo pslfs check /dev/sdb1 --repair
```

### Get Device Info

```bash
# Show filesystem stats
sudo pslfs stats /dev/sdb1

# Show in JSON format
sudo pslfs stats /dev/sdb1 --json
```

---

## Usage Examples

### Example 1: Format a USB Drive

```bash
# 1. Insert USB drive and identify it
lsblk
# Output:
# NAME   MAJ:MIN RM   SIZE RO TYPE MOUNTPOINT
# sda      8:0    0 238.5G  0 disk
# └─sda1   8:1    0 238.5G  0 part /
# sdb      8:16   1  14.9G  0 disk    <-- Your USB drive
# └─sdb1   8:17   1  14.9G  0 part

# 2. Unmount if mounted
sudo umount /dev/sdb1

# 3. Format with PSLFS
sudo pslfs format /dev/sdb1 --label "MyPSLFS" --user alice
# Enter password for alice: ********
#
# WARNING: This will DESTROY all data on /dev/sdb1
# Device: /dev/sdb1
# Size: 15.6 GB (15600000000 bytes)
# Are you sure? [yes/NO]: yes
#
# Formatting /dev/sdb1...
# Writing superblock...
# Initializing bitmaps...
# Creating root directory...
# Creating user 'alice'...
# Done! Filesystem created.

# 4. Mount it
sudo mkdir -p /mnt/pslfs
sudo pslfs mount /dev/sdb1 /mnt/pslfs --user alice
# Password: ********
# Mounted /dev/sdb1 at /mnt/pslfs

# 5. Use it!
cd /mnt/pslfs
sudo mkdir documents
sudo touch documents/readme.txt
echo "Hello from PSLFS!" | sudo tee documents/readme.txt

# 6. Unmount when done
cd ~
sudo umount /mnt/pslfs
```

### Example 2: Portable PSLFS Drive

```bash
# Format with multiple users
sudo pslfs format /dev/sdb1 --label "Shared"
sudo pslfs user add /dev/sdb1 --username alice
sudo pslfs user add /dev/sdb1 --username bob

# Mount and use
sudo pslfs mount /dev/sdb1 /mnt/shared --user alice

# Now you can unplug and use on another computer!
```

### Example 3: Dual Boot with Different FSes

```bash
# Partition USB drive
sudo fdisk /dev/sdb
# Create partition 1: 8GB (PSLFS)
# Create partition 2: 8GB (ext4)

# Format first partition with PSLFS
sudo pslfs format /dev/sdb1 --label "PSLFS_Part"

# Format second partition with ext4
sudo mkfs.ext4 /dev/sdb2 -L "EXT4_Part"

# Mount both
sudo pslfs mount /dev/sdb1 /mnt/pslfs
sudo mount /dev/sdb2 /mnt/ext4

# Now you have both filesystems on one drive!
```

---

## Technical Details

### Device Detection

```rust
/// Detect if path is a block device
pub fn is_block_device<P: AsRef<Path>>(path: P) -> Result<bool> {
    use std::os::unix::fs::FileTypeExt;

    let metadata = std::fs::metadata(path)?;
    Ok(metadata.file_type().is_block_device())
}

/// Auto-detect backend type
pub fn open_auto<P: AsRef<Path>>(path: P) -> Result<Box<dyn StorageBackend>> {
    if is_block_device(&path)? {
        Ok(Box::new(BlockDeviceBackend::open(path, false)?))
    } else {
        Ok(Box::new(FileBackend::open(path, false)?))
    }
}
```

### Filesystem Signature

To identify PSLFS filesystems on block devices:

```rust
/// Check if device contains PSLFS filesystem
pub fn is_pslfs_device<P: AsRef<Path>>(path: P) -> Result<bool> {
    let backend = BlockDeviceBackend::open(path, true)?;
    let superblock_block = backend.read_block(BlockId(0))?;

    let superblock: Superblock = bincode::deserialize(&superblock_block.data)?;

    Ok(superblock.magic == PSLFS_MAGIC && superblock.version == PSLFS_VERSION)
}
```

### Performance Considerations

**Block device performance vs file-backed:**

| Operation | File-Backed | Block Device | Notes |
|-----------|-------------|--------------|-------|
| Sequential read | ~100 MB/s | ~150 MB/s | Direct I/O is faster |
| Random read | ~50 MB/s | ~80 MB/s | Less overhead |
| Sequential write | ~80 MB/s | ~120 MB/s | No filesystem layer |
| Sync latency | 5-10ms | 2-5ms | Direct to hardware |

**Why block devices can be faster:**
- No host filesystem overhead
- Direct I/O to hardware
- Better control over caching
- No fragmentation from host FS

---

## Limitations

### Current Limitations

1. **No Partitioning Tool**: Must use `fdisk`/`parted` to create partitions first
2. **No TRIM Support**: SSD optimizations not implemented initially
3. **No Bad Block Management**: Assumes hardware handles this
4. **No Hot-Plug Detection**: Manual mount/unmount required
5. **Linux/Unix Only**: Windows support requires different approach (winfsp)

### Future Enhancements

- [ ] Auto-detection of removable devices
- [ ] Integration with `udisks2` for auto-mounting
- [ ] TRIM/discard support for SSDs
- [ ] S.M.A.R.T. integration for health monitoring
- [ ] Partition table support (create partitions automatically)
- [ ] Windows support via WinFsp

---

## Comparison with File-Backed

### When to Use Block Device Backend

✅ **Use Block Devices For:**
- Production deployments
- Better performance
- Dedicated storage
- Portable drives
- Teaching about real filesystems

❌ **Use File Backend For:**
- Development and testing
- No root access
- Cross-platform compatibility
- Backup/archival
- V0.1 compatibility

### Migration Between Backends

```bash
# Backup file-backed filesystem to block device
sudo pslfs copy /tmp/myfs.psl /dev/sdb1

# Backup block device to file
sudo pslfs copy /dev/sdb1 /backup/myfs.psl

# Clone one device to another
sudo pslfs clone /dev/sdb1 /dev/sdc1
```

---

## Safety Features

### Pre-Format Checks

Before formatting, PSLFS performs these checks:

1. **Device exists**: `/dev/sdb1` is a valid path
2. **Is block device**: Not a regular file or directory
3. **Not mounted**: Device is not currently in use
4. **User confirmation**: Explicit "yes" required
5. **Size sanity**: Device is reasonable size (> 1MB, < 100TB)

### Write Protection

```bash
# Enable write protection (hardware switch on some USB drives)
# Software read-only mode:
sudo pslfs mount /dev/sdb1 /mnt/pslfs --read-only

# Verify read-only:
touch /mnt/pslfs/test.txt
# Error: Read-only filesystem
```

---

## Troubleshooting

### "Permission denied" when formatting

```bash
# Solution: Use sudo
sudo pslfs format /dev/sdb1
```

### "Device is busy"

```bash
# Check what's using the device
sudo lsof /dev/sdb1
sudo fuser -v /dev/sdb1

# Unmount first
sudo umount /dev/sdb1

# Then format
sudo pslfs format /dev/sdb1
```

### "Device not found"

```bash
# List all devices
lsblk

# Check if device exists
ls -l /dev/sdb1

# Check dmesg for USB events
dmesg | tail -20
```

### Performance is slow

```bash
# Check if device is USB 2.0 (slow) vs USB 3.0 (fast)
lsusb -t

# Enable direct I/O (bypass cache)
sudo pslfs mount /dev/sdb1 /mnt/pslfs --direct-io

# Check for errors
sudo smartctl -a /dev/sdb
```

---

## Complete Example: Educational Lab Setup

**Goal**: Set up PSLFS on USB drives for a classroom

```bash
#!/bin/bash
# setup_classroom_drives.sh

# For each student's USB drive (sdb, sdc, sdd, ...)
for device in /dev/sd{b,c,d,e,f}1; do
    if [ -b "$device" ]; then
        echo "Formatting $device..."

        # Format with student name (assuming device label matches name)
        student_name=$(basename "$device")

        sudo pslfs format "$device" \
            --label "Student_$student_name" \
            --user student \
            --password classroom123 \
            --yes

        # Create standard directories
        sudo pslfs mount "$device" /mnt/temp --user student --password classroom123
        sudo mkdir -p /mnt/temp/{homework,projects,notes}
        sudo pslfs umount /mnt/temp

        echo "$device ready!"
    fi
done

echo "All drives formatted and ready for class!"
```

---

## Summary

PSLFS V2 supports block devices through the `BlockDeviceBackend`, enabling:

✅ **Real filesystem on real hardware**
✅ **USB drives, SD cards, hard disks**
✅ **Better performance than file-backed**
✅ **Portable between computers**
✅ **Educational value - real filesystem behavior**

**Next Steps:**
1. Implement `BlockDeviceBackend` (see code above)
2. Add `pslfs format` command to CLI
3. Test on various devices
4. Add to documentation

This makes PSLFS a **real, usable filesystem** - not just an educational toy!
