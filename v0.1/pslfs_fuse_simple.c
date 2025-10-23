/*
 * PSLFS FUSE Wrapper - Simplified Version
 *
 * This wraps your PSLFS filesystem to work as a mountable Linux filesystem
 *
 * Installation:
 *   sudo apt-get install libfuse3-dev fuse3  # Ubuntu/Debian
 *   sudo dnf install fuse3-devel fuse3       # Fedora
 *
 * Compile:
 *   gcc -Wall -o pslfs_fuse pslfs_fuse_simple.c `pkg-config fuse3 --cflags --libs`
 *
 * Usage:
 *   mkdir -p /tmp/pslfs_mount
 *   ./pslfs_fuse /tmp/pslfs_mount
 *
 *   # In another terminal:
 *   ls /tmp/pslfs_mount
 *   echo "Hello" > /tmp/pslfs_mount/test.txt
 *   cat /tmp/pslfs_mount/test.txt
 *
 *   # Unmount:
 *   fusermount3 -u /tmp/pslfs_mount
 */

#define FUSE_USE_VERSION 31

#include <fuse.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <fcntl.h>
#include <stddef.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include <time.h>

// Simplified PSLFS structures (without full header dependencies)
typedef struct {
    char name[32];
    long sector;
    long insector;
    long upsector;
    long next;
    long prev;
    long filesector;
    char properties[3];
    int size;
} psl_folder;

typedef struct {
    char name[32];
    long next;
    long prev;
    long sector;
    long fsector;
    char properties[4];
    int size;
} psl_file;

// Global filesystem state
static char *fs_path = "test1.psl";
static psl_folder root_folder;

// Helper: Read folder from PSLFS
static int read_psl_folder(long sector, psl_folder *folder) {
    FILE *f = fopen(fs_path, "rb");
    if (!f) return -1;

    if (fseek(f, sector, SEEK_SET) != 0) {
        fclose(f);
        return -1;
    }

    size_t read = fread(folder, sizeof(psl_folder), 1, f);
    fclose(f);

    return (read == 1) ? 0 : -1;
}

// Helper: Read file from PSLFS
static int read_psl_file(long sector, psl_file *file) {
    FILE *f = fopen(fs_path, "rb");
    if (!f) return -1;

    if (fseek(f, sector, SEEK_SET) != 0) {
        fclose(f);
        return -1;
    }

    size_t read = fread(file, sizeof(psl_file), 1, f);
    fclose(f);

    return (read == 1) ? 0 : -1;
}

// FUSE: Get file attributes
static int pslfs_getattr(const char *path, struct stat *stbuf,
                         struct fuse_file_info *fi) {
    (void) fi;
    memset(stbuf, 0, sizeof(struct stat));

    printf("[FUSE] getattr: %s\n", path);

    // Root directory
    if (strcmp(path, "/") == 0) {
        stbuf->st_mode = S_IFDIR | 0755;
        stbuf->st_nlink = 2;
        stbuf->st_uid = getuid();
        stbuf->st_gid = getgid();
        stbuf->st_atime = stbuf->st_mtime = stbuf->st_ctime = time(NULL);
        return 0;
    }

    // For now, return file not found for everything else
    // TODO: Traverse PSLFS to find actual files
    return -ENOENT;
}

// FUSE: Read directory contents
static int pslfs_readdir(const char *path, void *buf, fuse_fill_dir_t filler,
                         off_t offset, struct fuse_file_info *fi,
                         enum fuse_readdir_flags flags) {
    (void) offset;
    (void) fi;
    (void) flags;

    printf("[FUSE] readdir: %s\n", path);

    if (strcmp(path, "/") != 0)
        return -ENOENT;

    // Standard entries
    filler(buf, ".", NULL, 0, 0);
    filler(buf, "..", NULL, 0, 0);

    // Read root folder from PSLFS
    psl_folder folder;
    if (read_psl_folder(1, &folder) == 0) {
        printf("[PSLFS] Root folder: %s (insector=%ld, filesector=%ld)\n",
               folder.name, folder.insector, folder.filesector);

        // TODO: Traverse child folders and files
        // For now, add a demo entry
        filler(buf, "readme.txt", NULL, 0, 0);
    }

    return 0;
}

// FUSE: Open file
static int pslfs_open(const char *path, struct fuse_file_info *fi) {
    printf("[FUSE] open: %s\n", path);

    // TODO: Check if file exists in PSLFS
    return 0;
}

// FUSE: Read file
static int pslfs_read(const char *path, char *buf, size_t size, off_t offset,
                      struct fuse_file_info *fi) {
    (void) fi;

    printf("[FUSE] read: %s (size=%zu, offset=%ld)\n", path, size, offset);

    // Demo: Return a message
    const char *demo = "This is PSLFS mounted via FUSE!\n";
    size_t len = strlen(demo);

    if (offset < len) {
        if (offset + size > len)
            size = len - offset;
        memcpy(buf, demo + offset, size);
        return size;
    }

    return 0;
}

// FUSE operations structure
static struct fuse_operations pslfs_oper = {
    .getattr    = pslfs_getattr,
    .readdir    = pslfs_readdir,
    .open       = pslfs_open,
    .read       = pslfs_read,
};

int main(int argc, char *argv[]) {
    // Check if PSLFS files exist
    if (access(fs_path, F_OK) != 0) {
        fprintf(stderr, "Error: %s not found!\n", fs_path);
        fprintf(stderr, "Please run './base' first to create the filesystem.\n");
        return 1;
    }

    printf("╔════════════════════════════════════════╗\n");
    printf("║   PSLFS FUSE Filesystem Wrapper       ║\n");
    printf("╚════════════════════════════════════════╝\n\n");
    printf("Filesystem: %s\n", fs_path);
    printf("Mounting at: %s\n\n", (argc > 1) ? argv[1] : "(not specified)");
    printf("Usage:\n");
    printf("  Mount:   %s <mountpoint>\n", argv[0]);
    printf("  Unmount: fusermount3 -u <mountpoint>\n\n");

    if (argc < 2) {
        fprintf(stderr, "Usage: %s <mountpoint>\n", argv[0]);
        return 1;
    }

    // Read root folder
    if (read_psl_folder(1, &root_folder) != 0) {
        fprintf(stderr, "Error: Could not read PSLFS root folder\n");
        return 1;
    }

    printf("Root folder: %s\n", root_folder.name);
    printf("Starting FUSE...\n\n");

    return fuse_main(argc, argv, &pslfs_oper, NULL);
}
