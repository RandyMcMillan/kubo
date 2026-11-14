use kubo_ffi::{self, Node};
use std::thread;
use std::time::Duration;

/// WebUI CAR built from the `webui` submodule by
/// `assets/build-webui-car.sh` (recipe pinned to upstream CI: node 24.11.0,
/// `REACT_APP_GIT_REV=c51d9809`, `ipfs add --cid-version 1 -r build`).
/// When the built root matches Kubo's hardcoded `core/corehttp.WebUIPath`,
/// the API `/webui` redirect resolves to it.
const WEBUI_CAR: &[u8] = include_bytes!("../assets/ipfs-webui-v4.13.0.car");
/// Kubo 0.43's hardcoded WebUI root (v4.13.0).
const KUBO_WEBUI_ROOT: &str = "bafybeiciqeyipumpmhxzlxnbqdbbv6u5uij4hy4wax64dmj7kvrhusiq6y";

fn host_port(multiaddr: &str) -> (String, String) {
    let mut host = "127.0.0.1".to_string();
    let mut port = "0".to_string();
    let parts: Vec<&str> = multiaddr.split('/').collect();
    let mut i = 0;
    while i + 1 < parts.len() {
        match parts[i] {
            "ip4" | "ip6" | "dns4" | "dns6" | "dns" => host = parts[i + 1].to_string(),
            "tcp" => port = parts[i + 1].to_string(),
            _ => {}
        }
        i += 1;
    }
    (host, port)
}

fn main() {
    println!("Kubo version: {}", kubo_ffi::version());
    let dir = std::env::temp_dir().join("kubo-rust-web");
    let _ = std::fs::remove_dir_all(&dir);
    let repo = dir.to_string_lossy().to_string();
    if let Err(e) = kubo_ffi::init_repo(&repo) {
        eprintln!("init failed: {}", e);
        std::process::exit(1);
    }

    let node = match Node::start(&repo, true) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("node start failed: {}", e);
            std::process::exit(1);
        }
    };
    println!("peer id: {:?}", node.peer_id());

    // Import the embedded WebUI CAR. Fully offline: no bootstrap fetch.
    let root = match node.dag_import(WEBUI_CAR) {
        Ok(roots_json) => {
            // dag_import returns {"roots":["<cid>",...]}; take the first root.
            let root = roots_json
                .split('"')
                .nth(3)
                .unwrap_or("")
                .to_string();
            if root.is_empty() {
                eprintln!("could not parse root from dag import response: {}", roots_json);
                std::process::exit(1);
            }
            println!("dag import roots: {}", roots_json);
            root
        }
        Err(e) => {
            eprintln!("webui import failed: {}", e);
            std::process::exit(1);
        }
    };
    match node.pin_is_pinned(&root) {
        Ok(true) => println!("webui pinned: {}", root),
        Ok(false) => {
            eprintln!("webui root {} not pinned after import", root);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("pin check failed: {}", e);
            std::process::exit(1);
        }
    }
    if root == KUBO_WEBUI_ROOT {
        println!("root matches kubo WebUIPath: API /webui redirect will resolve");
    } else {
        println!("note: root != kubo WebUIPath ({}); use the gateway URL below", KUBO_WEBUI_ROOT);
    }
    if let Ok(entries) = node.unixfs_ls(&root) {
        println!("webui entries: {}", entries.chars().take(200).collect::<String>());
    }

    let api = match node.start_api("/ip4/127.0.0.1/tcp/0") {
        Some(u) => u,
        None => {
            eprintln!("start_api failed: {:?}", kubo_ffi::last_error());
            std::process::exit(1);
        }
    };
    let gw = match node.start_gateway("/ip4/127.0.0.1/tcp/0") {
        Some(u) => u,
        None => {
            eprintln!("start_gateway failed: {:?}", kubo_ffi::last_error());
            std::process::exit(1);
        }
    };

    let (ah, ap) = host_port(&api);
    let (gh, gp) = host_port(&gw);
    println!();
    if root == KUBO_WEBUI_ROOT {
        println!("WebUI (API redirect):  http://{}:{}/webui", ah, ap);
    }
    println!("WebUI (gateway direct): http://{}:{}/ipfs/{}/", gh, gp, root);
    println!("Press Ctrl-C to stop.");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
