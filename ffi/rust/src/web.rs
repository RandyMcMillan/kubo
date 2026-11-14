//! Self-contained IPFS WebUI server (enable with `--features web`).
//!
//! [`serve_blocking`] does everything the `web` example does: a networked
//! node on a throwaway repo, the embedded WebUI CAR imported and pinned,
//! permissive CORS for browser WebUIs, the API on 127.0.0.1:5001 (falling
//! back to an ephemeral port) plus a gateway, and a one-shot `/setup/` shim
//! that stores the API multiaddr in the browser's WebUI settings.
//!
//! The CAR is embedded from the path in `KUBO_FFI_WEBUI_CAR` (cached by
//! build.rs); when it is a placeholder (offline/skip builds) [`WebNode::start`]
//! returns an error explaining how to supply the file at runtime.

use crate::{self as kubo_ffi, init_repo_networked, last_error, Node};
use std::time::Duration;

/// Kubo 0.43's hardcoded WebUI root (v4.13.0).
pub const KUBO_WEBUI_ROOT: &str =
    "bafybeiciqeyipumpmhxzlxnbqdbbv6u5uij4hy4wax64dmj7kvrhusiq6y";

/// WebUI CAR (ipfs-webui v4.13.0, root matches `KUBO_WEBUI_ROOT`).
/// Fetched and verified by build.rs; a 0-byte placeholder when the fetch
/// was skipped (offline, `KUBO_FFI_SKIP_WEBUI_ASSET=1`, or docs.rs).
pub const WEBUI_CAR: &[u8] = include_bytes!(env!("KUBO_FFI_WEBUI_CAR"));

/// URLs a [`WebNode`] serves.
pub struct WebUrls {
    /// RPC API listener (`/api/v0`, WebUI redirect at `/webui`).
    pub api: String,
    /// Read-only gateway (`/ipfs`, `/ipns`, static dirs).
    pub gateway: String,
    /// One-shot shim: stores the API multiaddr in the browser's WebUI
    /// settings, then loads the WebUI.
    pub setup: String,
    /// `Some` when the imported root matches kubo's hardcoded WebUIPath,
    /// meaning the API `/webui` redirect resolves.
    pub webui_api: Option<String>,
    /// WebUI served from the gateway (works after one visit to `setup`).
    pub webui_gateway: String,
    /// Root CID the WebUI was imported under.
    pub root: String,
    /// API multiaddr multiaddr-style (`/ip4/127.0.0.1/tcp/<port>/http`).
    pub api_multiaddr: String,
}

/// A running self-contained WebUI server.
pub struct WebNode {
    node: Node,
    /// Repo path of the throwaway networked repo.
    pub repo: String,
    /// Served URLs.
    pub urls: WebUrls,
    api_is_default: bool,
}

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

impl WebNode {
    /// Start the full stack. Blocks on nothing — use [`WebNode::wait`] or
    /// [`serve_blocking`] to keep the process alive.
    pub fn start() -> Result<Self, String> {
        if WEBUI_CAR.is_empty() {
            return Err(
                "WebUI CAR not available (offline or skipped build).\n\
                 Fetch it once, then set KUBO_FFI_WEBUI_CAR to its path:\n  \
                 curl -sSfL -o ipfs-webui-v4.13.0.car \
                 https://github.com/ipfs/ipfs-webui/releases/download/v4.13.0/ipfs-webui%40v4.13.0.car\n  \
                 KUBO_FFI_WEBUI_CAR=$PWD/ipfs-webui-v4.13.0.car cargo run --features web --example web"
                    .to_string(),
            );
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
        let repo = dir.to_string_lossy().to_string();
        init_repo_networked(&repo)?;

        let node = Node::start(&repo, true)?;

        // Import the embedded WebUI CAR (roots are pinned by dag_import).
        let roots_json = node.dag_import(WEBUI_CAR)?;
        let root = roots_json
            .split('"')
            .nth(3)
            .unwrap_or("")
            .to_string();
        if root.is_empty() {
            return Err(format!("could not parse root from dag import response: {roots_json}"));
        }
        if !node.pin_is_pinned(&root).unwrap_or(false) {
            return Err(format!("webui root {root} not pinned after import"));
        }

        // Permissive CORS so browser WebUIs can reach this ephemeral node.
        node.config_set(
            "API.HTTPHeaders",
            r#"{"Access-Control-Allow-Origin":["*"],"Access-Control-Allow-Methods":["PUT","POST","GET"]}"#,
        )?;

        // Bind the API to 127.0.0.1:5001 when free — the WebUI connects to
        // that default with zero configuration. Fall back to an ephemeral
        // port; the /setup/ shim then carries the real multiaddr.
        let mut api = node.start_api("/ip4/127.0.0.1/tcp/5001");
        let mut api_is_default = true;
        if api.is_none() {
            last_error();
            api = node.start_api("/ip4/127.0.0.1/tcp/0");
            api_is_default = false;
        }
        let api_addr = api.ok_or_else(|| format!("start_api failed: {:?}", last_error()))?;

        // One-shot setup shim mounted on the gateway: it writes the node's
        // API multiaddr into the WebUI's localStorage (the Kubo RPC API
        // itself cannot — localStorage is browser state) and loads the UI.
        let (ah, ap) = host_port(&api_addr);
        let shim_dir = std::env::temp_dir().join(format!("kubo-webui-shim-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&shim_dir);
        std::fs::create_dir_all(&shim_dir).map_err(|e| e.to_string())?;
        let shim = format!(
            r#"<!doctype html><meta charset="utf-8"><title>webui setup</title>
<p>Configuring WebUI to talk to <code>/ip4/{ah}/tcp/{ap}/http</code> …</p>
<script>
localStorage.setItem('ipfsApi', '/ip4/{ah}/tcp/{ap}/http');
location.replace('/ipfs/{root}/');
</script>
"#
        );
        std::fs::write(shim_dir.join("index.html"), shim).map_err(|e| e.to_string())?;
        if !node.add_static("/setup/", &shim_dir.to_string_lossy()) {
            return Err(format!("add_static(/setup/) failed: {:?}", last_error()));
        }

        let gw_addr = node
            .start_gateway("/ip4/127.0.0.1/tcp/0")
            .ok_or_else(|| format!("start_gateway failed: {:?}", last_error()))?;
        let (gh, gp) = host_port(&gw_addr);

        let webui_api = (root == KUBO_WEBUI_ROOT)
            .then(|| format!("http://{ah}:{ap}/webui"));
        let urls = WebUrls {
            api: format!("http://{ah}:{ap}"),
            gateway: format!("http://{gh}:{gp}"),
            setup: format!("http://{gh}:{gp}/setup/"),
            webui_api,
            webui_gateway: format!("http://{gh}:{gp}/ipfs/{root}/"),
            root,
            api_multiaddr: format!("/ip4/{ah}/tcp/{ap}/http"),
        };

        Ok(WebNode {
            node,
            repo,
            urls,
            api_is_default,
        })
    }

    /// Access the underlying node (pin, add, swarm, …).
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Print the served URLs and setup guidance (for binaries).
    pub fn print_summary(&self) {
        println!();
        if let Some(u) = &self.urls.webui_api {
            println!("WebUI (API redirect):  {u}");
        }
        println!("WebUI (gateway direct): {}", self.urls.webui_gateway);
        println!("WebUI (first time):     {}", self.urls.setup);
        println!();
        println!("Open the /setup/ URL once per browser: it stores the node's API");
        println!("address in this browser's WebUI settings, then loads the WebUI.");
        if self.api_is_default {
            println!("(API is on the default :5001, so the API-redirect WebUI already");
            println!("connects without setup; /setup/ is for the gateway-hosted one.)");
        } else {
            println!("(port 5001 was busy, so the /setup/ step is REQUIRED for the");
            println!("gateway-hosted WebUI; the API-redirect WebUI needs no setup.)");
        }
    }

    /// Block forever (Ctrl-C to exit). Takes `&self` so callers keep the
    /// node alive for the process lifetime.
    pub fn wait(&self) -> ! {
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

/// Start a [`WebNode`], print its summary, and block forever.
pub fn serve_blocking() -> Result<(), String> {
    let web = WebNode::start()?;
    println!("Kubo version: {}", kubo_ffi::version());
    println!("peer id: {:?}", web.node().peer_id());
    web.print_summary();
    web.wait();
}
