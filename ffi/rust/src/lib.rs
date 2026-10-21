use std::ffi::{c_char, CStr, CString};
use std::os::raw::c_int;
use std::slice;

#[link(name = "kubo_ffi", kind = "static")]
extern "C" {
    // core
    fn kubo_version() -> *mut c_char;
    fn kubo_ffi_last_error() -> *mut c_char;
    fn kubo_ffi_free_string(s: *mut c_char);
    fn kubo_ffi_free_buffer(buf: *mut u8);
    fn kubo_get_logs() -> *mut c_char;

    // repo init
    fn kubo_init_repo(repo_path: *const c_char) -> i64;
    fn kubo_init_repo_networked(repo_path: *const c_char) -> i64;
    fn kubo_init_repo_with_config(repo_path: *const c_char, config_json: *const c_char) -> i64;

    // node lifecycle
    fn kubo_node_start(repo_path: *const c_char, online: u8) -> u64;
    fn kubo_node_stop(handle: u64) -> i64;
    fn kubo_node_peer_id(handle: u64) -> *mut c_char;
    fn kubo_node_id(handle: u64) -> *mut c_char;
    fn kubo_node_listening_addrs(handle: u64) -> *mut c_char;
    fn kubo_node_api_addrs(handle: u64) -> *mut c_char;
    fn kubo_swarm_peers(handle: u64) -> *mut c_char;
    fn kubo_node_connect(handle: u64, addr: *const c_char) -> i64;

    // serving
    fn kubo_node_start_api(handle: u64, multiaddr: *const c_char) -> *mut c_char;
    fn kubo_node_start_gateway(handle: u64, multiaddr: *const c_char) -> *mut c_char;
    fn kubo_node_add_static(handle: u64, prefix: *const c_char, dir: *const c_char) -> i64;
    fn kubo_node_add_crate_redirect(handle: u64, registry_dir: *const c_char) -> i64;

    // content
    fn kubo_unixfs_add_bytes(handle: u64, data: *const u8, length: usize) -> *mut c_char;
    fn kubo_unixfs_cat(
        handle: u64,
        cid_str: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;

    // commands (full coverage)
    fn kubo_run_command_line(repo_path: *const c_char, line: *const c_char) -> *mut c_char;
    fn kubo_run_command_on_node(
        repo_path: *const c_char,
        handle: u64,
        argc: c_int,
        argv: *mut *mut c_char,
    ) -> *mut c_char;

    // watch
    fn kubo_watch_path(repo_path: *const c_char, watch_path: *const c_char) -> u64;
    fn kubo_watch_stop(handle: u64) -> i64;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the Kubo version string.
pub fn version() -> String {
    unsafe {
        let p = kubo_version();
        if p.is_null() {
            return String::new();
        }
        let s = CStr::from_ptr(p).to_string_lossy().to_string();
        kubo_ffi_free_string(p);
        s
    }
}

/// Returns accumulated Go log output since the last call.
pub fn get_logs() -> String {
    unsafe {
        let p = kubo_get_logs();
        if p.is_null() {
            return String::new();
        }
        let s = CStr::from_ptr(p).to_string_lossy().to_string();
        kubo_ffi_free_string(p);
        s
    }
}

/// Returns the last error message, if any.
///
/// NOTE: the underlying Go state is process-global, not per-thread. Under a
/// multi-threaded runtime (tokio), concurrent FFI calls can race on it.
pub fn last_error() -> Option<String> {
    unsafe {
        let p = kubo_ffi_last_error();
        if p.is_null() {
            return None;
        }
        let s = CStr::from_ptr(p).to_string_lossy().to_string();
        kubo_ffi_free_string(p);
        Some(s)
    }
}

fn cstr(s: &str) -> Result<CString, String> {
    CString::new(s).map_err(|e| e.to_string())
}

fn from_cstr(p: *mut c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        let s = unsafe { CStr::from_ptr(p).to_string_lossy().to_string() };
        unsafe { kubo_ffi_free_string(p) };
        Some(s)
    }
}

// ---------------------------------------------------------------------------
// Repo init
// ---------------------------------------------------------------------------

/// Initialize an isolated (loopback-only, no bootstrap) repository.
pub fn init_repo(repo_path: &str) -> Result<(), String> {
    let c = cstr(repo_path)?;
    let rc = unsafe { kubo_init_repo(c.as_ptr()) };
    if rc != 0 {
        Err(last_error().unwrap_or_else(|| "init failed".into()))
    } else {
        Ok(())
    }
}

/// Initialize a repository configured for real network participation.
pub fn init_repo_networked(repo_path: &str) -> Result<(), String> {
    let c = cstr(repo_path)?;
    let rc = unsafe { kubo_init_repo_networked(c.as_ptr()) };
    if rc != 0 {
        Err(last_error().unwrap_or_else(|| "init failed".into()))
    } else {
        Ok(())
    }
}

/// Initialize a repository from a caller-supplied Kubo config JSON blob.
pub fn init_repo_with_config(repo_path: &str, config_json: &str) -> Result<(), String> {
    let rp = cstr(repo_path)?;
    let cj = cstr(config_json)?;
    let rc = unsafe { kubo_init_repo_with_config(rp.as_ptr(), cj.as_ptr()) };
    if rc != 0 {
        Err(last_error().unwrap_or_else(|| "init failed".into()))
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

/// A running Kubo node handle.
pub struct Node(u64);

impl Node {
    pub fn start(repo_path: &str, online: bool) -> Result<Self, String> {
        let c = cstr(repo_path)?;
        let h = unsafe { kubo_node_start(c.as_ptr(), if online { 1 } else { 0 }) };
        if h == 0 {
            Err(last_error().unwrap_or_else(|| "node start failed".into()))
        } else {
            Ok(Node(h))
        }
    }

    pub fn peer_id(&self) -> Option<String> {
        from_cstr(unsafe { kubo_node_peer_id(self.0) })
    }

    /// Returns the node's full identity as a JSON string {"id":..., "public_key":...}.
    pub fn id(&self) -> Option<String> {
        from_cstr(unsafe { kubo_node_id(self.0) })
    }

    /// Multiaddr(s) the node is listening on, one per line.
    pub fn listening_addrs(&self) -> Option<String> {
        from_cstr(unsafe { kubo_node_listening_addrs(self.0) })
    }

    /// The API listener multiaddr, if `start_api`/`start_gateway` was called.
    pub fn api_addrs(&self) -> Option<String> {
        from_cstr(unsafe { kubo_node_api_addrs(self.0) })
    }

    pub fn swarm_peers(&self) -> Vec<String> {
        match from_cstr(unsafe { kubo_swarm_peers(self.0) }) {
            Some(s) => s
                .lines()
                .map(|l| l.to_string())
                .filter(|l| !l.is_empty())
                .collect(),
            None => vec![],
        }
    }

    pub fn connect(&self, addr: &str) -> bool {
        match cstr(addr) {
            Ok(c) => unsafe { kubo_node_connect(self.0, c.as_ptr()) == 0 },
            Err(_) => false,
        }
    }

    /// Starts the full API server (gateway + CoreAPI + WebUI + commands).
    /// WARNING: exposes the writable CoreAPI (/api/v0/*). Do not bind to a
    /// public address. Prefer `start_gateway` for registry servers.
    pub fn start_api(&self, multiaddr: &str) -> Option<String> {
        let c = cstr(multiaddr).ok()?;
        from_cstr(unsafe { kubo_node_start_api(self.0, c.as_ptr()) })
    }

    /// Starts a public-safe server: IPFS gateway + static dirs only.
    /// No writable CoreAPI, no WebUI. Use this for registry servers.
    pub fn start_gateway(&self, multiaddr: &str) -> Option<String> {
        let c = cstr(multiaddr).ok()?;
        from_cstr(unsafe { kubo_node_start_gateway(self.0, c.as_ptr()) })
    }

    pub fn add_static(&self, prefix: &str, dir: &str) -> bool {
        let p = match cstr(prefix) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let d = match cstr(dir) {
            Ok(c) => c,
            Err(_) => return false,
        };
        unsafe { kubo_node_add_static(self.0, p.as_ptr(), d.as_ptr()) == 0 }
    }

    /// Register a redirect handler: /crates/... requests redirect to /ipfs/<cid>
    /// when the index entry has an ipfs_cid. Call before start_gateway.
    pub fn add_crate_redirect(&self, registry_dir: &str) -> bool {
        match cstr(registry_dir) {
            Ok(c) => unsafe { kubo_node_add_crate_redirect(self.0, c.as_ptr()) == 0 },
            Err(_) => false,
        }
    }

    pub fn stop(&self) {
        unsafe {
            kubo_node_stop(self.0);
        }
    }

    // ---- content ----

    /// Adds bytes to IPFS and returns the root CID. Content is PINNED.
    pub fn unixfs_add_bytes(&self, data: &[u8]) -> Result<String, String> {
        let p = unsafe { kubo_unixfs_add_bytes(self.0, data.as_ptr(), data.len()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "add failed".into()))
    }

    /// Fetches the bytes for a CID. Returns the raw bytes (binary-safe).
    pub fn unixfs_cat(&self, cid: &str) -> Result<Vec<u8>, String> {
        let c = cstr(cid)?;
        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let rc = unsafe { kubo_unixfs_cat(self.0, c.as_ptr(), &mut out, &mut out_len) };
        if rc != 0 {
            return Err(last_error().unwrap_or_else(|| "cat failed".into()));
        }
        if out.is_null() || out_len == 0 {
            return Ok(Vec::new());
        }
        let bytes = unsafe { slice::from_raw_parts(out, out_len).to_vec() };
        unsafe { kubo_ffi_free_buffer(out) };
        Ok(bytes)
    }

    // ---- commands (full coverage) ----

    /// Runs a Kubo command on a *running* node, reusing its handle (preferred).
    ///
    /// `args` must NOT include the leading "ipfs" token, e.g. `&["version"]`
    /// or `&["name", "publish", "/ipfs/..."]`. Output is the command's emitted
    /// value, which is often a Go struct debug dump rather than plain text; do
    /// NOT use this for binary content — use `unixfs_cat` instead.
    pub fn run_on_node(&self, repo_path: &str, args: &[&str]) -> Result<String, String> {
        let rp = cstr(repo_path)?;
        // Build argv as C strings.
        let mut cstrings: Vec<CString> = Vec::with_capacity(args.len());
        for a in args {
            cstrings.push(cstr(a)?);
        }
        let mut argv: Vec<*mut c_char> =
            cstrings.iter().map(|c| c.as_ptr() as *mut c_char).collect();
        let p = unsafe {
            kubo_run_command_on_node(
                rp.as_ptr(),
                self.0,
                argv.len() as c_int,
                if argv.is_empty() {
                    std::ptr::null_mut()
                } else {
                    argv.as_mut_ptr()
                },
            )
        };
        // Keep cstrings alive for the duration of the call (already done above).
        if p.is_null() {
            Err(last_error().unwrap_or_else(|| "command failed".into()))
        } else {
            let s = unsafe { CStr::from_ptr(p).to_string_lossy().to_string() };
            unsafe { kubo_ffi_free_string(p) };
            Ok(s)
        }
    }

    /// Runs a Kubo command against a repo, spawning its own node.
    /// Prefer `run_on_node` when a node is already running.
    ///
    /// `line` must NOT include the leading "ipfs" token, e.g. "version".
    pub fn run(&self, repo_path: &str, line: &str) -> Result<String, String> {
        let rp = cstr(repo_path)?;
        let l = cstr(line)?;
        let p = unsafe { kubo_run_command_line(rp.as_ptr(), l.as_ptr()) };
        if p.is_null() {
            Err(last_error().unwrap_or_else(|| "command failed".into()))
        } else {
            let s = unsafe { CStr::from_ptr(p).to_string_lossy().to_string() };
            unsafe { kubo_ffi_free_string(p) };
            // Surface an error reported via last_error even on non-null output.
            if let Some(e) = last_error() {
                // Only treat as error if output is empty.
                if s.trim().is_empty() {
                    return Err(e);
                }
            }
            Ok(s)
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            kubo_node_stop(self.0);
        }
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

pub fn run_command(repo_path: &str, line: &str) -> Result<String, String> {
    let rp = cstr(repo_path)?;
    let l = cstr(line)?;
    let p = unsafe { kubo_run_command_line(rp.as_ptr(), l.as_ptr()) };
    if p.is_null() {
        Err(last_error().unwrap_or_else(|| "command failed".into()))
    } else {
        let s = unsafe { CStr::from_ptr(p).to_string_lossy().to_string() };
        unsafe { kubo_ffi_free_string(p) };
        Ok(s)
    }
}

// ---------------------------------------------------------------------------
// Watch
// ---------------------------------------------------------------------------

pub struct WatchHandle(u64);

impl WatchHandle {
    pub fn start(repo_path: &str, watch_path: &str) -> Result<Self, String> {
        let rp = cstr(repo_path)?;
        let wp = cstr(watch_path)?;
        let h = unsafe { kubo_watch_path(rp.as_ptr(), wp.as_ptr()) };
        if h == 0 {
            Err(last_error().unwrap_or_else(|| "watch start failed".into()))
        } else {
            Ok(WatchHandle(h))
        }
    }

    pub fn stop(&self) -> Result<(), String> {
        let rc = unsafe { kubo_watch_stop(self.0) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "watch stop failed".into()))
        } else {
            Ok(())
        }
    }
}

impl Drop for WatchHandle {
    fn drop(&mut self) {
        unsafe {
            kubo_watch_stop(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_works() {
        assert!(!version().is_empty());
    }
}
