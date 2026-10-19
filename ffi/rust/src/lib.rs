use std::ffi::{c_char, CStr, CString};
#[link(name = "kubo_ffi", kind = "static")]
extern "C" {
    fn kubo_version() -> *mut c_char;
    fn kubo_ffi_last_error() -> *mut c_char;
    fn kubo_ffi_free_string(s: *mut c_char);
    fn kubo_ffi_free_buffer(buf: *mut u8);
    fn kubo_init_repo(repo_path: *const c_char) -> i64;
    fn kubo_node_start(repo_path: *const c_char, online: u8) -> u64;
    fn kubo_node_stop(handle: u64) -> i64;
    fn kubo_node_peer_id(handle: u64) -> *mut c_char;
    fn kubo_run_command_line(repo_path: *const c_char, line: *const c_char) -> *mut c_char;
}

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

/// Returns the last error message, if any.
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

/// Initialize a repository at `repo_path`.
pub fn init_repo(repo_path: &str) -> Result<(), String> {
    let c = CString::new(repo_path).map_err(|e| e.to_string())?;
    let rc = unsafe { kubo_init_repo(c.as_ptr()) };
    if rc != 0 {
        Err(last_error().unwrap_or_else(|| "init failed".into()))
    } else {
        Ok(())
    }
}

/// A running Kubo node handle.
pub struct Node(u64);

impl Node {
    pub fn start(repo_path: &str, online: bool) -> Result<Self, String> {
        let c = CString::new(repo_path).map_err(|e| e.to_string())?;
        let h = unsafe { kubo_node_start(c.as_ptr(), if online { 1 } else { 0 }) };
        if h == 0 {
            Err(last_error().unwrap_or_else(|| "node start failed".into()))
        } else {
            Ok(Node(h))
        }
    }

    pub fn peer_id(&self) -> Option<String> {
        unsafe {
            let p = kubo_node_peer_id(self.0);
            if p.is_null() {
                return None;
            }
            let s = CStr::from_ptr(p).to_string_lossy().to_string();
            kubo_ffi_free_string(p);
            Some(s)
        }
    }

    pub fn run(&self, command: &str) -> Result<String, String> {
        let rp = CString::new("").unwrap();
        let line = CString::new(command).map_err(|e| e.to_string())?;
        let p = unsafe { kubo_run_command_line(rp.as_ptr(), line.as_ptr()) };
        if p.is_null() {
            return Err(last_error().unwrap_or_else(|| "command failed".into()));
        }
        let s = unsafe { CStr::from_ptr(p).to_string_lossy().to_string() };
        unsafe { kubo_ffi_free_string(p) };
        Ok(s)
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe { kubo_node_stop(self.0); }
    }
}

/// Run an arbitrary Kubo command (full coverage).
pub fn run_command(repo_path: &str, command: &str) -> Result<String, String> {
    let rp = CString::new(repo_path).map_err(|e| e.to_string())?;
    let line = CString::new(command).map_err(|e| e.to_string())?;
    let p = unsafe { kubo_run_command_line(rp.as_ptr(), line.as_ptr()) };
    if p.is_null() {
        return Err(last_error().unwrap_or_else(|| "command failed".into()));
    }
    let s = unsafe { CStr::from_ptr(p).to_string_lossy().to_string() };
    unsafe { kubo_ffi_free_string(p) };
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_works() {
        assert!(!version().is_empty());
    }
}