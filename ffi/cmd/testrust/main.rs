// testrust — standalone Rust test runner for the kubo FFI layer.
// Build the archive first:
//   cd kubo-sys/ffi && go build -buildmode=c-archive -o tmp/libkubo_ffi.a ffi.go
// Then compile and run:
//   cd kubo-sys/ffi/cmd/testrust
//   rustc main.rs -L ../../tmp -lkubo_ffi -o testrust
// On macOS add: -framework Security -framework CoreFoundation -lresolv -lpthread -ldl
// On Linux add: -lpthread -ldl

use std::ffi::{c_char, CStr, CString};
use std::slice;

#[link(name = "kubo_ffi", kind = "static")]
extern "C" {
    fn kubo_version() -> *mut c_char;
    fn kubo_last_error() -> *mut c_char;
    fn kubo_free_string(s: *mut c_char);
    fn kubo_init_repo(repoPath: *const c_char) -> i64;
    fn kubo_node_start(repoPath: *const c_char, online: u8) -> u64;
    fn kubo_node_stop(handle: u64) -> i64;
    fn kubo_node_peer_id(handle: u64) -> *mut c_char;
    fn kubo_node_listening_addrs(handle: u64) -> *mut c_char;
    fn kubo_unixfs_add_bytes(handle: u64, data: *const u8, length: usize) -> *mut c_char;
    fn kubo_unixfs_cat(
        handle: u64,
        cid_str: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;
    fn kubo_block_put(handle: u64, data: *const u8, length: usize) -> *mut c_char;
    fn kubo_block_get(
        handle: u64,
        cid_str: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;
    fn kubo_block_stat(handle: u64, cid_str: *const c_char) -> i64;
    fn kubo_free_buffer(buf: *mut u8);
}

use std::sync::atomic::{AtomicUsize, Ordering};

static FAILURES: AtomicUsize = AtomicUsize::new(0);

fn fail(msg: &str) {
    eprintln!("FAIL: {}", msg);
    FAILURES.fetch_add(1, Ordering::SeqCst);
}

fn ok(msg: &str) {
    println!("OK: {}", msg);
}

fn rmrf(path: &str) {
    let _ = std::fs::remove_dir_all(path);
}

fn test_version() {
    unsafe {
        let v = kubo_version();
        if v.is_null() {
            fail("kubo_version returned null");
            return;
        }
        let s = CStr::from_ptr(v).to_string_lossy();
        if s.is_empty() {
            kubo_free_string(v);
            fail("version is empty");
            return;
        }
        ok(&format!("version = {}", s));
        kubo_free_string(v);
    }
}

fn test_init_repo_and_node_lifecycle() {
    let tmp = "./tmp/kubo-rust-test-lifecycle";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            let err = kubo_last_error();
            let msg = if err.is_null() {
                "unknown".to_string()
            } else {
                let s = CStr::from_ptr(err).to_string_lossy().to_string();
                kubo_free_string(err);
                s
            };
            fail(&format!("init repo: {}", msg));
            return;
        }

        let handle = kubo_node_start(path.as_ptr(), 0);
        if handle == 0 {
            let err = kubo_last_error();
            let msg = if err.is_null() {
                "unknown".to_string()
            } else {
                let s = CStr::from_ptr(err).to_string_lossy().to_string();
                kubo_free_string(err);
                s
            };
            fail(&format!("node start: {}", msg));
            return;
        }

        let peer_id = kubo_node_peer_id(handle);
        if peer_id.is_null() {
            fail("peer_id returned null");
            kubo_node_stop(handle);
            return;
        }
        let id_str = CStr::from_ptr(peer_id).to_string_lossy();
        if id_str.is_empty() {
            kubo_free_string(peer_id);
            fail("peer_id is empty");
            kubo_node_stop(handle);
            return;
        }
        ok(&format!("peer_id = {}", id_str));
        kubo_free_string(peer_id);

        if kubo_node_stop(handle) != 0 {
            fail("node stop failed");
            return;
        }
        ok("node lifecycle");
    }
}

fn test_unixfs_add_and_cat() {
    let tmp = "./tmp/kubo-rust-test-unixfs";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            fail("init repo failed");
            return;
        }

        let handle = kubo_node_start(path.as_ptr(), 0);
        if handle == 0 {
            fail("node start failed");
            return;
        }

        let data = b"hello from ffi test";
        let cid = kubo_unixfs_add_bytes(handle, data.as_ptr(), data.len());
        if cid.is_null() {
            fail("add_bytes returned null");
            kubo_node_stop(handle);
            return;
        }
        let cid_str = CStr::from_ptr(cid).to_string_lossy().to_string();
        if cid_str.is_empty() {
            kubo_free_string(cid);
            fail("cid is empty");
            kubo_node_stop(handle);
            return;
        }

        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        if kubo_unixfs_cat(handle, cid, &mut out, &mut out_len) != 0 {
            kubo_free_string(cid);
            fail("cat failed");
            kubo_node_stop(handle);
            return;
        }

        let got = slice::from_raw_parts(out, out_len);
        if got != data {
            fail(&format!(
                "cat: expected {:?}, got {:?}",
                String::from_utf8_lossy(data),
                String::from_utf8_lossy(got)
            ));
        } else {
            ok("unixfs add/cat roundtrip");
        }

        if !out.is_null() {
            kubo_free_buffer(out);
        }
        kubo_free_string(cid);
        kubo_node_stop(handle);
    }
}

fn test_block_put_get_stat() {
    let tmp = "./tmp/kubo-rust-test-block";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            fail("init repo failed");
            return;
        }

        let handle = kubo_node_start(path.as_ptr(), 0);
        if handle == 0 {
            fail("node start failed");
            return;
        }

        let data = b"raw block data";
        let cid = kubo_block_put(handle, data.as_ptr(), data.len());
        if cid.is_null() {
            fail("block_put returned null");
            kubo_node_stop(handle);
            return;
        }
        let cid_str = CStr::from_ptr(cid).to_string_lossy().to_string();
        if cid_str.is_empty() {
            kubo_free_string(cid);
            fail("cid is empty");
            kubo_node_stop(handle);
            return;
        }

        let size = kubo_block_stat(handle, cid);
        if size != data.len() as i64 {
            fail(&format!("block_stat: expected {}, got {}", data.len(), size));
        }

        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        if kubo_block_get(handle, cid, &mut out, &mut out_len) != 0 {
            kubo_free_string(cid);
            fail("block_get failed");
            kubo_node_stop(handle);
            return;
        }

        let got = slice::from_raw_parts(out, out_len);
        if got != data {
            fail(&format!(
                "block_get: expected {:?}, got {:?}",
                String::from_utf8_lossy(data),
                String::from_utf8_lossy(got)
            ));
        } else {
            ok("block put/get/stat roundtrip");
        }

        if !out.is_null() {
            kubo_free_buffer(out);
        }
        kubo_free_string(cid);
        kubo_node_stop(handle);
    }
}

fn test_listening_addrs() {
    let tmp = "./tmp/kubo-rust-test-addrs";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            fail("init repo failed");
            return;
        }

        let handle = kubo_node_start(path.as_ptr(), 1);
        if handle == 0 {
            fail("node start failed");
            return;
        }

        let addrs = kubo_node_listening_addrs(handle);
        if addrs.is_null() {
            fail("listening_addrs returned null");
            kubo_node_stop(handle);
            return;
        }
        let s = CStr::from_ptr(addrs).to_string_lossy();
        if s.is_empty() {
            kubo_free_string(addrs);
            fail("listening_addrs is empty");
            kubo_node_stop(handle);
            return;
        }
        ok(&format!("listening_addrs = {}", s));
        kubo_free_string(addrs);
        kubo_node_stop(handle);
    }
}

fn main() {
    println!("=== Rust FFI Test Runner ===");

    test_version();
    test_init_repo_and_node_lifecycle();
    test_unixfs_add_and_cat();
    test_block_put_get_stat();
    test_listening_addrs();

    println!();
    let failures = FAILURES.load(Ordering::SeqCst);
    if failures > 0 {
        println!("=== {} FAILURE(S) ===", failures);
        std::process::exit(1);
    }
    println!("=== ALL TESTS PASSED ===");
}
