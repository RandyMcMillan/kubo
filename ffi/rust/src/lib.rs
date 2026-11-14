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
    fn kubo_node_set_auth_token(handle: u64, auth_token: *const c_char) -> i64;
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

    // git (extended)
    fn kubo_git_fetch_all(path: *const c_char) -> i64;
    fn kubo_git_blame(path: *const c_char, file_path: *const c_char) -> *mut c_char;
    fn kubo_git_log(path: *const c_char, max_count: c_int) -> *mut c_char;
    fn kubo_git_tags(path: *const c_char) -> *mut c_char;

    // nostr (extended)
    fn kubo_nostr_generate_key() -> *mut c_char;
    fn kubo_nostr_get_public_key(sk: *const c_char) -> *mut c_char;
    fn kubo_nostr_event_sign(sk: *const c_char, content: *const c_char, kind: c_int) -> *mut c_char;
    fn kubo_nostr_event_sign_with_tags(
        sk: *const c_char,
        content: *const c_char,
        kind: c_int,
        tags_json: *const c_char,
    ) -> *mut c_char;
    fn kubo_nostr_relay_connect(url: *const c_char) -> u64;
    fn kubo_nostr_relay_close(handle: u64) -> i64;
    fn kubo_nostr_relay_publish(handle: u64, event_json: *const c_char) -> i64;
    fn kubo_nostr_relay_subscribe(handle: u64, filter_json: *const c_char) -> u64;
    fn kubo_nostr_relay_drain(sub_handle: u64) -> *mut c_char;
    fn kubo_nostr_relay_unsubscribe(sub_handle: u64) -> i64;

    // swarm key helpers
    fn kubo_nostr_nsec_to_swarm_key(nsec_or_hex: *const c_char) -> *mut c_char;
    fn kubo_swarm_key_generate() -> *mut c_char;

    // blocks
    fn kubo_block_put(handle: u64, data: *const u8, length: usize) -> *mut c_char;
    fn kubo_block_get(
        handle: u64,
        cid_str: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;
    fn kubo_block_stat(handle: u64, cid_str: *const c_char) -> i64;

    // git (repo handles)
    fn kubo_git_clone(url: *const c_char, path: *const c_char, bare: u8) -> i64;
    fn kubo_git_open(path: *const c_char) -> u64;
    fn kubo_git_init(path: *const c_char, bare: u8) -> i64;
    fn kubo_git_repo_head(handle: u64) -> *mut c_char;
    fn kubo_git_repo_free(handle: u64) -> i64;
    fn kubo_git_repo_is_bare(handle: u64) -> i64;
    fn kubo_git_repo_branches(handle: u64) -> *mut c_char;
    fn kubo_git_repo_remotes(handle: u64) -> *mut c_char;
    fn kubo_git_repo_create_branch(
        handle: u64,
        name: *const c_char,
        commit_hash: *const c_char,
    ) -> i64;
    fn kubo_git_repo_commit_lookup(handle: u64, hash: *const c_char) -> *mut c_char;
    fn kubo_git_repo_tree_entries(handle: u64, hash: *const c_char) -> *mut c_char;
    fn kubo_git_repo_blob_read(
        handle: u64,
        hash: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;
    fn kubo_git_repo_status(handle: u64) -> *mut c_char;
    fn kubo_git_repo_diff_trees(
        handle: u64,
        old_hash: *const c_char,
        new_hash: *const c_char,
    ) -> *mut c_char;
    fn kubo_git_push(path: *const c_char, remote_name: *const c_char) -> i64;

    // nostr extras
    fn kubo_nostr_event_verify(json_str: *const c_char) -> i64;
    fn kubo_nostr_nip19_encode_pubkey(hex: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_decode_pubkey(bech32: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_encode_seckey(hex: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_decode_seckey(bech32: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_encode_note(hex: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_decode_note(bech32: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_encode_entity(
        pubkey: *const c_char,
        kind: c_int,
        identifier: *const c_char,
        relays: *const c_char,
    ) -> *mut c_char;
    fn kubo_nostr_nip19_decode_entity(bech32: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip05_verify(identifier: *const c_char, pubkey: *const c_char) -> i64;
    fn kubo_nostr_nip05_query(identifier: *const c_char) -> *mut c_char;

    // pin + dag + unixfs ls (Batch A)
    fn kubo_pin_add(handle: u64, path_str: *const c_char) -> i64;
    fn kubo_pin_rm(handle: u64, path_str: *const c_char) -> i64;
    fn kubo_pin_ls(handle: u64) -> *mut c_char;
    fn kubo_pin_is_pinned(handle: u64, path_str: *const c_char) -> i64;
    fn kubo_pin_update(handle: u64, from_str: *const c_char, to_str: *const c_char) -> i64;
    fn kubo_pin_verify(handle: u64) -> *mut c_char;
    fn kubo_dag_import(handle: u64, data: *const u8, length: usize) -> *mut c_char;
    fn kubo_unixfs_ls(handle: u64, path_str: *const c_char) -> *mut c_char;

    // swarm completion (Batch B)
    fn kubo_swarm_disconnect(handle: u64, addr: *const c_char) -> i64;
    fn kubo_swarm_local_addrs(handle: u64) -> *mut c_char;
    fn kubo_swarm_listen_addrs(handle: u64) -> *mut c_char;
    fn kubo_swarm_known_addrs(handle: u64) -> *mut c_char;

    // IPNS + keys (Batch C)
    fn kubo_name_publish(handle: u64, path_str: *const c_char) -> *mut c_char;
    fn kubo_name_resolve(handle: u64, name: *const c_char) -> *mut c_char;
    fn kubo_name_search(handle: u64, name: *const c_char) -> *mut c_char;
    fn kubo_key_generate(handle: u64, name: *const c_char) -> *mut c_char;
    fn kubo_key_list(handle: u64) -> *mut c_char;
    fn kubo_key_self(handle: u64) -> *mut c_char;
    fn kubo_key_rename(handle: u64, old_name: *const c_char, new_name: *const c_char) -> *mut c_char;
    fn kubo_key_remove(handle: u64, name: *const c_char) -> *mut c_char;

    // objects (Batch D)
    fn kubo_object_add_link(
        handle: u64,
        base: *const c_char,
        name: *const c_char,
        child: *const c_char,
    ) -> *mut c_char;
    fn kubo_object_rm_link(handle: u64, base: *const c_char, link: *const c_char) -> *mut c_char;
    fn kubo_object_diff(
        handle: u64,
        base: *const c_char,
        target: *const c_char,
    ) -> *mut c_char;

    // pubsub + routing (Batch E)
    fn kubo_pubsub_publish(handle: u64, topic: *const c_char, data: *const u8, length: usize) -> i64;
    fn kubo_pubsub_ls(handle: u64) -> *mut c_char;
    fn kubo_pubsub_peers(handle: u64, topic: *const c_char) -> *mut c_char;
    fn kubo_pubsub_subscribe(handle: u64, topic: *const c_char) -> u64;
    fn kubo_pubsub_next(sub_handle: u64) -> *mut c_char;
    fn kubo_pubsub_unsubscribe(sub_handle: u64) -> i64;
    fn kubo_routing_get(handle: u64, key: *const c_char, out: *mut *mut u8, out_len: *mut usize) -> i64;
    fn kubo_routing_put(handle: u64, key: *const c_char, value: *const u8, length: usize) -> i64;
    fn kubo_routing_find_peer(handle: u64, peer_id: *const c_char) -> *mut c_char;
    fn kubo_routing_find_providers(handle: u64, path_str: *const c_char) -> *mut c_char;
    fn kubo_routing_provide(handle: u64, path_str: *const c_char) -> i64;
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

    /// Set a Bearer token for gateway authentication. Call before start_gateway.
    pub fn set_auth_token(&self, token: &str) -> bool {
        match cstr(token) {
            Ok(c) => unsafe { kubo_node_set_auth_token(self.0, c.as_ptr()) == 0 },
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

// ---------------------------------------------------------------------------
// Git helpers (extended)
// ---------------------------------------------------------------------------

pub fn git_fetch_all(path: &str) -> Result<(), String> {
    let c = cstr(path)?;
    let rc = unsafe { kubo_git_fetch_all(c.as_ptr()) };
    if rc != 0 {
        Err(last_error().unwrap_or_else(|| "fetch failed".into()))
    } else {
        Ok(())
    }
}

pub fn git_blame(path: &str, file_path: &str) -> Result<String, String> {
    let p = cstr(path)?;
    let f = cstr(file_path)?;
    from_cstr(unsafe { kubo_git_blame(p.as_ptr(), f.as_ptr()) })
        .ok_or_else(|| last_error().unwrap_or_else(|| "blame failed".into()))
}

pub fn git_log(path: &str, max_count: i32) -> Result<String, String> {
    let c = cstr(path)?;
    from_cstr(unsafe { kubo_git_log(c.as_ptr(), max_count) })
        .ok_or_else(|| last_error().unwrap_or_else(|| "log failed".into()))
}

pub fn git_tags(path: &str) -> Result<String, String> {
    let c = cstr(path)?;
    from_cstr(unsafe { kubo_git_tags(c.as_ptr()) })
        .ok_or_else(|| last_error().unwrap_or_else(|| "tags failed".into()))
}

// ---------------------------------------------------------------------------
// Swarm key helpers (Nostr → IPFS private swarm)
// ---------------------------------------------------------------------------

/// Derive an IPFS swarm key from a Nostr private key (nsec bech32 or hex).
/// The returned string is in IPFS PSK format and can be written to a swarm.key
/// file or passed to init_repo_with_config / kubo_init_repo_with_swarm_key.
pub fn nostr_nsec_to_swarm_key(nsec_or_hex: &str) -> Result<String, String> {
    let c = cstr(nsec_or_hex)?;
    from_cstr(unsafe { kubo_nostr_nsec_to_swarm_key(c.as_ptr()) })
        .ok_or_else(|| last_error().unwrap_or_else(|| "swarm key derivation failed".into()))
}

/// Generate a random IPFS swarm key.
pub fn swarm_key_generate() -> Result<String, String> {
    from_cstr(unsafe { kubo_swarm_key_generate() })
        .ok_or_else(|| last_error().unwrap_or_else(|| "swarm key generation failed".into()))
}

// ---------------------------------------------------------------------------
// Nostr helpers (extended)
// ---------------------------------------------------------------------------

pub fn nostr_generate_key() -> Result<String, String> {
    from_cstr(unsafe { kubo_nostr_generate_key() })
        .ok_or_else(|| last_error().unwrap_or_else(|| "keygen failed".into()))
}

pub fn nostr_get_public_key(sk: &str) -> Result<String, String> {
    let c = cstr(sk)?;
    from_cstr(unsafe { kubo_nostr_get_public_key(c.as_ptr()) })
        .ok_or_else(|| last_error().unwrap_or_else(|| "pubkey failed".into()))
}

pub fn nostr_event_sign_with_tags(
    sk: &str,
    content: &str,
    kind: i32,
    tags_json: &str,
) -> Result<String, String> {
    let s = cstr(sk)?;
    let c = cstr(content)?;
    let t = cstr(tags_json)?;
    from_cstr(unsafe { kubo_nostr_event_sign_with_tags(s.as_ptr(), c.as_ptr(), kind, t.as_ptr()) })
        .ok_or_else(|| last_error().unwrap_or_else(|| "sign failed".into()))
}

pub fn nostr_event_sign(sk: &str, content: &str, kind: i32) -> Result<String, String> {
    let s = cstr(sk)?;
    let c = cstr(content)?;
    from_cstr(unsafe { kubo_nostr_event_sign(s.as_ptr(), c.as_ptr(), kind) })
        .ok_or_else(|| last_error().unwrap_or_else(|| "sign failed".into()))
}

pub struct NostrRelay(u64);

impl NostrRelay {
    pub fn connect(url: &str) -> Result<Self, String> {
        let c = cstr(url)?;
        let h = unsafe { kubo_nostr_relay_connect(c.as_ptr()) };
        if h == 0 {
            Err(last_error().unwrap_or_else(|| "connect failed".into()))
        } else {
            Ok(NostrRelay(h))
        }
    }

    pub fn publish(&self, event_json: &str) -> Result<(), String> {
        let e = cstr(event_json)?;
        let rc = unsafe { kubo_nostr_relay_publish(self.0, e.as_ptr()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "publish failed".into()))
        } else {
            Ok(())
        }
    }

    pub fn subscribe(&self, filter_json: &str) -> Result<NostrRelaySub, String> {
        NostrRelaySub::subscribe(self.0, filter_json)
    }

    pub fn close(&self) {
        unsafe { kubo_nostr_relay_close(self.0); }
    }
}

impl Drop for NostrRelay {
    fn drop(&mut self) {
        unsafe { kubo_nostr_relay_close(self.0); }
    }
}

pub struct NostrRelaySub(u64);

impl NostrRelaySub {
    pub fn subscribe(relay_handle: u64, filter_json: &str) -> Result<Self, String> {
        let f = cstr(filter_json)?;
        let h = unsafe { kubo_nostr_relay_subscribe(relay_handle, f.as_ptr()) };
        if h == 0 {
            Err(last_error().unwrap_or_else(|| "subscribe failed".into()))
        } else {
            Ok(NostrRelaySub(h))
        }
    }

    pub fn drain(&self) -> Option<String> {
        from_cstr(unsafe { kubo_nostr_relay_drain(self.0) })
    }

    pub fn unsubscribe(&self) -> Result<(), String> {
        let rc = unsafe { kubo_nostr_relay_unsubscribe(self.0) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "unsubscribe failed".into()))
        } else {
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Blocks (Phase 0)
// ---------------------------------------------------------------------------

impl Node {
    /// Store a raw block; returns its CID string.
    pub fn block_put(&self, data: &[u8]) -> Result<String, String> {
        let p = unsafe { kubo_block_put(self.0, data.as_ptr(), data.len()) };
        if p.is_null() {
            Err(last_error().unwrap_or_else(|| "block put failed".into()))
        } else {
            Ok(from_cstr(p).unwrap_or_default())
        }
    }

    /// Fetch a raw block by CID.
    pub fn block_get(&self, cid: &str) -> Result<Vec<u8>, String> {
        let c = cstr(cid)?;
        let mut out: *mut u8 = std::ptr::null_mut();
        let mut len: usize = 0;
        let rc = unsafe { kubo_block_get(self.0, c.as_ptr(), &mut out, &mut len) };
        if rc != 0 {
            return Err(last_error().unwrap_or_else(|| "block get failed".into()));
        }
        if out.is_null() {
            return Ok(Vec::new());
        }
        let data = unsafe { slice::from_raw_parts(out, len) }.to_vec();
        unsafe { kubo_ffi_free_buffer(out) };
        Ok(data)
    }

    /// Size of a block by CID, or error if absent.
    pub fn block_stat(&self, cid: &str) -> Result<i64, String> {
        let c = cstr(cid)?;
        let rc = unsafe { kubo_block_stat(self.0, c.as_ptr()) };
        if rc < 0 {
            Err(last_error().unwrap_or_else(|| "block stat failed".into()))
        } else {
            Ok(rc)
        }
    }
}

// ---------------------------------------------------------------------------
// Git repos (Phase 0)
// ---------------------------------------------------------------------------

/// An opened git repository handle. Frees itself on drop.
pub struct GitRepo(u64);

impl GitRepo {
    /// Clone `url` into `path`; `bare` for a bare clone.
    pub fn clone(url: &str, path: &str, bare: bool) -> Result<(), String> {
        let u = cstr(url)?;
        let p = cstr(path)?;
        let rc = unsafe { kubo_git_clone(u.as_ptr(), p.as_ptr(), if bare { 1 } else { 0 }) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "git clone failed".into()))
        } else {
            Ok(())
        }
    }

    /// Create a new repository at `path`.
    pub fn init(path: &str, bare: bool) -> Result<(), String> {
        let p = cstr(path)?;
        let rc = unsafe { kubo_git_init(p.as_ptr(), if bare { 1 } else { 0 }) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "git init failed".into()))
        } else {
            Ok(())
        }
    }

    /// Open an existing repository.
    pub fn open(path: &str) -> Result<Self, String> {
        let p = cstr(path)?;
        let h = unsafe { kubo_git_open(p.as_ptr()) };
        if h == 0 {
            Err(last_error().unwrap_or_else(|| "git open failed".into()))
        } else {
            Ok(GitRepo(h))
        }
    }

    /// Current HEAD as a hash string.
    pub fn head(&self) -> Option<String> {
        from_cstr(unsafe { kubo_git_repo_head(self.0) })
    }

    pub fn is_bare(&self) -> bool {
        (unsafe { kubo_git_repo_is_bare(self.0) }) != 0
    }

    /// Branch names, one per line.
    pub fn branches(&self) -> Option<String> {
        from_cstr(unsafe { kubo_git_repo_branches(self.0) })
    }

    /// Remotes, one per line.
    pub fn remotes(&self) -> Option<String> {
        from_cstr(unsafe { kubo_git_repo_remotes(self.0) })
    }

    /// Create a branch at an existing commit hash.
    pub fn create_branch(&self, name: &str, commit_hash: &str) -> Result<(), String> {
        let n = cstr(name)?;
        let h = cstr(commit_hash)?;
        let rc = unsafe { kubo_git_repo_create_branch(self.0, n.as_ptr(), h.as_ptr()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "create_branch failed".into()))
        } else {
            Ok(())
        }
    }

    /// Look up a commit; returns a JSON description.
    pub fn commit_lookup(&self, hash: &str) -> Result<String, String> {
        let h = cstr(hash)?;
        let p = unsafe { kubo_git_repo_commit_lookup(self.0, h.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "commit lookup failed".into()))
    }

    /// List a tree object's entries as JSON.
    pub fn tree_entries(&self, hash: &str) -> Result<String, String> {
        let h = cstr(hash)?;
        let p = unsafe { kubo_git_repo_tree_entries(self.0, h.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "tree entries failed".into()))
    }

    /// Read a blob object's raw bytes.
    pub fn blob_read(&self, hash: &str) -> Result<Vec<u8>, String> {
        let h = cstr(hash)?;
        let mut out: *mut u8 = std::ptr::null_mut();
        let mut len: usize = 0;
        let rc = unsafe { kubo_git_repo_blob_read(self.0, h.as_ptr(), &mut out, &mut len) };
        if rc != 0 {
            return Err(last_error().unwrap_or_else(|| "blob read failed".into()));
        }
        if out.is_null() {
            return Ok(Vec::new());
        }
        let data = unsafe { slice::from_raw_parts(out, len) }.to_vec();
        unsafe { kubo_ffi_free_buffer(out) };
        Ok(data)
    }

    /// Working-tree status as JSON.
    pub fn status(&self) -> Result<String, String> {
        let p = unsafe { kubo_git_repo_status(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "status failed".into()))
    }

    /// Diff two trees as JSON.
    pub fn diff_trees(&self, old_hash: &str, new_hash: &str) -> Result<String, String> {
        let o = cstr(old_hash)?;
        let n = cstr(new_hash)?;
        let p = unsafe { kubo_git_repo_diff_trees(self.0, o.as_ptr(), n.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "diff failed".into()))
    }
}

impl Drop for GitRepo {
    fn drop(&mut self) {
        unsafe {
            kubo_git_repo_free(self.0);
        }
    }
}

/// Push `remote_name` in the repository at `path`.
pub fn git_push(path: &str, remote_name: &str) -> Result<(), String> {
    let p = cstr(path)?;
    let r = cstr(remote_name)?;
    let rc = unsafe { kubo_git_push(p.as_ptr(), r.as_ptr()) };
    if rc != 0 {
        Err(last_error().unwrap_or_else(|| "git push failed".into()))
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Nostr extras (Phase 0)
// ---------------------------------------------------------------------------

/// Verify a signed event JSON; Ok(()) means valid.
pub fn nostr_event_verify(event_json: &str) -> Result<(), String> {
    let j = cstr(event_json)?;
    let rc = unsafe { kubo_nostr_event_verify(j.as_ptr()) };
    if rc != 0 {
        Err(last_error().unwrap_or_else(|| "event verify failed".into()))
    } else {
        Ok(())
    }
}

/// npub encode a hex public key.
pub fn nip19_encode_pubkey(hex: &str) -> Result<String, String> {
    let h = cstr(hex)?;
    let p = unsafe { kubo_nostr_nip19_encode_pubkey(h.as_ptr()) };
    from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "encode failed".into()))
}

/// npub decode to hex.
pub fn nip19_decode_pubkey(bech32: &str) -> Result<String, String> {
    let b = cstr(bech32)?;
    let p = unsafe { kubo_nostr_nip19_decode_pubkey(b.as_ptr()) };
    from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "decode failed".into()))
}

/// nsec encode a hex secret key.
pub fn nip19_encode_seckey(hex: &str) -> Result<String, String> {
    let h = cstr(hex)?;
    let p = unsafe { kubo_nostr_nip19_encode_seckey(h.as_ptr()) };
    from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "encode failed".into()))
}

/// nsec decode to hex.
pub fn nip19_decode_seckey(bech32: &str) -> Result<String, String> {
    let b = cstr(bech32)?;
    let p = unsafe { kubo_nostr_nip19_decode_seckey(b.as_ptr()) };
    from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "decode failed".into()))
}

/// note1 encode an event-id hex.
pub fn nip19_encode_note(hex: &str) -> Result<String, String> {
    let h = cstr(hex)?;
    let p = unsafe { kubo_nostr_nip19_encode_note(h.as_ptr()) };
    from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "encode failed".into()))
}

/// note1 decode to event-id hex.
pub fn nip19_decode_note(bech32: &str) -> Result<String, String> {
    let b = cstr(bech32)?;
    let p = unsafe { kubo_nostr_nip19_decode_note(b.as_ptr()) };
    from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "decode failed".into()))
}

/// naddr encode (kind, identifier, relays JSON).
pub fn nip19_encode_entity(
    pubkey_hex: &str,
    kind: i32,
    identifier: &str,
    relays_json: &str,
) -> Result<String, String> {
    let p = cstr(pubkey_hex)?;
    let i = cstr(identifier)?;
    let r = cstr(relays_json)?;
    let out = unsafe { kubo_nostr_nip19_encode_entity(p.as_ptr(), kind, i.as_ptr(), r.as_ptr()) };
    from_cstr(out).ok_or_else(|| last_error().unwrap_or_else(|| "encode failed".into()))
}

/// naddr decode; returns the entity JSON.
pub fn nip19_decode_entity(bech32: &str) -> Result<String, String> {
    let b = cstr(bech32)?;
    let p = unsafe { kubo_nostr_nip19_decode_entity(b.as_ptr()) };
    from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "decode failed".into()))
}

/// NIP-05 verify: does `identifier`'s server declare `pubkey_hex`?
pub fn nip05_verify(identifier: &str, pubkey_hex: &str) -> Result<(), String> {
    let i = cstr(identifier)?;
    let p = cstr(pubkey_hex)?;
    let rc = unsafe { kubo_nostr_nip05_verify(i.as_ptr(), p.as_ptr()) };
    if rc != 0 {
        Err(last_error().unwrap_or_else(|| "nip05 verify failed".into()))
    } else {
        Ok(())
    }
}

/// NIP-05 query: returns the pubkey JSON for an identifier.
pub fn nip05_query(identifier: &str) -> Result<String, String> {
    let i = cstr(identifier)?;
    let p = unsafe { kubo_nostr_nip05_query(i.as_ptr()) };
    from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "nip05 query failed".into()))
}

// ---------------------------------------------------------------------------
// Pin + Dag + Unixfs ls (Batch A)
// ---------------------------------------------------------------------------

impl Node {
    /// Pin a CID or /ipfs/... path.
    pub fn pin_add(&self, cid_or_path: &str) -> Result<(), String> {
        let c = cstr(cid_or_path)?;
        let rc = unsafe { kubo_pin_add(self.0, c.as_ptr()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "pin add failed".into()))
        } else {
            Ok(())
        }
    }

    /// Unpin a CID or path.
    pub fn pin_rm(&self, cid_or_path: &str) -> Result<(), String> {
        let c = cstr(cid_or_path)?;
        let rc = unsafe { kubo_pin_rm(self.0, c.as_ptr()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "pin rm failed".into()))
        } else {
            Ok(())
        }
    }

    /// List pins as a JSON array of {path, name, type}.
    pub fn pin_ls(&self) -> Result<String, String> {
        let p = unsafe { kubo_pin_ls(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "pin ls failed".into()))
    }

    /// Whether a CID or path is pinned.
    pub fn pin_is_pinned(&self, cid_or_path: &str) -> Result<bool, String> {
        let c = cstr(cid_or_path)?;
        let rc = unsafe { kubo_pin_is_pinned(self.0, c.as_ptr()) };
        if rc < 0 {
            Err(last_error().unwrap_or_else(|| "pin is_pinned failed".into()))
        } else {
            Ok(rc == 1)
        }
    }

    /// Move a pin from one root to another.
    pub fn pin_update(&self, from: &str, to: &str) -> Result<(), String> {
        let f = cstr(from)?;
        let t = cstr(to)?;
        let rc = unsafe { kubo_pin_update(self.0, f.as_ptr(), t.as_ptr()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "pin update failed".into()))
        } else {
            Ok(())
        }
    }

    /// Verify all pins; returns a JSON array of {ok, bad_nodes, err}.
    pub fn pin_verify(&self) -> Result<String, String> {
        let p = unsafe { kubo_pin_verify(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "pin verify failed".into()))
    }

    /// Import a CAR archive (bytes); pins the roots. Returns {"roots":[...]} JSON.
    pub fn dag_import(&self, car: &[u8]) -> Result<String, String> {
        let p = unsafe { kubo_dag_import(self.0, car.as_ptr(), car.len()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "dag import failed".into()))
    }

    /// List a UnixFS directory as a JSON array of {name, cid, size, type}.
    pub fn unixfs_ls(&self, cid_or_path: &str) -> Result<String, String> {
        let c = cstr(cid_or_path)?;
        let p = unsafe { kubo_unixfs_ls(self.0, c.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "unixfs ls failed".into()))
    }
}

// ---------------------------------------------------------------------------
// Swarm completion (Batch B)
// ---------------------------------------------------------------------------

impl Node {
    /// Disconnect from a peer multiaddr.
    pub fn swarm_disconnect(&self, multiaddr: &str) -> Result<(), String> {
        let a = cstr(multiaddr)?;
        let rc = unsafe { kubo_swarm_disconnect(self.0, a.as_ptr()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "swarm disconnect failed".into()))
        } else {
            Ok(())
        }
    }

    /// This node's own addresses as a JSON array of multiaddr strings.
    pub fn swarm_local_addrs(&self) -> Result<String, String> {
        let p = unsafe { kubo_swarm_local_addrs(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "local addrs failed".into()))
    }

    /// Addresses the swarm is listening on, JSON array.
    pub fn swarm_listen_addrs(&self) -> Result<String, String> {
        let p = unsafe { kubo_swarm_listen_addrs(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "listen addrs failed".into()))
    }

    /// Known addresses per connected peer: JSON map peer_id -> [multiaddr].
    pub fn swarm_known_addrs(&self) -> Result<String, String> {
        let p = unsafe { kubo_swarm_known_addrs(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "known addrs failed".into()))
    }
}

// ---------------------------------------------------------------------------
// IPNS + keys (Batch C)
// ---------------------------------------------------------------------------

impl Node {
    /// Publish an IPNS record for a path; returns the IPNS name.
    pub fn name_publish(&self, cid_or_path: &str) -> Result<String, String> {
        let c = cstr(cid_or_path)?;
        let p = unsafe { kubo_name_publish(self.0, c.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "name publish failed".into()))
    }

    /// Resolve an IPNS name to a path.
    pub fn name_resolve(&self, name: &str) -> Result<String, String> {
        let n = cstr(name)?;
        let p = unsafe { kubo_name_resolve(self.0, n.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "name resolve failed".into()))
    }

    /// Search an IPNS name through the DHT; JSON array of {path, err}.
    pub fn name_search(&self, name: &str) -> Result<String, String> {
        let n = cstr(name)?;
        let p = unsafe { kubo_name_search(self.0, n.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "name search failed".into()))
    }

    /// Generate a new IPNS key; returns {name, path} JSON.
    pub fn key_generate(&self, name: &str) -> Result<String, String> {
        let n = cstr(name)?;
        let p = unsafe { kubo_key_generate(self.0, n.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "key generate failed".into()))
    }

    /// List keys as a JSON array of {name, path}.
    pub fn key_list(&self) -> Result<String, String> {
        let p = unsafe { kubo_key_list(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "key list failed".into()))
    }

    /// The node's own IPNS key.
    pub fn key_self(&self) -> Result<String, String> {
        let p = unsafe { kubo_key_self(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "key self failed".into()))
    }

    /// Rename a key; returns the updated {name, path} JSON.
    pub fn key_rename(&self, old_name: &str, new_name: &str) -> Result<String, String> {
        let o = cstr(old_name)?;
        let n = cstr(new_name)?;
        let p = unsafe { kubo_key_rename(self.0, o.as_ptr(), n.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "key rename failed".into()))
    }

    /// Remove a key; returns the removed {name, path} JSON.
    pub fn key_remove(&self, name: &str) -> Result<String, String> {
        let n = cstr(name)?;
        let p = unsafe { kubo_key_remove(self.0, n.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "key remove failed".into()))
    }
}

// ---------------------------------------------------------------------------
// Objects (Batch D)
// ---------------------------------------------------------------------------

impl Node {
    /// Add a named link to a DAG object; returns the new root path.
    pub fn object_add_link(&self, base: &str, name: &str, child: &str) -> Result<String, String> {
        let b = cstr(base)?;
        let n = cstr(name)?;
        let c = cstr(child)?;
        let p = unsafe { kubo_object_add_link(self.0, b.as_ptr(), n.as_ptr(), c.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "add link failed".into()))
    }

    /// Remove a named link from a DAG object; returns the new root path.
    pub fn object_rm_link(&self, base: &str, link: &str) -> Result<String, String> {
        let b = cstr(base)?;
        let l = cstr(link)?;
        let p = unsafe { kubo_object_rm_link(self.0, b.as_ptr(), l.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "rm link failed".into()))
    }

    /// Diff two DAG objects; JSON array of {type, path, before, after}.
    pub fn object_diff(&self, base: &str, target: &str) -> Result<String, String> {
        let b = cstr(base)?;
        let t = cstr(target)?;
        let p = unsafe { kubo_object_diff(self.0, b.as_ptr(), t.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "object diff failed".into()))
    }
}

// ---------------------------------------------------------------------------
// PubSub + Routing (Batch E)
// ---------------------------------------------------------------------------

impl Node {
    /// Publish a message to a pubsub topic.
    pub fn pubsub_publish(&self, topic: &str, data: &[u8]) -> Result<(), String> {
        let t = cstr(topic)?;
        let rc = unsafe { kubo_pubsub_publish(self.0, t.as_ptr(), data.as_ptr(), data.len()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "pubsub publish failed".into()))
        } else {
            Ok(())
        }
    }

    /// Topics with active subscriptions, JSON array of strings.
    pub fn pubsub_ls(&self) -> Result<String, String> {
        let p = unsafe { kubo_pubsub_ls(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "pubsub ls failed".into()))
    }

    /// Peers subscribed to a topic (empty topic = all), JSON array of peer IDs.
    pub fn pubsub_peers(&self, topic: &str) -> Result<String, String> {
        let t = cstr(topic)?;
        let p = unsafe { kubo_pubsub_peers(self.0, t.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "pubsub peers failed".into()))
    }

    /// Subscribe to a topic; returns a subscription handle for `pubsub_next`.
    pub fn pubsub_subscribe(&self, topic: &str) -> Result<u64, String> {
        let t = cstr(topic)?;
        let h = unsafe { kubo_pubsub_subscribe(self.0, t.as_ptr()) };
        if h == 0 {
            Err(last_error().unwrap_or_else(|| "pubsub subscribe failed".into()))
        } else {
            Ok(h)
        }
    }

    /// Next message on a subscription as JSON {from, data(base64)}; blocks.
    pub fn pubsub_next(sub_handle: u64) -> Result<String, String> {
        let p = unsafe { kubo_pubsub_next(sub_handle) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "pubsub next failed".into()))
    }

    /// Unsubscribe and release the subscription handle.
    pub fn pubsub_unsubscribe(sub_handle: u64) -> Result<(), String> {
        let rc = unsafe { kubo_pubsub_unsubscribe(sub_handle) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "pubsub unsubscribe failed".into()))
        } else {
            Ok(())
        }
    }

    /// Routing value lookup by key.
    pub fn routing_get(&self, key: &str) -> Result<Vec<u8>, String> {
        let k = cstr(key)?;
        let mut out: *mut u8 = std::ptr::null_mut();
        let mut len: usize = 0;
        let rc = unsafe { kubo_routing_get(self.0, k.as_ptr(), &mut out, &mut len) };
        if rc != 0 {
            return Err(last_error().unwrap_or_else(|| "routing get failed".into()));
        }
        if out.is_null() {
            return Ok(Vec::new());
        }
        let data = unsafe { slice::from_raw_parts(out, len) }.to_vec();
        unsafe { kubo_ffi_free_buffer(out) };
        Ok(data)
    }

    /// Store a routing value under a key.
    pub fn routing_put(&self, key: &str, value: &[u8]) -> Result<(), String> {
        let k = cstr(key)?;
        let rc = unsafe { kubo_routing_put(self.0, k.as_ptr(), value.as_ptr(), value.len()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "routing put failed".into()))
        } else {
            Ok(())
        }
    }

    /// Find a peer by ID; returns {id, addrs} JSON.
    pub fn routing_find_peer(&self, peer_id: &str) -> Result<String, String> {
        let p = cstr(peer_id)?;
        let out = unsafe { kubo_routing_find_peer(self.0, p.as_ptr()) };
        from_cstr(out).ok_or_else(|| last_error().unwrap_or_else(|| "find peer failed".into()))
    }

    /// Find providers of a path; JSON array of {id, addrs}.
    pub fn routing_find_providers(&self, cid_or_path: &str) -> Result<String, String> {
        let c = cstr(cid_or_path)?;
        let p = unsafe { kubo_routing_find_providers(self.0, c.as_ptr()) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "find providers failed".into()))
    }

    /// Announce this node as a provider of a path.
    pub fn routing_provide(&self, cid_or_path: &str) -> Result<(), String> {
        let c = cstr(cid_or_path)?;
        let rc = unsafe { kubo_routing_provide(self.0, c.as_ptr()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "routing provide failed".into()))
        } else {
            Ok(())
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

    #[test]
    fn nsec_to_swarm_key() {
        let nsec = "nsec18qvaqktundgyyc74vdxdl907tq0vam2aad2ptnku9cjw8ethdgdqzmwqct";
        let key = nostr_nsec_to_swarm_key(nsec).unwrap();
        assert!(key.contains("/key/swarm/psk/1.0.0/"));
        println!("swarm key from nsec:\n{}", key);

        // hex should produce same key
        let hex = "3819d0597c9b504263d5634cdf95fe581eceed5deb5415cedc2e24e3e5776a1a";
        let key2 = nostr_nsec_to_swarm_key(hex).unwrap();
        assert_eq!(key, key2, "nsec and hex should produce same swarm key");

        // generate should produce valid key
        let gen = swarm_key_generate().unwrap();
        assert!(gen.contains("/key/swarm/psk/1.0.0/"));
        println!("generated swarm key:\n{}", gen);
    }
}
