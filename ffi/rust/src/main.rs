//! kubo-ffi binary: two entry points selected by the `web` feature.
//!
//! With `--features web`: starts the self-contained WebUI server
//! (see [`kubo_ffi::web`]) and blocks forever.
//!
//! Without it: runs a small FFI demo (version, throwaway node, identity)
//! so the binary is useful out of the box.

#[cfg(feature = "web")]
fn main() {
    if let Err(e) = kubo_ffi::web::serve_blocking() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "web"))]
fn main() {
    use kubo_ffi::{init_repo, Node};

    println!("kubo-ffi demo (build with --features web for the WebUI server)");
    println!("Kubo version: {}", kubo_ffi::version());

    let dir = std::env::temp_dir().join(format!(
        "kubo-ffi-main-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));
    let repo = dir.to_string_lossy().to_string();

    if let Err(e) = init_repo(&repo) {
        eprintln!("init failed: {e}");
        std::process::exit(1);
    }
    match Node::start(&repo, true) {
        Ok(node) => {
            println!("peer id: {:?}", node.peer_id());
            println!("listening: {:?}", node.listening_addrs());
        }
        Err(e) => {
            eprintln!("node start failed: {e}");
            std::process::exit(1);
        }
    }
}
