/*
 * PSLFS FUSE Wrapper
 * Mounts PSLFS as a real Linux filesystem using FUSE
 *
 * Compile: gcc -Wall -o pslfs_fuse pslfs_fuse.c `pkg-config fuse3 --cflags --libs`
 * Mount:   ./pslfs_fuse /mnt/pslfs
 * Unmount: fusermount -u /mnt/pslfs
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
#include <sys/stat.h>

// Include PSLFS core structures
#include "ds.h"
#include "partition.h"
#include "fstest1.h"

// FUSE filesystem context
struct pslfs_context {
    char *base_path;
    ffolder root;
};

static struct pslfs_context *get_context() {
    return (struct pslfs_context *)fuse_get_context()->private_data;
}

// Convert PSLFS path to absolute path
static void get_full_path(char *fpath, const char *path) {
    struct pslfs_context *ctx = get_context();
    strcpy(fpath, ctx->base_path);
    strcat(fpath, "/test1.psl");
}

// FUSE operation: getattr (like stat)
static int pslfs_getattr(const char *path, struct stat *stbuf,
                         struct fuse_file_info *fi) {
    (void) fi;
    char fpath[256];
    ffolder current;

    memset(stbuf, 0, sizeof(struct stat));

    // Root directory
    if (strcmp(path, "/") == 0) {
        stbuf->st_mode = S_IFDIR | 0755;
        stbuf->st_nlink = 2;
        return 0;
    }

    // TODO: Traverse PSLFS structure to find file/folder
    // For now, return -ENOENT for everything else
    return -ENOENT;
}

// FUSE operation: readdir (list directory)
static int pslfs_readdir(const char *path, void *buf, fuse_fill_dir_t filler,
                         off_t offset, struct fuse_file_info *fi,
                         enum fuse_readdir_flags flags) {
    (void) offset;
    (void) fi;
    (void) flags;

    if (strcmp(path, "/") != 0)
        return -ENOENT;

    // Add standard entries
    filler(buf, ".", NULL, 0, 0);
    filler(buf, "..", NULL, 0, 0);

    // TODO: Read PSLFS directory and add entries
    // For now, just show it's working
    filler(buf, "example.txt", NULL, 0, 0);

    return 0;
}

// FUSE operation: open
static int pslfs_open(const char *path, struct fuse_file_info *fi) {
    // TODO: Check if file exists in PSLFS
    return 0;
}

// FUSE operation: read
static int pslfs_read(const char *path, char *buf, size_t size, off_t offset,
                      struct fuse_file_info *fi) {
    (void) fi;

    // TODO: Read from PSLFS file
    // For now, return empty
    return 0;
}

// FUSE operation: write
static int pslfs_write(const char *path, const char *buf, size_t size,
                       off_t offset, struct fuse_file_info *fi) {
    (void) fi;

    // TODO: Write to PSLFS file
    return size;
}

// FUSE operation: create
static int pslfs_create(const char *path, mode_t mode,
                        struct fuse_file_info *fi) {
    (void) fi;
    (void) mode;

    // TODO: Create file in PSLFS
    return 0;
}

// FUSE operation: mkdir
static int pslfs_mkdir(const char *path, mode_t mode) {
    (void) mode;

    // TODO: Create directory in PSLFS
    return 0;
}

// FUSE operation: unlink (delete file)
static int pslfs_unlink(const char *path) {
    // TODO: Delete file from PSLFS
    return 0;
}

// FUSE operation: rmdir (delete directory)
static int pslfs_rmdir(const char *path) {
    // TODO: Delete directory from PSLFS
    return 0;
}

// FUSE operations structure
static struct fuse_operations pslfs_oper = {
    .getattr    = pslfs_getattr,
    .readdir    = pslfs_readdir,
    .open       = pslfs_open,
    .read       = pslfs_read,
    .write      = pslfs_write,
    .create     = pslfs_create,
    .mkdir      = pslfs_mkdir,
    .unlink     = pslfs_unlink,
    .rmdir      = pslfs_rmdir,
};

int main(int argc, char *argv[]) {
    struct pslfs_context ctx;

    // Get current directory as base path
    ctx.base_path = getcwd(NULL, 0);
    if (!ctx.base_path) {
        perror("getcwd");
        return 1;
    }

    printf("PSLFS FUSE Filesystem\n");
    printf("Base path: %s\n", ctx.base_path);
    printf("Mounting...\n\n");

    // Initialize PSLFS root
    // TODO: Read root from test1.psl

    // Run FUSE
    int ret = fuse_main(argc, argv, &pslfs_oper, &ctx);

    free(ctx.base_path);
    return ret;
}
