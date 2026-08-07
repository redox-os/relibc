#include <errno.h>
#include <fcntl.h>
#include <redox/ring.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

// Copies the next CQE into `cqe`. Blocks the current process if the CQ is
// empty.
int redox_ring_wait_cqe(struct redox_ring* ring, void* cqe);

// Marks the next unacknowledged CQE as processed.
void redox_ring_cqe_seen(struct redox_ring* ring);

// Submit pending SQEs.
//
// On success, the number of submitted SQEs is returned.
int redox_ring_submit(struct redox_ring* ring);

void* redox_ring_get_sqe(struct redox_ring* ring);

enum redox_ring_disk_op {
    REDOX_RING_OP_DISK_READ,
    REDOX_RING_OP_DISK_WRITE,
};

struct disk_op_sqe {
    uint8_t opcode;
    uint8_t pad[3];
    uint32_t file_idx;
    uint64_t block;
    uint32_t buf_offset;
    uint32_t buf_len;
    uint64_t user_data;
};

struct disk_op_cqe {
    uint64_t user_data;
    uint32_t count;
    uint16_t status; // 0 = Success
    uint16_t pad;
};

static void redox_ring_disk_prep_rw(struct disk_op_sqe* sqe, uint8_t op, uint32_t file_idx, uint32_t buf_offset, uint32_t buf_len, uint64_t block)
{
    sqe->opcode = op;
    sqe->file_idx = file_idx;
    sqe->buf_offset = buf_offset;
    sqe->buf_len = buf_len;
    sqe->block = block;
    memset(sqe->pad, 0, sizeof(sqe->pad));
}

static void redox_ring_disk_sqe_set_data64(struct disk_op_sqe* sqe, uint64_t user_data)
{
    sqe->user_data = user_data;
}

static inline void redox_ring_disk_prep_readv(struct disk_op_sqe* sqe, uint64_t file_idx, uint32_t buf_offset, uint32_t buf_len, uint64_t offset_bytes)
{
    redox_ring_disk_prep_rw(sqe, REDOX_RING_OP_DISK_READ, file_idx, buf_offset, buf_len, offset_bytes / 512);
}

static inline void redox_ring_disk_prep_writev(struct disk_op_sqe* sqe, uint64_t file_idx, uint32_t buf_offset, uint32_t buf_len, uint64_t offset_bytes)
{
    redox_ring_disk_prep_rw(sqe, REDOX_RING_OP_DISK_WRITE, file_idx, buf_offset, buf_len, offset_bytes / 512);
}

int main()
{
    const char* disk_scheme_root = "/scheme/disk.pci-0000-00-06.0-nvme";
    const char* disk_path = "/scheme/disk.pci-0000-00-06.0-nvme/1";
    int disk_fd;

    if ((disk_fd = open(disk_path, O_RDWR)) == -1) {
        perror("open");
        return EXIT_FAILURE;
    }

    uint32_t entries = 128;
    uint32_t sqe_size = sizeof(struct disk_op_sqe);
    uint32_t cqe_size = sizeof(struct disk_op_cqe);
    uint32_t pool_size = 4 * 1024 * 1024; // 16 MB pool
    uint32_t chunk_size = 256 * 1024; // 256 kB chunks

    fflush(stdout);
    void* ring = malloc(redox_ring_sizeof());
    if (!ring) {
        printf("Failed to allocate memory for ring structure.\n");
        return 1;
    }

    printf("Initializing ring (Dynamic alloc) on path '%s'...\n", disk_path);
    fflush(stdout);

    int ret = redox_ring_queue_init_with_path(
        entries, sqe_size, cqe_size, pool_size, chunk_size, 1, ring, 0, disk_scheme_root);

    if (ret != 0) {
        printf("Failed to init ring on '%s'. Error: %s (code %d)\n", disk_path, strerror(-ret), ret);
        free(ring);
        return 1;
    }

    int fixed_ftbl[1] = { disk_fd };
    if ((ret = redox_ring_register_files(ring, fixed_ftbl, 1)) < 0) {
        printf("redox_ring_register_files: %s", strerror(-ret));
        return EXIT_FAILURE;
    }

    uint8_t* temp_buf = NULL;
    int temp_off = redox_ring_alloc_sized(ring, chunk_size, 1, &temp_buf);
    if (temp_off < 0) {
        printf("Failed to allocate initial buffer for base calculation.\n");
        return 1;
    }
    uint8_t* shm_base = temp_buf - temp_off;
    redox_ring_free_buf_sized(ring, temp_buf);

    printf("\nStarting Write Benchmark (Dynamic alloc)...\n");
    fflush(stdout);

    uint8_t* buf = NULL;
    int buf_offset;
    int total_ops = 32000;
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

    uint32_t file_idx = 0;
    int in_flight = 0;
    int unsubmitted = 0;

    for (int i = 0; i < total_ops; i++) {
        buf_offset = redox_ring_alloc_sized(ring, chunk_size, 1, &buf);
        while (buf_offset < 0) {
            if (unsubmitted > 0) {
                redox_ring_submit(ring);
                unsubmitted = 0;
            }
            struct disk_op_cqe cqe;
            redox_ring_wait_cqe(ring, &cqe);
            redox_ring_cqe_seen(ring);

            if (cqe.status != 0 || cqe.count != 4096) {
                printf("Error: Write failed at reaping. Status: %u, Count: %u (Expected 4096)\n", cqe.status, cqe.count);
                return 1;
            }
            redox_ring_free_buf(ring, (uint32_t)cqe.user_data);
            in_flight--;
            buf_offset = redox_ring_alloc_sized(ring, chunk_size, 1, &buf);
        }

        struct disk_op_sqe* sqe = (struct disk_op_sqe*)redox_ring_get_sqe(ring);
        while (sqe == NULL) {
            if (unsubmitted > 0) {
                redox_ring_submit(ring);
                unsubmitted = 0;
            }
            struct disk_op_cqe cqe;
            redox_ring_wait_cqe(ring, &cqe);
            redox_ring_cqe_seen(ring);

            if (cqe.status != 0 || cqe.count != 4096) {
                printf("Error: Write failed at SQE reaping. Status: %u, Count: %u (Expected 4096)\n", cqe.status, cqe.count);
                return 1;
            }
            redox_ring_free_buf(ring, (uint32_t)cqe.user_data);
            in_flight--;
            sqe = (struct disk_op_sqe*)redox_ring_get_sqe(ring);
        }

        memset(buf, 'A' + (i % 26), 4096);
        uint64_t req_id = ((uint64_t)i << 32) | (uint32_t)buf_offset;
        redox_ring_disk_prep_writev(sqe, file_idx, buf_offset, 4096, i * 4096);
        redox_ring_disk_sqe_set_data64(sqe, req_id);

        unsubmitted++;
        in_flight++;

        if (unsubmitted >= 32) {
            redox_ring_submit(ring);
            unsubmitted = 0;
        }
    }

    if (unsubmitted > 0) {
        redox_ring_submit(ring);
    }
    while (in_flight > 0) {
        struct disk_op_cqe cqe;
        redox_ring_wait_cqe(ring, &cqe);
        redox_ring_cqe_seen(ring);

        if (cqe.status != 0 || cqe.count != 4096) {
            printf("Error: Write failed at final drain. Status: %u, Count: %u (Expected 4096)\n", cqe.status, cqe.count);
            return 1;
        }
        redox_ring_free_buf(ring, (uint32_t)cqe.user_data);
        in_flight--;
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    double duration = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    double bytes_processed = (double)total_ops * 4096.0;
    double mbps = (bytes_processed / 1024.0 / 1024.0) / duration;

    printf("Write Benchmark: %d ops in %.3f seconds (%.2f MB/s)\n", total_ops, duration, mbps);

    printf("\nStarting Read Benchmark (Dynamic alloc with async batching & verify)...\n");
    fflush(stdout);
    clock_gettime(CLOCK_MONOTONIC, &start);

    in_flight = 0;
    unsubmitted = 0;

    for (int i = 0; i < total_ops; i++) {
        buf_offset = redox_ring_alloc_sized(ring, chunk_size, 1, &buf);
        while (buf_offset < 0) {
            if (unsubmitted > 0) {
                redox_ring_submit(ring);
                unsubmitted = 0;
            }
            struct disk_op_cqe cqe;
            redox_ring_wait_cqe(ring, &cqe);
            redox_ring_cqe_seen(ring);

            uint32_t completed_i = (uint32_t)(cqe.user_data >> 32);

            if (cqe.status != 0 || cqe.count != 4096) {
                printf("Error: Read failed for block %u. Status: %u, Count: %u (Expected 4096)\n", completed_i, cqe.status, cqe.count);
                return 1;
            }

            uint32_t completed_buf_offset = (uint32_t)cqe.user_data;
            uint8_t* completed_buf = shm_base + completed_buf_offset;

            uint8_t expected_char = 'A' + (completed_i % 26);
            for (int j = 0; j < 4096; j++) {
                if (completed_buf[j] != expected_char) {
                    printf("Error: Data mismatch at block %u, byte %d. Expected '%c' (0x%02X), got 0x%02X\n",
                        completed_i, j, expected_char, expected_char, completed_buf[j]);
                    return 1;
                }
            }

            redox_ring_free_buf_sized(ring, completed_buf);
            in_flight--;
            buf_offset = redox_ring_alloc_sized(ring, chunk_size, 1, &buf);
        }

        struct disk_op_sqe* sqe = (struct disk_op_sqe*)redox_ring_get_sqe(ring);
        while (sqe == NULL) {
            if (unsubmitted > 0) {
                redox_ring_submit(ring);
                unsubmitted = 0;
            }
            struct disk_op_cqe cqe;
            redox_ring_wait_cqe(ring, &cqe);
            redox_ring_cqe_seen(ring);

            uint32_t completed_i = (uint32_t)(cqe.user_data >> 32);

            if (cqe.status != 0 || cqe.count != 4096) {
                printf("Error: Read failed for block %u at SQE wait. Status: %u, Count: %u\n", completed_i, cqe.status, cqe.count);
                return 1;
            }

            uint32_t completed_buf_offset = (uint32_t)cqe.user_data;
            uint8_t* completed_buf = shm_base + completed_buf_offset;

            uint8_t expected_char = 'A' + (completed_i % 26);
            for (int j = 0; j < 4096; j++) {
                if (completed_buf[j] != expected_char) {
                    printf("Error: Data mismatch at block %u, byte %d. Expected '%c' (0x%02X), got 0x%02X\n",
                        completed_i, j, expected_char, expected_char, completed_buf[j]);
                    return 1;
                }
            }

            redox_ring_free_buf_sized(ring, completed_buf);
            in_flight--;
            sqe = (struct disk_op_sqe*)redox_ring_get_sqe(ring);
        }

        memset(buf, 0, 4096);
        uint64_t req_id = ((uint64_t)i << 32) | (uint32_t)buf_offset;
        redox_ring_disk_prep_readv(sqe, file_idx, buf_offset, 4096, i * 4096);
        redox_ring_disk_sqe_set_data64(sqe, req_id);

        unsubmitted++;
        in_flight++;

        if (unsubmitted >= 32) {
            redox_ring_submit(ring);
            unsubmitted = 0;
        }
    }

    if (unsubmitted > 0) {
        redox_ring_submit(ring);
    }
    while (in_flight > 0) {
        struct disk_op_cqe cqe;
        redox_ring_wait_cqe(ring, &cqe);
        redox_ring_cqe_seen(ring);

        uint32_t completed_i = (uint32_t)(cqe.user_data >> 32);

        if (cqe.status != 0 || cqe.count != 4096) {
            printf("Error: Read failed for block %u at final drain. Status: %u, Count: %u\n", completed_i, cqe.status, cqe.count);
            return 1;
        }

        uint32_t completed_buf_offset = (uint32_t)cqe.user_data;
        uint8_t* completed_buf = shm_base + completed_buf_offset;

        uint8_t expected_char = 'A' + (completed_i % 26);
        for (int j = 0; j < 4096; j++) {
            if (completed_buf[j] != expected_char) {
                printf("Error: Data mismatch at block %u, byte %d. Expected '%c' (0x%02X), got 0x%02X\n",
                    completed_i, j, expected_char, expected_char, completed_buf[j]);
                return 1;
            }
        }

        redox_ring_free_buf_sized(ring, completed_buf);
        in_flight--;
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    duration = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    mbps = (bytes_processed / 1024.0 / 1024.0) / duration;

    printf("Read Benchmark: %d ops in %.3f seconds (%.2f MB/s)\n", total_ops, duration, mbps);
    fflush(stdout);

    redox_ring_queue_exit(ring);
    free(ring);

    printf("\nTests finished successfully!\n");
    return 0;
}
