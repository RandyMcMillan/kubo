use kubo_ffi::{self, Node};
use std::thread;
use std::time::Duration;

/// WebUI CAR, cached by build.rs in OUT_DIR (see build.rs; fetch can be
/// skipped with KUBO_FFI_SKIP_WEBUI_ASSET=1, e.g. offline builds). Built
/// from the `webui` submodule by `assets/build-webui-car.sh` (recipe pinned
/// to upstream CI: node 24.11.0, `REACT_APP_GIT_REV=d87d88b`,
/// `ipfs add --cid-version 1 -r build`). When the built root matches Kubo's
/// hardcoded `core/corehttp.WebUIPath`, the API `/webui` redirect resolves.
const WEBUI_CAR: &[u8] = include_bytes!(env!("KUBO_FFI_WEBUI_CAR"));
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
    if WEBUI_CAR.is_empty() {
        eprintln!("WebUI CAR not available (offline or skipped build).");
        eprintln!("Fetch it once, then set KUBO_FFI_WEBUI_CAR to its path:");
        eprintln!("  curl -sSfL -o ipfs-webui-v4.13.0.car \\");
        eprintln!("    https://github.com/ipfs/ipfs-webui/releases/download/v4.13.0/ipfs-webui%40v4.13.0.car");
        eprintln!("  KUBO_FFI_WEBUI_CAR=$PWD/ipfs-webui-v4.13.0.car cargo run --example web");
        std::process::exit(1);
    }
    // Unique repo per run: a fixed path lets two concurrent runs delete
    // and re-init each other's repo mid-flight.
    let dir = std::env::temp_dir().join(format!(
        "kubo-rust-web-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let repo = dir.to_string_lossy().to_string();
    if let Err(e) = kubo_ffi::init_repo_networked(&repo) {
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

    // Permissive CORS so browser WebUIs can reach this ephemeral node:
    // any http(s) origin may call the API. Safe for a throwaway local
    // example node; do NOT copy onto a real daemon.
    if let Err(e) = node.config_set(
        "API.HTTPHeaders",
        r#"{"Access-Control-Allow-Origin":["*"],"Access-Control-Allow-Methods":["PUT","POST","GET"]}"#,
    ) {
        eprintln!("cors config failed: {}", e);
        std::process::exit(1);
    }

    // Bind the API to 127.0.0.1:5001 when free: that is the address the
    // WebUI connects to by default (kubo-rpc-client default), so the WebUI
    // works with zero configuration. Fall back to an ephemeral port and
    // print the custom API address the user must then set in the WebUI.
    let mut api = node.start_api("/ip4/127.0.0.1/tcp/5001");
    let mut api_is_default = true;
    if api.is_none() {
        kubo_ffi::last_error();
        api = node.start_api("/ip4/127.0.0.1/tcp/0");
        api_is_default = false;
    }
    let api = match api {
        Some(u) => u,
        None => {
            eprintln!("start_api failed: {:?}", kubo_ffi::last_error());
            std::process::exit(1);
        }
    };
    // Serve a one-shot setup shim from the gateway origin: it writes the
    // node's API multiaddr into the WebUI's localStorage ('ipfsApi' is the
    // only place the WebUI looks) and loads the WebUI. After one visit the
    // gateway-hosted WebUI connects with no manual configuration. The Kubo
    // RPC API itself cannot do this — localStorage is browser state.
    let shim_dir = std::env::temp_dir().join(format!("kubo-webui-shim-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&shim_dir);
    std::fs::create_dir_all(&shim_dir).expect("shim dir");
    let (ah, ap) = host_port(&api);
    let shim = format!(
        r#"<!doctype html><meta charset="utf-8"><title>webui setup</title>
<p>Configuring WebUI to talk to <code>/ip4/{ah}/tcp/{ap}/http</code> …</p>
<script>
localStorage.setItem('ipfsApi', '/ip4/{ah}/tcp/{ap}/http');
location.replace('/ipfs/{root}/');
</script>
"#,
        ah = ah,
        ap = ap,
        root = root
    );
    std::fs::write(shim_dir.join("index.html"), shim).expect("shim write");
    if !node.add_static("/setup/", &shim_dir.to_string_lossy()) {
        eprintln!("add_static(/setup) failed: {:?}", kubo_ffi::last_error());
        std::process::exit(1);
    }

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
    println!("WebUI (first time):     http://{}:{}/setup/", gh, gp);
    println!();
    println!("Open the /setup/ URL once per browser: it stores the node's API");
    println!("address in this browser's WebUI settings, then loads the WebUI.");
    if api_is_default {
        println!("(API is on the default :5001, so the API-redirect WebUI already");
        println!("connects without setup; /setup/ is for the gateway-hosted one.)");
    } else {
        println!("(port 5001 was busy, so the /setup/ step is REQUIRED for the");
        println!("gateway-hosted WebUI; the API-redirect WebUI needs no setup.)");
    }
    println!("Press Ctrl-C to stop.");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
