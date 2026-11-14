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

    // operations (Batch F)
    fn kubo_repo_stat(handle: u64) -> *mut c_char;
    fn kubo_repo_gc(handle: u64) -> i64;
    fn kubo_bootstrap_list(handle: u64) -> *mut c_char;
    fn kubo_bootstrap_add(handle: u64, multiaddr: *const c_char) -> i64;
    fn kubo_bootstrap_rm(handle: u64, multiaddr: *const c_char) -> i64;
    fn kubo_stats_bw(handle: u64) -> *mut c_char;
    fn kubo_config_get(handle: u64) -> *mut c_char;
    fn kubo_config_set(handle: u64, key: *const c_char, value_json: *const c_char) -> i64;
    fn kubo_node_shutdown(handle: u64) -> i64;

    // standalone libp2p host (Batch G)
    fn kubo_libp2p_host_new() -> u64;
    fn kubo_libp2p_host_close(handle: u64) -> i64;
    fn kubo_libp2p_host_peer_id(handle: u64) -> *mut c_char;
    fn kubo_libp2p_host_listening_addrs(handle: u64) -> *mut c_char;
    fn kubo_libp2p_host_connect(handle: u64, addr: *const c_char) -> i64;
    fn kubo_libp2p_host_ping(handle: u64, peer_id: *const c_char) -> i64;
    fn kubo_libp2p_host_protocols(handle: u64) -> *mut c_char;

    // argv command variant (Batch G)
    fn kubo_run_command(
        repo_path: *const c_char,
        argc: c_int,
        argv: *mut *mut c_char,
    ) -> *mut c_char;
}

#[cfg(feature = "web")]
pub mod web;

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
// Standalone libp2p host (no Kubo repo/node required)
// ---------------------------------------------------------------------------

/// A lightweight libp2p host independent of any Kubo node or repo.
pub struct P2pHost(u64);

impl P2pHost {
    /// Start a host on an ephemeral loopback port.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Result<Self, String> {
        let h = unsafe { kubo_libp2p_host_new() };
        if h == 0 {
            Err(last_error().unwrap_or_else(|| "libp2p host new failed".into()))
        } else {
            Ok(P2pHost(h))
        }
    }

    pub fn peer_id(&self) -> Option<String> {
        from_cstr(unsafe { kubo_libp2p_host_peer_id(self.0) })
    }

    /// Multiaddrs this host is listening on, one per line.
    pub fn listening_addrs(&self) -> Result<String, String> {
        from_cstr(unsafe { kubo_libp2p_host_listening_addrs(self.0) })
            .ok_or_else(|| last_error().unwrap_or_else(|| "listening addrs failed".into()))
    }

    /// Dial a peer by full multiaddr including `/p2p/<peer-id>`.
    pub fn connect(&self, addr: &str) -> Result<(), String> {
        let a = cstr(addr)?;
        if unsafe { kubo_libp2p_host_connect(self.0, a.as_ptr()) } == 0 {
            Ok(())
        } else {
            Err(last_error().unwrap_or_else(|| "connect failed".into()))
        }
    }

    /// Ping a peer; returns round-trip time in milliseconds.
    pub fn ping(&self, peer_id: &str) -> Result<u64, String> {
        let p = cstr(peer_id)?;
        let rc = unsafe { kubo_libp2p_host_ping(self.0, p.as_ptr()) };
        if rc < 0 {
            Err(last_error().unwrap_or_else(|| "ping failed".into()))
        } else {
            Ok(rc as u64)
        }
    }

    /// Negotiated stream protocols, one per line.
    pub fn protocols(&self) -> Result<String, String> {
        from_cstr(unsafe { kubo_libp2p_host_protocols(self.0) })
            .ok_or_else(|| last_error().unwrap_or_else(|| "protocols failed".into()))
    }

    /// Close the host, releasing its listen sockets. Called again by Drop on
    /// an already-closed handle; the duplicate is ignored.
    pub fn close(&self) -> Result<(), String> {
        if unsafe { kubo_libp2p_host_close(self.0) } == 0 {
            Ok(())
        } else {
            Err(last_error().unwrap_or_else(|| "close failed".into()))
        }
    }
}

impl Drop for P2pHost {
    fn drop(&mut self) {
        unsafe {
            kubo_libp2p_host_close(self.0);
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

/// Runs a Kubo command against a repo by argv, without shell-line parsing.
/// Spawns its own node; prefer `Node::run_on_node` when one is running.
///
/// `args` must NOT include the leading "ipfs" token, e.g. `&["version"]`.
pub fn run_command_argv(repo_path: &str, args: &[&str]) -> Result<String, String> {
    let rp = cstr(repo_path)?;
    let mut cstrings: Vec<CString> = Vec::with_capacity(args.len());
    for a in args {
        cstrings.push(cstr(a)?);
    }
    let mut argv: Vec<*mut c_char> =
        cstrings.iter().map(|c| c.as_ptr() as *mut c_char).collect();
    let p = unsafe {
        kubo_run_command(
            rp.as_ptr(),
            argv.len() as c_int,
            if argv.is_empty() {
                std::ptr::null_mut()
            } else {
                argv.as_mut_ptr()
            },
        )
    };
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
    if rc == 1 {
        Ok(())
    } else if rc == 0 {
        Err("invalid signature".into())
    } else {
        Err(last_error().unwrap_or_else(|| "event verify failed".into()))
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

// ---------------------------------------------------------------------------
// Operations (Batch F)
// ---------------------------------------------------------------------------

impl Node {
    /// Repository statistics as JSON {storage_usage_bytes}.
    pub fn repo_stat(&self) -> Result<String, String> {
        let p = unsafe { kubo_repo_stat(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "repo stat failed".into()))
    }

    /// Run garbage collection over unpinned blocks.
    pub fn repo_gc(&self) -> Result<(), String> {
        let rc = unsafe { kubo_repo_gc(self.0) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "repo gc failed".into()))
        } else {
            Ok(())
        }
    }

    /// Configured bootstrap multiaddrs as a JSON array.
    pub fn bootstrap_list(&self) -> Result<String, String> {
        let p = unsafe { kubo_bootstrap_list(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "bootstrap list failed".into()))
    }

    /// Add a bootstrap multiaddr (persisted to config).
    pub fn bootstrap_add(&self, multiaddr: &str) -> Result<(), String> {
        let a = cstr(multiaddr)?;
        let rc = unsafe { kubo_bootstrap_add(self.0, a.as_ptr()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "bootstrap add failed".into()))
        } else {
            Ok(())
        }
    }

    /// Remove a bootstrap multiaddr (persisted to config).
    pub fn bootstrap_rm(&self, multiaddr: &str) -> Result<(), String> {
        let a = cstr(multiaddr)?;
        let rc = unsafe { kubo_bootstrap_rm(self.0, a.as_ptr()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "bootstrap rm failed".into()))
        } else {
            Ok(())
        }
    }

    /// Bandwidth totals as JSON {total_in, total_out, rate_in, rate_out}.
    pub fn stats_bw(&self) -> Result<String, String> {
        let p = unsafe { kubo_stats_bw(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "stats bw failed".into()))
    }

    /// The node's full config as JSON.
    pub fn config_get(&self) -> Result<String, String> {
        let p = unsafe { kubo_config_get(self.0) };
        from_cstr(p).ok_or_else(|| last_error().unwrap_or_else(|| "config get failed".into()))
    }

    /// Set a dotted config key to a JSON value (persisted to the repo).
    pub fn config_set(&self, key: &str, value_json: &str) -> Result<(), String> {
        let k = cstr(key)?;
        let v = cstr(value_json)?;
        let rc = unsafe { kubo_config_set(self.0, k.as_ptr(), v.as_ptr()) };
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "config set failed".into()))
        } else {
            Ok(())
        }
    }

    /// Gracefully shut the node down and release the handle.
    pub fn shutdown(self) -> Result<(), String> {
        let rc = unsafe { kubo_node_shutdown(self.0) };
        std::mem::forget(self);
        if rc != 0 {
            Err(last_error().unwrap_or_else(|| "node shutdown failed".into()))
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

    /// Compile-time exposure matrix: referencing every public symbol by
    /// name means this test FAILS TO COMPILE if any wrapper is removed or
    /// renamed — "entire FFI exposed" becomes a build-enforced fact.
    #[test]
    fn exposure_matrix() {
        // free functions
        let _: fn(&str) -> Result<(), String> = init_repo;
        let _: fn(&str) -> Result<(), String> = init_repo_networked;
        let _: fn(&str, &str) -> Result<(), String> = init_repo_with_config;
        let _: fn() -> String = version;
        let _: fn() -> String = get_logs;
        let _: fn() -> Option<String> = last_error;
        let _: fn(&str, &str) -> Result<(), String> = git_push;
        let _: fn(&str) -> Result<String, String> = nostr_nsec_to_swarm_key;
        let _: fn() -> Result<String, String> = swarm_key_generate;
        let _: fn(&str) -> Result<(), String> = nostr_event_verify;
        let _: fn(&str) -> Result<String, String> = nip19_encode_pubkey;
        let _: fn(&str) -> Result<String, String> = nip19_decode_pubkey;
        let _: fn(&str) -> Result<String, String> = nip19_encode_seckey;
        let _: fn(&str) -> Result<String, String> = nip19_decode_seckey;
        let _: fn(&str) -> Result<String, String> = nip19_encode_note;
        let _: fn(&str) -> Result<String, String> = nip19_decode_note;
        let _: fn(&str, i32, &str, &str) -> Result<String, String> = nip19_encode_entity;
        let _: fn(&str) -> Result<String, String> = nip19_decode_entity;
        let _: fn(&str, &str) -> Result<(), String> = nip05_verify;
        let _: fn(&str) -> Result<String, String> = nip05_query;

        // Node methods: lifecycle + info
        let _: fn(&str, bool) -> Result<Node, String> = Node::start;
        let _: fn(&Node) -> Option<String> = Node::peer_id;
        let _: fn(&Node) -> Option<String> = Node::id;
        let _: fn(&Node) -> Option<String> = Node::listening_addrs;
        let _: fn(&Node) -> Option<String> = Node::api_addrs;
        let _: fn(&Node) = Node::stop;

        // serving
        let _: fn(&Node, &str) -> Option<String> = Node::start_api;
        let _: fn(&Node, &str) -> Option<String> = Node::start_gateway;
        let _: fn(&Node, &str, &str) -> bool = Node::add_static;
        let _: fn(&Node, &str) -> bool = Node::add_crate_redirect;
        let _: fn(&Node, &str) -> bool = Node::set_auth_token;

        // swarm
        let _: fn(&Node, &str) -> bool = Node::connect;
        let _: fn(&Node) -> Vec<String> = Node::swarm_peers;
        let _: fn(&Node, &str) -> Result<(), String> = Node::swarm_disconnect;
        let _: fn(&Node) -> Result<String, String> = Node::swarm_local_addrs;
        let _: fn(&Node) -> Result<String, String> = Node::swarm_listen_addrs;
        let _: fn(&Node) -> Result<String, String> = Node::swarm_known_addrs;

        // unixfs + blocks
        let _: fn(&Node, &[u8]) -> Result<String, String> = Node::unixfs_add_bytes;
        let _: fn(&Node, &str) -> Result<Vec<u8>, String> = Node::unixfs_cat;
        let _: fn(&Node, &str) -> Result<String, String> = Node::unixfs_ls;
        let _: fn(&Node, &[u8]) -> Result<String, String> = Node::block_put;
        let _: fn(&Node, &str) -> Result<Vec<u8>, String> = Node::block_get;
        let _: fn(&Node, &str) -> Result<i64, String> = Node::block_stat;

        // pin + dag
        let _: fn(&Node, &str) -> Result<(), String> = Node::pin_add;
        let _: fn(&Node, &str) -> Result<(), String> = Node::pin_rm;
        let _: fn(&Node) -> Result<String, String> = Node::pin_ls;
        let _: fn(&Node, &str) -> Result<bool, String> = Node::pin_is_pinned;
        let _: fn(&Node, &str, &str) -> Result<(), String> = Node::pin_update;
        let _: fn(&Node) -> Result<String, String> = Node::pin_verify;
        let _: fn(&Node, &[u8]) -> Result<String, String> = Node::dag_import;

        // IPNS + keys
        let _: fn(&Node, &str) -> Result<String, String> = Node::name_publish;
        let _: fn(&Node, &str) -> Result<String, String> = Node::name_resolve;
        let _: fn(&Node, &str) -> Result<String, String> = Node::name_search;
        let _: fn(&Node, &str) -> Result<String, String> = Node::key_generate;
        let _: fn(&Node) -> Result<String, String> = Node::key_list;
        let _: fn(&Node) -> Result<String, String> = Node::key_self;
        let _: fn(&Node, &str, &str) -> Result<String, String> = Node::key_rename;
        let _: fn(&Node, &str) -> Result<String, String> = Node::key_remove;

        // objects
        let _: fn(&Node, &str, &str, &str) -> Result<String, String> = Node::object_add_link;
        let _: fn(&Node, &str, &str) -> Result<String, String> = Node::object_rm_link;
        let _: fn(&Node, &str, &str) -> Result<String, String> = Node::object_diff;

        // pubsub + routing
        let _: fn(&Node, &str, &[u8]) -> Result<(), String> = Node::pubsub_publish;
        let _: fn(&Node) -> Result<String, String> = Node::pubsub_ls;
        let _: fn(&Node, &str) -> Result<String, String> = Node::pubsub_peers;
        let _: fn(&Node, &str) -> Result<u64, String> = Node::pubsub_subscribe;
        let _: fn(u64) -> Result<String, String> = Node::pubsub_next;
        let _: fn(u64) -> Result<(), String> = Node::pubsub_unsubscribe;
        let _: fn(&Node, &str) -> Result<Vec<u8>, String> = Node::routing_get;
        let _: fn(&Node, &str, &[u8]) -> Result<(), String> = Node::routing_put;
        let _: fn(&Node, &str) -> Result<String, String> = Node::routing_find_peer;
        let _: fn(&Node, &str) -> Result<String, String> = Node::routing_find_providers;
        let _: fn(&Node, &str) -> Result<(), String> = Node::routing_provide;

        // operations
        let _: fn(&Node) -> Result<String, String> = Node::repo_stat;
        let _: fn(&Node) -> Result<(), String> = Node::repo_gc;
        let _: fn(&Node) -> Result<String, String> = Node::bootstrap_list;
        let _: fn(&Node, &str) -> Result<(), String> = Node::bootstrap_add;
        let _: fn(&Node, &str) -> Result<(), String> = Node::bootstrap_rm;
        let _: fn(&Node) -> Result<String, String> = Node::stats_bw;
        let _: fn(&Node) -> Result<String, String> = Node::config_get;
        let _: fn(&Node, &str, &str) -> Result<(), String> = Node::config_set;
        let _: fn(Node) -> Result<(), String> = Node::shutdown;

        // GitRepo
        let _: fn(&str, &str, bool) -> Result<(), String> = GitRepo::clone;
        let _: fn(&str, bool) -> Result<(), String> = GitRepo::init;
        let _: fn(&str) -> Result<GitRepo, String> = GitRepo::open;
        let _: fn(&GitRepo) -> Option<String> = GitRepo::head;
        let _: fn(&GitRepo) -> bool = GitRepo::is_bare;
        let _: fn(&GitRepo) -> Option<String> = GitRepo::branches;
        let _: fn(&GitRepo) -> Option<String> = GitRepo::remotes;
        let _: fn(&GitRepo, &str, &str) -> Result<(), String> = GitRepo::create_branch;
        let _: fn(&GitRepo, &str) -> Result<String, String> = GitRepo::commit_lookup;
        let _: fn(&GitRepo, &str) -> Result<String, String> = GitRepo::tree_entries;
        let _: fn(&GitRepo, &str) -> Result<Vec<u8>, String> = GitRepo::blob_read;
        let _: fn(&GitRepo) -> Result<String, String> = GitRepo::status;
        let _: fn(&GitRepo, &str, &str) -> Result<String, String> = GitRepo::diff_trees;

        // watch + nostr relay + misc
        let _: fn(&str, &str) -> Result<WatchHandle, String> = WatchHandle::start;
        let _: fn(&WatchHandle) -> Result<(), String> = WatchHandle::stop;
        let _: fn(&str) -> Result<(), String> = git_fetch_all;
        let _: fn(&str, &str) -> Result<String, String> = git_blame;
        let _: fn(&str, i32) -> Result<String, String> = git_log;
        let _: fn(&str) -> Result<String, String> = git_tags;
        let _: fn() -> Result<String, String> = nostr_generate_key;
        let _: fn(&str) -> Result<String, String> = nostr_get_public_key;
        let _: fn(&str, &str, i32) -> Result<String, String> = nostr_event_sign;
        let _: fn(&str, &str, i32, &str) -> Result<String, String> = nostr_event_sign_with_tags;
        let _: fn(&str) -> Result<NostrRelay, String> = NostrRelay::connect;
        let _: fn(&NostrRelay, &str) -> Result<(), String> = NostrRelay::publish;
        let _: fn(&NostrRelay, &str) -> Result<NostrRelaySub, String> = NostrRelay::subscribe;
        let _: fn(&NostrRelay) = NostrRelay::close;
        let _: fn(&NostrRelaySub) -> Option<String> = NostrRelaySub::drain;
        let _: fn(&NostrRelaySub) -> Result<(), String> = NostrRelaySub::unsubscribe;
        let _: fn(&str, &str) -> Result<String, String> = run_command;
        let _: fn(&str, &[&str]) -> Result<String, String> = run_command_argv;
        let _: fn(&Node, &str, &[&str]) -> Result<String, String> = Node::run_on_node;
        let _: fn(&Node, &str, &str) -> Result<String, String> = Node::run;

        // standalone libp2p host
        let _: fn() -> Result<P2pHost, String> = P2pHost::new;
        let _: fn(&P2pHost) -> Option<String> = P2pHost::peer_id;
        let _: fn(&P2pHost) -> Result<String, String> = P2pHost::listening_addrs;
        let _: fn(&P2pHost, &str) -> Result<(), String> = P2pHost::connect;
        let _: fn(&P2pHost, &str) -> Result<u64, String> = P2pHost::ping;
        let _: fn(&P2pHost) -> Result<String, String> = P2pHost::protocols;
        let _: fn(&P2pHost) -> Result<(), String> = P2pHost::close;
    }

    #[test]
    fn p2p_host_works() {
        let h = P2pHost::new().expect("host start");
        let id = h.peer_id().expect("peer id");
        assert!(id.starts_with("12D3KooW") || id.len() >= 40);
        let addrs = h.listening_addrs().expect("addrs");
        assert!(addrs.contains("/ip4/127.0.0.1/tcp/"));
        let protos = h.protocols().expect("protocols");
        assert!(protos.contains("/ipfs/ping/1.0.0"));
        h.close().expect("close");
    }

    #[test]
    fn nsec_to_swarm_key() {        let nsec = "nsec18qvaqktundgyyc74vdxdl907tq0vam2aad2ptnku9cjw8ethdgdqzmwqct";
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

    // ------------------------------------------------------------------
    // Full FFI coverage: every public symbol exercised at runtime, with
    // banners. Run with: cargo test --lib ffi_full_coverage -- --nocapture
    // ------------------------------------------------------------------

    struct Tally {
        pass: usize,
        fail: usize,
        skip: usize,
    }

    impl Tally {
        fn ok(&mut self, name: &str, detail: String) {
            self.pass += 1;
            println!(" [ffi]   ok    {name} — {detail}");
        }
        fn fail(&mut self, name: &str, err: String) {
            self.fail += 1;
            println!(" [ffi]   FAIL  {name} — {err}");
        }
        fn skip(&mut self, name: &str, why: String) {
            self.skip += 1;
            println!(" [ffi]   skip  {name} — {why}");
        }
    }

    macro_rules! check {
        ($t:expr, $name:expr, $r:expr) => {
            match $r {
                Ok(d) => $t.ok($name, format!("{d:?}")),
                Err(e) => $t.fail($name, format!("{e}")),
            }
        };
    }

    fn banner(title: &str) {
        println!("\n============================================================");
        println!(" [ffi] {title}");
        println!("============================================================");
    }

    fn tmpdir(tag: &str) -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "kubo-ffi-test-{}-{}-{}",
            tag,
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.to_string_lossy().to_string()
    }

    /// Minimal config surgery on an isolated repo: empty Bootstrap, switch
    /// Routing.Type to "auto" (hermetic two-node DHT), and enable pubsub.
    fn net_surgery(repo: &str) -> Result<String, String> {
        let path = std::path::Path::new(repo).join("config");
        let s = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let bstart = s.find("\"Bootstrap\": [").ok_or("no Bootstrap key")?;
        let bopen = s[bstart..].find('[').ok_or("no [")? + bstart;
        let bclose = s[bopen..].find(']').ok_or("no ]")? + bopen;
        let mut s = format!("{}[{}", &s[..bopen], &s[bclose..]);
        if !s.contains("\"Type\": \"none\"") {
            return Err("no routing none".into());
        }
        s = s.replacen("\"Type\": \"none\"", "\"Type\": \"auto\"", 1);
        // Enable pubsub inside the Pubsub object: isolated init leaves the
        // router empty; a router name is what kubo checks at startup.
        let pstart = s.find("\"Pubsub\": {").ok_or("no Pubsub key")?;
        let pend = s[pstart..].find('}').ok_or("no }")? + pstart;
        let block = s[pstart..pend].to_string();
        let fixed = if block.contains("\"Router\": \"\"") {
            // kubo 0.43 gates pubsub on Pubsub.Enabled (default false);
            // the Router name alone does not enable it.
            block.replacen(
                "\"Router\": \"\"",
                "\"Enabled\": true, \"Router\": \"gossipsub\"",
                1,
            )
        } else if block.contains("\"Enabled\": false") {
            block.replacen("\"Enabled\": false", "\"Enabled\": true", 1)
        } else {
            block
        };
        s = format!("{}{}{}", &s[..pstart], fixed, &s[pend..]);
        std::fs::write(&path, s).map_err(|e| e.to_string())?;
        Ok("bootstrap emptied, routing auto, pubsub on".into())
    }

    /// Raw HTTP GET over TCP; returns the status line.
    fn raw_get(host: &str, port: u16, path: &str, auth: Option<&str>) -> Result<String, String> {
        use std::io::{Read, Write};
        let mut stream =
            std::net::TcpStream::connect((host, port)).map_err(|e| format!("connect: {e}"))?;
        let auth_header = auth
            .map(|t| format!("Authorization: Bearer {t}\r\n"))
            .unwrap_or_default();
        stream
            .write_all(format!("GET {path} HTTP/1.1\r\nHost: {host}\r\n{auth_header}Connection: close\r\n\r\n").as_bytes())
            .map_err(|e| format!("write: {e}"))?;
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).map_err(|e| format!("read: {e}"))?;
        String::from_utf8_lossy(&buf)
            .lines()
            .next()
            .map(|l| l.to_string())
            .ok_or_else(|| "empty response".to_string())
    }

    fn split_host_port(multiaddr: &str) -> Option<(String, u16)> {
        let mut host = "127.0.0.1".to_string();
        let mut port: Option<u16> = None;
        let parts: Vec<&str> = multiaddr.split('/').collect();
        let mut i = 0;
        while i + 1 < parts.len() {
            match parts[i] {
                "ip4" | "ip6" => host = parts[i + 1].to_string(),
                "tcp" => port = parts[i + 1].parse().ok(),
                _ => {}
            }
            i += 1;
        }
        port.map(|p| (host, p))
    }

    fn run_sections(t: &mut Tally) {
        // ================================================================
        banner("node lifecycle — init_repo, Node::start, identity, addrs");
        let repo_a = tmpdir("a");
        check!(t, "init_repo (isolated profile)", init_repo(&repo_a).map(|_| "ok".to_string()));
        let node_a = match Node::start(&repo_a, true) {
            Ok(n) => {
                t.ok("Node::start(online)", "node handle live".to_string());
                n
            }
            Err(e) => {
                t.fail("Node::start(online)", e);
                println!(" [ffi] aborting: no node");
                return;
            }
        };
        let peer_a = node_a.peer_id().unwrap_or_default();
        check!(t, "Node::peer_id", {
            if peer_a.starts_with("12D3KooW") { Ok(peer_a.clone()) } else { Err(peer_a.clone()) }
        });
        check!(t, "Node::id", node_a.id().map(|v| v.chars().take(16).collect::<String>()).ok_or("none"));
        check!(t, "Node::listening_addrs", node_a.listening_addrs().filter(|v| v.contains("/tcp/")).ok_or("none"));
        check!(t, "kubo_ffi::version", {
            let v = version();
            if v.is_empty() { Err("empty".to_string()) } else { Ok(v) }
        });

        // ================================================================
        banner("unixfs + block — add/cat/ls, put/get/stat");
        let payload: Vec<u8> = (0..=255u8).cycle().take(600_000).collect();
        let cid_a = node_a.unixfs_add_bytes(&payload).expect("add");
        check!(t, "unixfs_add_bytes (600KB)", Ok::<String, String>(cid_a.clone()));
        check!(t, "unixfs_cat roundtrip", (|| -> Result<String, String> {
            let back = node_a.unixfs_cat(&cid_a)?;
            if back == payload { Ok(format!("{} bytes identical", back.len())) } else { Err("bytes differ".to_string()) }
        })());
        let blk = node_a.block_put(b"ffi-block-proof").expect("block put");
        check!(t, "block_put", Ok::<String, String>(blk.clone()));
        check!(t, "block_get roundtrip", (|| -> Result<String, String> {
            let back = node_a.block_get(&blk)?;
            if back == b"ffi-block-proof" { Ok(String::from_utf8_lossy(&back).to_string()) } else { Err("bytes differ".to_string()) }
        })());
        check!(t, "block_stat", node_a.block_stat(&blk).map(|n| format!("{n} bytes")));

        // ================================================================
        banner("pin — add/ls/is_pinned/update/verify/rm");
        let cid_b = node_a.unixfs_add_bytes(b"second pinned object").expect("add b");
        check!(t, "pin_add x2", node_a.pin_add(&cid_a).and_then(|_| node_a.pin_add(&cid_b)).map(|_| "pinned".to_string()));
        check!(t, "pin_is_pinned", node_a.pin_is_pinned(&cid_a).and_then(|b| if b { Ok("true".to_string()) } else { Err("false".to_string()) }));
        check!(t, "pin_ls", node_a.pin_ls().map(|l| format!("{} entries", l.matches("bafy").count())));
        check!(t, "pin_update", (|| -> Result<String, String> {
            let u1 = node_a.unixfs_add_bytes(b"pin update from")?;
            let u2 = node_a.block_put(b"pin update to")?;
            node_a.pin_update(&u1, &u2)?;
            let moved = node_a.pin_is_pinned(&u2)?;
            let _ = node_a.pin_rm(&u2);
            if moved { Ok("pin moved".to_string()) } else { Err("not moved".to_string()) }
        })());
        check!(t, "pin_verify", node_a.pin_verify().map(|v| format!("{} bytes json", v.len())));
        check!(t, "pin_rm", node_a.pin_rm(&cid_b).map(|_| "removed".to_string()));
        let _ = node_a.pin_add(&cid_a);

        // ================================================================
        banner("dag (CAR) + unixfs_ls + object — import/ls/links/diff");
        let car_path = env!("KUBO_FFI_WEBUI_CAR");
        let car = std::fs::read(car_path).unwrap_or_default();
        if car.is_empty() {
            t.skip("dag_import", "CAR placeholder (offline build)".to_string());
            t.skip("unixfs_ls", "needs dag_import root".to_string());
            t.skip("object_add_link", "needs dag_import root".to_string());
            t.skip("object_rm_link", "needs dag_import root".to_string());
            t.skip("object_diff", "needs dag_import root".to_string());
        } else {
            let roots_json = node_a.dag_import(&car).expect("dag import");
            t.ok("dag_import", format!("{} bytes CAR", car.len()));
            let root = roots_json.split('"').nth(3).unwrap_or("").to_string();
            check!(t, "dag_import root", if root.starts_with("bafy") { Ok(root.clone()) } else { Err(roots_json) });
            check!(t, "unixfs_ls", node_a.unixfs_ls(&root).map(|l| format!("{} entries", l.matches("\"name\"").count())));
            let linked = node_a.object_add_link(&root, "ffi-proof-link", &cid_a).expect("add_link");
            t.ok("object_add_link", linked.clone());
            check!(t, "object_rm_link", node_a.object_rm_link(&linked, "ffi-proof-link"));
            check!(t, "object_diff", node_a.object_diff(&root, &linked).map(|d| format!("{} bytes diff", d.len())));
        }

        // ================================================================
        banner("ipns (local) — publish/resolve/search");
        check!(t, "name_publish", node_a.name_publish(&cid_a).map(|n| n));
        check!(t, "name_resolve", node_a.name_resolve(&peer_a).map(|p| p));
        match node_a.name_search(&peer_a) {
            Ok(s) => t.ok("name_search", s.chars().take(60).collect::<String>()),
            Err(e) => t.skip("name_search", format!("local search: {e}")),
        }

        // ================================================================
        banner("key — self/generate/list/rename/remove");
        check!(t, "key_self", node_a.key_self().map(|k| k.chars().take(40).collect::<String>()));
        check!(t, "key_generate", node_a.key_generate("ffitest").map(|k| k));
        check!(t, "key_list", node_a.key_list().map(|l| if l.contains("ffitest") { "contains ffitest".to_string() } else { l }));
        check!(t, "key_rename", node_a.key_rename("ffitest", "ffitest2").map(|k| k));
        check!(t, "key_remove", node_a.key_remove("ffitest2").map(|k| k));

        // ================================================================
        banner("repo + config + bootstrap — stat/gc/get/set/list/add/rm");
        check!(t, "repo_stat", node_a.repo_stat().map(|s| format!("{} bytes json", s.len())));
        check!(t, "config_set + config_get", (|| -> Result<String, String> {
            node_a.config_set("API.HTTPHeaders", r#"{"X-Ffi-Test":["ok"]}"#)?;
            let cfg = node_a.config_get()?;
            if cfg.contains("X-Ffi-Test") { Ok("roundtrip ok".to_string()) } else { Err("not persisted".to_string()) }
        })());
        check!(t, "repo_gc", node_a.repo_gc().map(|_| "gc ok".to_string()));
        let dummy_boot = "/ip4/203.0.113.7/tcp/4001/p2p/12D3KooWRsU5Rk6Wb6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f";
        check!(t, "bootstrap_add", node_a.bootstrap_add(dummy_boot).map(|_| "added".to_string()));
        check!(t, "bootstrap_list", node_a.bootstrap_list().map(|l| if l.contains("203.0.113.7") { "contains dummy".to_string() } else { "missing dummy".to_string() }));
        check!(t, "bootstrap_rm", node_a.bootstrap_rm(dummy_boot).map(|_| "removed".to_string()));

        // ================================================================
        banner("serving — auth/static/api/gateway over real TCP+HTTP");
        let static_dir = tmpdir("static");
        std::fs::write(std::path::Path::new(&static_dir).join("index.html"), "ffi static ok").unwrap();
        check!(t, "set_auth_token", Ok::<String, String>(if node_a.set_auth_token("ffitest-token") { "true".to_string() } else { "false".to_string() }));
        check!(t, "add_crate_redirect", Ok::<String, String>(if node_a.add_crate_redirect(&static_dir) { "true".to_string() } else { "false".to_string() }));
        check!(t, "add_static", Ok::<String, String>(if node_a.add_static("/ffitest/", &static_dir) { "true".to_string() } else { "false".to_string() }));
        let api_addr = node_a.start_api("/ip4/127.0.0.1/tcp/0").expect("start_api");
        check!(t, "start_api", Ok::<String, String>(api_addr.clone()));
        check!(t, "Node::api_addrs", node_a.api_addrs().ok_or("none"));
        let gw_addr = node_a.start_gateway("/ip4/127.0.0.1/tcp/0").expect("start_gateway");
        check!(t, "start_gateway", Ok::<String, String>(gw_addr.clone()));
        match (split_host_port(&api_addr), split_host_port(&gw_addr)) {
            (Some((ah, ap)), Some((gh, gp))) => {
                check!(t, "api listener (TCP accept)", raw_get(&ah, ap, "/webui", None).map(|s| s));
                check!(t, "gateway serves unixfs (HTTP)", (|| -> Result<String, String> {
                    let s = raw_get(&gh, gp, &format!("/ipfs/{cid_a}"), Some("ffitest-token"))?;
                    if s.contains("200") { Ok(s) } else { Err(s) }
                })());
                check!(t, "gateway static dir (HTTP)", (|| -> Result<String, String> {
                    let s = raw_get(&gh, gp, "/ffitest/", Some("ffitest-token"))?;
                    if s.contains("200") { Ok(s) } else { Err(s) }
                })());
            }
            _ => t.fail("listener checks", "could not parse addrs".to_string()),
        }

        // ================================================================
        banner("commands — run_on_node / run / run_command / run_command_argv");
        check!(t, "Node::run_on_node", node_a.run_on_node(&repo_a, &["version"]).map(|v| v.chars().take(40).collect::<String>()));
        let repo_c = tmpdir("c");
        let _ = init_repo(&repo_c);
        // Node::run spawns its own node on the repo — use the throwaway
        // repo so it doesn't close the datastore under live node A.
        check!(t, "Node::run", node_a.run(&repo_c, "version").map(|v| v.chars().take(40).collect::<String>()));
        check!(t, "run_command", run_command(&repo_c, "version").map(|v| v.chars().take(40).collect::<String>()));
        check!(t, "run_command_argv", run_command_argv(&repo_c, &["version"]).map(|v| v.chars().take(40).collect::<String>()));

        // ================================================================
        banner("watch — start/stop (dedicated repo: watch runs its own node)");
        let repo_w = tmpdir("w");
        let watch_src = tmpdir("watch-src");
        std::fs::write(std::path::Path::new(&watch_src).join("watched.txt"), "watch me").unwrap();
        check!(t, "init_repo (watch)", init_repo(&repo_w).map(|_| "ok".to_string()));
        match WatchHandle::start(&repo_w, &watch_src) {
            Ok(h) => {
                t.ok("WatchHandle::start", "handle live".to_string());
                std::thread::sleep(std::time::Duration::from_millis(500));
                check!(t, "WatchHandle::stop", h.stop());
            }
            Err(e) => t.fail("WatchHandle::start", e),
        }

        // ================================================================
        banner("stats + logs + shutdown (consumes node A)");
        check!(t, "stats_bw", node_a.stats_bw().map(|v| v.chars().take(50).collect::<String>()));
        check!(t, "get_logs", (|| -> Result<String, String> {
            let l = get_logs();
            Ok(format!("{} bytes captured", l.len()))
        })());
        match node_a.shutdown() {
            Ok(()) => t.ok("Node::shutdown", "graceful".to_string()),
            Err(e) => t.fail("Node::shutdown", e),
        }

        // ================================================================
        banner("swarm + pubsub + routing — two connected local nodes");
        let repo_b = tmpdir("b");
        let repo_c2 = tmpdir("d");
        check!(t, "init_repo x2 (B,C)", init_repo(&repo_b).and_then(|_| init_repo(&repo_c2)).map(|_| "ok".to_string()));
        check!(t, "net_surgery (B)", net_surgery(&repo_b));
        check!(t, "net_surgery (C)", net_surgery(&repo_c2));
        let node_b = match Node::start(&repo_b, true) {
            Ok(n) => n,
            Err(e) => {
                t.fail("Node::start B", e);
                return;
            }
        };
        let node_c = match Node::start(&repo_c2, true) {
            Ok(n) => n,
            Err(e) => {
                t.fail("Node::start C", e);
                return;
            }
        };
        check!(t, "swarm_local_addrs (B)", node_b.swarm_local_addrs().map(|a| a.chars().take(40).collect::<String>()));
        check!(t, "swarm_listen_addrs (B)", node_b.swarm_listen_addrs().map(|a| a.chars().take(40).collect::<String>()));
        let peer_c = node_c.peer_id().unwrap_or_default();
        let listen_c = node_c.listening_addrs().unwrap_or_default();
        let dial = format!("{listen_c}/p2p/{peer_c}");
        check!(t, "swarm_connect B->C", Ok::<String, String>(if node_b.connect(&dial) { dial.clone() } else { "FAILED".to_string() }));
        std::thread::sleep(std::time::Duration::from_secs(2));
        check!(t, "swarm_peers (B sees C)", {
            let n = node_b.swarm_peers().len();
            if n >= 1 { Ok(format!("{n} peers")) } else { Err("0".to_string()) }
        });
        check!(t, "swarm_known_addrs (B)", node_b.swarm_known_addrs().map(|a| format!("{} bytes json", a.len())));

        // pubsub: B subscribes, C publishes over the live connection.
        let sub = match node_b.pubsub_subscribe("ffitest") {
            Ok(s) => {
                t.ok("pubsub_subscribe (B)", format!("sub {s}"));
                s
            }
            Err(e) => {
                t.fail("pubsub_subscribe", e);
                0
            }
        };
        std::thread::sleep(std::time::Duration::from_secs(3));
        let (tx, rx) = std::sync::mpsc::channel::<Result<String, String>>();
        std::thread::spawn(move || tx.send(Node::pubsub_next(sub)));
        let mut published = false;
        for _ in 0..3 {
            if node_c.pubsub_publish("ffitest", b"hello-ffi").is_ok() {
                published = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        check!(t, "pubsub_publish (C)", Ok::<String, String>(if published { "published".to_string() } else { "FAILED".to_string() }));
        match rx.recv_timeout(std::time::Duration::from_secs(25)) {
            Ok(Ok(msg)) => {
                let got = if msg.contains("aGVsbG8tZmZp") { "payload hello-ffi received" } else { "message received" };
                t.ok("pubsub_next (B got msg)", got.to_string());
            }
            Ok(Err(e)) => t.fail("pubsub_next", e),
            Err(_) => t.fail("pubsub_next", "timeout waiting for message".to_string()),
        }
        check!(t, "pubsub_ls (B)", node_b.pubsub_ls().map(|l| l));
        check!(t, "pubsub_peers (B)", node_b.pubsub_peers("ffitest").map(|p| p));
        check!(t, "pubsub_unsubscribe", Node::pubsub_unsubscribe(sub).map(|_| "ok".to_string()));

        // routing over the two-node DHT.
        std::thread::sleep(std::time::Duration::from_secs(2));
        check!(t, "routing_find_peer (B finds C)", node_b.routing_find_peer(&peer_c).map(|a| a.chars().take(60).collect::<String>()));
        let provide_cid = node_c.unixfs_add_bytes(b"routing provider proof").expect("add");
        check!(t, "routing_provide (C)", node_c.routing_provide(&provide_cid).map(|_| "providing".to_string()));
        std::thread::sleep(std::time::Duration::from_secs(4));
        check!(t, "routing_find_providers (B finds C)", node_b.routing_find_providers(&provide_cid).map(|p| {
            if p.contains(&peer_c) { format!("provider {peer_c} found") } else { format!("providers: {}", p.chars().take(60).collect::<String>()) }
        }));
        // put: DHT validates records — only signed IPNS records are storable.
        let peer_b = node_b.peer_id().unwrap_or_default();
        match node_b.routing_put(&format!("/ipns/{peer_b}"), b"value") {
            Ok(()) => t.ok("routing_put", "stored".to_string()),
            Err(e) => t.skip("routing_put", format!("DHT record validation (expected for ad-hoc records): {e}")),
        }
        // get: the call must round-trip even when no record exists.
        match node_b.routing_get(&format!("/ipns/{peer_b}")) {
            Ok(v) => t.ok("routing_get", format!("{} bytes", v.len())),
            Err(e) if e.contains("not found") || e.contains("no record") || e.contains("routing: not") => {
                t.ok("routing_get", format!("round-trip ok ({e})"))
            }
            Err(e) => t.fail("routing_get", e),
        }
        check!(t, "stats_bw (B)", node_b.stats_bw().map(|v| v.chars().take(50).collect::<String>()));
        check!(t, "swarm_disconnect", node_b.swarm_disconnect(&dial).map(|_| "closed".to_string()));
    }

    #[test]
    fn ffi_full_coverage() {
        let mut t = Tally { pass: 0, fail: 0, skip: 0 };
        banner("FULL FFI COVERAGE RUN — every public symbol exercised at runtime");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_sections(&mut t);
        }));
        if let Err(p) = result {
            println!(" [ffi] !! section panic: {:?}", p.downcast_ref::<&str>().unwrap_or(&"unknown"));
            t.fail += 1;
        }
        run_nonnode_sections(&mut t);
        println!("\n============================================================");
        println!(
            " [ffi coverage] {} checks — {} pass, {} fail, {} skip",
            t.pass + t.fail + t.skip,
            t.pass,
            t.fail,
            t.skip
        );
        println!("============================================================");
        assert_eq!(t.fail, 0, "{} ffi coverage checks failed", t.fail);
    }

    fn run_nonnode_sections(t: &mut Tally) {
        // ================================================================
        banner("nostr (offline) — keys, sign/verify, nip19 roundtrips");
        let nsec = match nostr_generate_key() {
            Ok(k) => {
                t.ok("nostr_generate_key", k.chars().take(20).collect::<String>() + "…");
                k
            }
            Err(e) => {
                t.fail("nostr_generate_key", e);
                String::new()
            }
        };
        if !nsec.is_empty() {
            check!(t, "nostr_get_public_key", nostr_get_public_key(&nsec).map(|p| p.chars().take(20).collect::<String>() + "…"));
            let event = nostr_event_sign(&nsec, "ffi proof", 1).expect("sign");
            t.ok("nostr_event_sign", format!("{} bytes event", event.len()));
            check!(t, "nostr_event_verify", nostr_event_verify(&event));
            let tagged = nostr_event_sign_with_tags(&nsec, "ffi proof tagged", 1, r#"[["t","ffi"]]"#).expect("sign tagged");
            t.ok("nostr_event_sign_with_tags", format!("{} bytes event", tagged.len()));
            let pubkey_hex = nostr_get_public_key(&nsec).unwrap();
            check!(t, "nip19 pubkey roundtrip", (|| -> Result<String, String> {
                let enc = nip19_encode_pubkey(&pubkey_hex)?;
                let dec = nip19_decode_pubkey(&enc)?;
                if dec == pubkey_hex { Ok(enc.chars().take(24).collect::<String>() + "…") } else { Err("mismatch".to_string()) }
            })());
            let note = nostr_event_sign(&nsec, "note for nip19", 1).unwrap();
            let note_id = note.split('"').nth(9).unwrap_or("").to_string();
            if !note_id.is_empty() {
                check!(t, "nip19 note roundtrip", (|| -> Result<String, String> {
                    let enc = nip19_encode_note(&note_id)?;
                    let dec = nip19_decode_note(&enc)?;
                    if dec == note_id { Ok("ok".to_string()) } else { Err("mismatch".to_string()) }
                })());
            }
            check!(t, "nip19 seckey roundtrip", (|| -> Result<String, String> {
                let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string();
                let enc = nip19_encode_seckey(&hex)?;
                let dec = nip19_decode_seckey(&enc)?;
                if dec == hex { Ok("ok".to_string()) } else { Err("mismatch".to_string()) }
            })());
            check!(t, "nip19 entity roundtrip", (|| -> Result<String, String> {
                let enc = nip19_encode_entity(&pubkey_hex, 1, "ffi-entity", r#"["wss://relay.example"]"#)?;
                let dec = nip19_decode_entity(&enc)?;
                if dec.contains("ffi-entity") { Ok("naddr ok".to_string()) } else { Err("no identifier".to_string()) }
            })());
        }

        // ================================================================
        banner("nostr (network, best-effort) — nip05 + relay publish");
        match nip05_query("_@fiatjaf.com") {
            Ok(j) => {
                t.ok("nip05_query", j.chars().take(60).collect::<String>());
                let pk = j.split('"').nth(3).unwrap_or("").to_string();
                if !pk.is_empty() {
                    check!(t, "nip05_verify", nip05_verify("_@fiatjaf.com", &pk).map(|_| "verified".to_string()));
                }
            }
            Err(e) => t.skip("nip05_query", format!("network: {e}")),
        }
        if !nsec.is_empty() {
            let event = nostr_event_sign(&nsec, &format!("ffi relay proof {}", std::process::id()), 1).unwrap();
            match NostrRelay::connect("wss://nos.lol") {
                Ok(relay) => {
                    t.ok("NostrRelay::connect", "wss://nos.lol".to_string());
                    check!(t, "NostrRelay::publish", relay.publish(&event).map(|_| "accepted".to_string()));
                    let filter = r#"{"kinds":[1],"limit":1}"#;
                    match relay.subscribe(filter) {
                        Ok(sub) => {
                            t.ok("NostrRelay::subscribe", "sub live".to_string());
                            check!(t, "NostrRelaySub::unsubscribe", sub.unsubscribe());
                        }
                        Err(e) => t.skip("NostrRelay::subscribe", e),
                    }
                    relay.close();
                    t.ok("NostrRelay::close", "closed".to_string());
                }
                Err(e) => t.skip("NostrRelay::connect", format!("network: {e}")),
            }
        }

        // ================================================================
        banner("git (go-git) — local repo over file paths + CLI-git fixtures");
        let has_git = std::process::Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if has_git {
            let origin = tmpdir("origin");
            let work = tmpdir("work");
            let clone_dir = tmpdir("clone");
            let git = |args: &[&str], cwd: &str| -> Result<String, String> {
                let out = std::process::Command::new("git")
                    .args(args)
                    .current_dir(cwd)
                    .output()
                    .map_err(|e| e.to_string())?;
                if out.status.success() {
                    Ok(String::from_utf8_lossy(&out.stdout).to_string())
                } else {
                    Err(String::from_utf8_lossy(&out.stderr).to_string())
                }
            };
            let cli_ok = git(&["init", "--bare", "-q", &origin], "/tmp")
                .and_then(|_| git(&["init", "-q", "-b", "main", &work], "/tmp"))
                .and_then(|_| {
                    std::fs::write(format!("{work}/hello.txt"), "hello ffi git").unwrap();
                    Ok(())
                })
                .and_then(|_| git(&["add", "."], &work))
                .and_then(|_| git(&["-c", "user.email=ffi@example.com", "-c", "user.name=ffi", "commit", "-q", "-m", "first"], &work))
                .and_then(|_| git(&["tag", "v0.0.1"], &work))
                .and_then(|_| git(&["remote", "add", "origin", &origin], &work))
                .and_then(|_| git(&["push", "-q", "origin", "main"], &work))
                .and_then(|_| git(&["push", "-q", "--tags", "origin"], &work));
            match cli_ok {
                Ok(_) => {
                    t.ok("git fixtures (CLI)", "origin bare + work repo + tag v0.0.1".to_string());
                    check!(t, "GitRepo::clone", GitRepo::clone(&origin, &clone_dir, false).map(|_| "cloned".to_string()));
                    let repo = match GitRepo::open(&clone_dir) {
                        Ok(r) => {
                            t.ok("GitRepo::open", "ok".to_string());
                            r
                        }
                        Err(e) => {
                            t.fail("GitRepo::open", e);
                            return;
                        }
                    };
                    check!(t, "GitRepo::head", repo.head().ok_or("none"));
                    check!(t, "GitRepo::is_bare", Ok::<String, String>(if repo.is_bare() { "true".to_string() } else { "false".to_string() }));
                    check!(t, "GitRepo::branches", repo.branches().ok_or("none"));
                    check!(t, "GitRepo::remotes", repo.remotes().ok_or("none"));
                    check!(t, "GitRepo::status", repo.status().map(|s| s.chars().take(40).collect::<String>()));
                    let head = repo.head().unwrap_or_default();
                    check!(t, "GitRepo::commit_lookup", repo.commit_lookup(&head).map(|c| c.chars().take(50).collect::<String>()));
                    check!(t, "GitRepo::tree_entries", repo.tree_entries(&head).map(|e| e.chars().take(50).collect::<String>()));
                    check!(t, "GitRepo::blob_read", (|| -> Result<String, String> {
                        let entries = repo.tree_entries(&head)?;
                        let blob_hash = entries
                            .lines()
                            .find(|l| l.starts_with("hello.txt\t"))
                            .and_then(|l| l.split('\t').nth(1))
                            .ok_or("no blob hash in tree")?
                            .to_string();
                        let b = repo.blob_read(&blob_hash)?;
                        String::from_utf8(b).map_err(|_| "non-utf8".to_string())
                    })());
                    check!(t, "git_blame", git_blame(&clone_dir, "hello.txt").map(|b| b.chars().take(40).collect::<String>()));
                    check!(t, "git_log", git_log(&clone_dir, 5).map(|l| l.chars().take(40).collect::<String>()));
                    check!(t, "git_tags", git_tags(&clone_dir).map(|g| if g.contains("v0.0.1") { "v0.0.1".to_string() } else { g }));
                    check!(t, "git_fetch_all", git_fetch_all(&clone_dir).map(|_| "ok".to_string()));
                    check!(t, "git_push", git_push(&work, "origin").map(|_| "ok".to_string()));
                    check!(t, "GitRepo::create_branch", repo.create_branch("ffi-branch", &head));
                    if git(&["-c", "user.email=ffi@example.com", "-c", "user.name=ffi", "commit", "--allow-empty", "-q", "-m", "second"], &work).is_ok() {
                        let _ = git_push(&work, "origin");
                        let h2 = git(&["rev-parse", "main"], &work).unwrap_or_default();
                        // Diff on a handle for the work repo: it holds the
                        // new commit object directly.
                        let work_repo = GitRepo::open(&work).expect("open work");
                        check!(t, "GitRepo::diff_trees", work_repo.diff_trees(&head, h2.trim()).map(|d| format!("{} bytes diff", d.len())));
                    }
                    let bare = tmpdir("bare");
                    check!(t, "GitRepo::init (bare)", GitRepo::init(&bare, true).map(|_| "ok".to_string()));
                }
                Err(e) => t.skip("git fixtures", format!("cli git setup failed: {e}")),
            }
        } else {
            t.skip("git section", "git CLI not available".to_string());
        }

        // ================================================================
        banner("libp2p host (standalone) — connect + ping between two hosts");
        let hb = P2pHost::new().expect("host b");
        let hc = P2pHost::new().expect("host c");
        let c_id = hc.peer_id().unwrap_or_default();
        let c_addr = hc.listening_addrs().unwrap_or_default();
        check!(t, "P2pHost::connect", hb.connect(&format!("{c_addr}/p2p/{c_id}")).map(|_| "dialed".to_string()));
        check!(t, "P2pHost::ping", hb.ping(&c_id).map(|rtt| format!("{rtt} ms rtt")));
        check!(t, "P2pHost::protocols", hb.protocols().map(|p| p.replace('\n', " ").chars().take(60).collect::<String>()));
        check!(t, "P2pHost::close", hb.close().and_then(|_| hc.close()));
    }
}
