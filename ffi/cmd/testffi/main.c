// testffi — standalone C test runner for the kubo FFI layer.
// Build the archive first:
//   go build -buildmode=c-archive -o tmp/libkubo_ffi.a ffi.go
// Then compile this file:
//   cc -o testffi main.c -I tmp tmp/libkubo_ffi.a -lpthread -ldl
// On macOS add: -framework Security -framework CoreFoundation -lresolv

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdarg.h>
#include "libkubo_ffi.h"

static int failures = 0;

static void failf(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    printf("FAIL: ");
    vprintf(fmt, args);
    printf("\n");
    va_end(args);
    failures++;
}

static void okf(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    printf("OK: ");
    vprintf(fmt, args);
    printf("\n");
    va_end(args);
}

static void rmrf(const char *path) {
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "rm -rf \"%s\"", path);
    system(cmd);
}

static void test_version(void) {
    char *v = kubo_version();
    if (!v) {
        failf("kubo_version returned nil");
        return;
    }
    if (strlen(v) == 0) {
        kubo_free_string(v);
        failf("version is empty");
        return;
    }
    okf("version = %s", v);
    kubo_free_string(v);
}

static void test_init_repo_and_node_lifecycle(void) {
    const char *tmp = "./tmp/kubo-ffi-test-lifecycle";
    rmrf(tmp);

    if (kubo_init_repo((char *)tmp) != 0) {
        char *err = kubo_last_error();
        failf("init repo: %s", err ? err : "unknown");
        if (err) kubo_free_string(err);
        return;
    }

    uint64_t handle = kubo_node_start((char *)tmp, 0);
    if (handle == 0) {
        char *err = kubo_last_error();
        failf("node start: %s", err ? err : "unknown");
        if (err) kubo_free_string(err);
        return;
    }

    char *peer_id = kubo_node_peer_id(handle);
    if (!peer_id) {
        failf("peer_id returned nil");
        kubo_node_stop(handle);
        return;
    }
    if (strlen(peer_id) == 0) {
        kubo_free_string(peer_id);
        failf("peer_id is empty");
        kubo_node_stop(handle);
        return;
    }
    okf("peer_id = %s", peer_id);
    kubo_free_string(peer_id);

    if (kubo_node_stop(handle) != 0) {
        failf("node stop failed");
        return;
    }
    okf("node lifecycle");
}

static void test_unixfs_add_and_cat(void) {
    const char *tmp = "./tmp/kubo-ffi-test-unixfs";
    rmrf(tmp);

    if (kubo_init_repo((char *)tmp) != 0) {
        failf("init repo failed");
        return;
    }

    uint64_t handle = kubo_node_start((char *)tmp, 0);
    if (handle == 0) {
        failf("node start failed");
        return;
    }

    const char *data = "hello from ffi test";
    size_t len = strlen(data);
    char *cid = kubo_unixfs_add_bytes(handle, (uint8_t *)data, len);
    if (!cid) {
        failf("add_bytes returned nil");
        kubo_node_stop(handle);
        return;
    }

    if (strlen(cid) == 0) {
        kubo_free_string(cid);
        failf("cid is empty");
        kubo_node_stop(handle);
        return;
    }

    uint8_t *out = NULL;
    size_t out_len = 0;
    if (kubo_unixfs_cat(handle, cid, &out, &out_len) != 0) {
        kubo_free_string(cid);
        failf("cat failed");
        kubo_node_stop(handle);
        return;
    }

    if (out_len != len || memcmp(out, data, len) != 0) {
        failf("cat: expected %s, got %.*s", data, (int)out_len, (char *)out);
    } else {
        okf("unixfs add/cat roundtrip");
    }

    if (out) kubo_free_buffer(out);
    kubo_free_string(cid);
    kubo_node_stop(handle);
}

static void test_block_put_get_stat(void) {
    const char *tmp = "./tmp/kubo-ffi-test-block";
    rmrf(tmp);

    if (kubo_init_repo((char *)tmp) != 0) {
        failf("init repo failed");
        return;
    }

    uint64_t handle = kubo_node_start((char *)tmp, 0);
    if (handle == 0) {
        failf("node start failed");
        return;
    }

    const char *data = "raw block data";
    size_t len = strlen(data);
    char *cid = kubo_block_put(handle, (uint8_t *)data, len);
    if (!cid) {
        failf("block_put returned nil");
        kubo_node_stop(handle);
        return;
    }

    if (strlen(cid) == 0) {
        kubo_free_string(cid);
        failf("cid is empty");
        kubo_node_stop(handle);
        return;
    }

    int64_t size = kubo_block_stat(handle, cid);
    if (size != (int64_t)len) {
        failf("block_stat: expected %zu, got %lld", len, (long long)size);
    }

    uint8_t *out = NULL;
    size_t out_len = 0;
    if (kubo_block_get(handle, cid, &out, &out_len) != 0) {
        kubo_free_string(cid);
        failf("block_get failed");
        kubo_node_stop(handle);
        return;
    }

    if (out_len != len || memcmp(out, data, len) != 0) {
        failf("block_get: expected %s, got %.*s", data, (int)out_len, (char *)out);
    } else {
        okf("block put/get/stat roundtrip");
    }

    if (out) kubo_free_buffer(out);
    kubo_free_string(cid);
    kubo_node_stop(handle);
}

static void test_listening_addrs(void) {
    const char *tmp = "./tmp/kubo-ffi-test-addrs";
    rmrf(tmp);

    if (kubo_init_repo((char *)tmp) != 0) {
        failf("init repo failed");
        return;
    }

    uint64_t handle = kubo_node_start((char *)tmp, 1);
    if (handle == 0) {
        failf("node start failed");
        return;
    }

    char *addrs = kubo_node_listening_addrs(handle);
    if (!addrs) {
        failf("listening_addrs returned nil");
        kubo_node_stop(handle);
        return;
    }
    if (strlen(addrs) == 0) {
        kubo_free_string(addrs);
        failf("listening_addrs is empty");
        kubo_node_stop(handle);
        return;
    }
    okf("listening_addrs = %s", addrs);
    kubo_free_string(addrs);
    kubo_node_stop(handle);
}

int main(void) {
    printf("=== FFI Test Runner ===\n");

    test_version();
    test_init_repo_and_node_lifecycle();
    test_unixfs_add_and_cat();
    test_block_put_get_stat();
    test_listening_addrs();

    printf("\n");
    if (failures > 0) {
        printf("=== %d FAILURE(S) ===\n", failures);
        return 1;
    }
    printf("=== ALL TESTS PASSED ===\n");
    return 0;
}
