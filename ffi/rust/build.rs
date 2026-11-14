use std::env;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    // Prefer the canonical module at ../ (the full kubo checkout; its
    // go.mod replaces github.com/ipfs/kubo with the repo root). The
    // in-crate go/ copy ships in the published tarball for consumers.
    let ffi_dir = if manifest_dir.join("../go.mod").exists() {
        manifest_dir.join("..")
    } else {
        manifest_dir.join("go")
    };
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let target = env::var("TARGET").expect("TARGET not set");
    let host = env::var("HOST").expect("HOST not set");

    // Cross-compilation: allow same-family builds, bail out early for
    // unsupported host/target pairs with a clear message.
    let host_parts: Vec<&str> = host.split('-').collect();
    let target_parts: Vec<&str> = target.split('-').collect();
    let host_os = host_parts.get(2).copied().unwrap_or("unknown");
    let target_os = target_parts.get(2).copied().unwrap_or("unknown");
    let host_vendor = host_parts.get(1).copied().unwrap_or("unknown");
    let target_vendor = target_parts.get(1).copied().unwrap_or("unknown");

    let both_darwin = host_vendor == "apple" && target_vendor == "apple";
    let both_linux = host_os == "linux" && target_os == "linux";
    let both_windows = host_os == "windows" && target_os == "windows";

    if target != host && !both_darwin && !both_linux && !both_windows {
        panic!(
            "kubo-ffi build script does not support cross-compilation \
             for this target/host pair. TARGET ({}) != HOST ({}). \
             To cross-compile, set up a C cross-toolchain and \
             manually compile go/ffi/ with CGO_ENABLED=1.",
            target, host
        );
    }

    // Find Go binary.
    let go = env::var("GO").unwrap_or_else(|_| "go".to_string());
    match Command::new(&go).args(["version"]).output() {
        Ok(output) if output.status.success() => {}
        Ok(_) => panic!("`go version` failed. Make sure Go is installed and in PATH."),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            panic!(
                "Go toolchain not found: set GO or add `go` to PATH. \
                 Kubo requires Go >= 1.21."
            );
        }
        Err(err) => panic!("failed to run `go version` with `{go}`: {err}"),
    }

    // Pin GOTOOLCHAIN to the Go version declared in go.mod so that the
    // build is reproducible even when the host has a newer Go installed.
    let go_mod_text =
        std::fs::read_to_string(ffi_dir.join("go.mod")).expect("failed to read go.mod");
    let toolchain = go_mod_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("go ")
                .map(|ver| format!("go{}", ver.trim()))
        })
        .expect("no 'go' directive found in go.mod");

    // Map Rust target to Go os/arch.
    let (goos, goarch) = parse_target(&target);

    // Go's c-archive naming depends on the platform toolchain.
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let env_kind = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let (archive_name, _header_name) = if os == "windows" && env_kind == "msvc" {
        ("kubo_ffi.lib", "kubo_ffi.h")
    } else {
        ("libkubo_ffi.a", "kubo_ffi.h")
    };
    let archive = out_dir.join(archive_name);

    // Rebuild when any Go source in the module changes (directory watch
    // covers every file, present and future).
    println!(
        "cargo:rerun-if-changed={}",
        ffi_dir.display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        ffi_dir.join("go.mod").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        ffi_dir.join("go.sum").display()
    );

    let mut go_build = Command::new(&go);
    go_build
        .current_dir(&ffi_dir)
        .env("CGO_ENABLED", "1")
        .env("GOOS", goos)
        .env("GOARCH", goarch)
        .env("GOTOOLCHAIN", &toolchain)
        .args([
            "build",
            "-buildmode=c-archive",
            "-o",
            archive.to_str().unwrap(),
            ".",
        ]);

    // Prevent Apple deployment-target env vars from leaking into the Go build.
    go_build.env_remove("IPHONEOS_DEPLOYMENT_TARGET");
    go_build.env_remove("MACOSX_DEPLOYMENT_TARGET");

    // When cross-compiling for Apple platforms, the Go toolchain needs an
    // Apple-specific C cross-compiler.
    if let Some((sdk, cc, cxx)) = apple_compiler(&target) {
        go_build.env("CC", &cc).env("CXX", &cxx).env("SDKROOT", sdk);

        let deployment_target = env::var("IPHONEOS_DEPLOYMENT_TARGET")
            .or_else(|_| env::var("MACOSX_DEPLOYMENT_TARGET"))
            .unwrap_or_else(|_| "14.0".to_string());
        go_build.env("IPHONEOS_DEPLOYMENT_TARGET", &deployment_target);
    }

    let status = go_build.status().expect("failed to run go");
    assert!(status.success(), "go build of ffi failed");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=kubo_ffi");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=resolv");
    }
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dylib=dl");

    // The WebUI CAR (29MB) is gitignored; examples that embed it get it
    // fetched once from the canonical release asset and sha256-verified.
    // Set KUBO_FFI_SKIP_WEBUI_ASSET=1 to skip (offline builds, CI that
    // never builds the web example).
    ensure_webui_asset(&manifest_dir);
}

const WEBUI_CAR_URL: &str =
    "https://github.com/ipfs/ipfs-webui/releases/download/v4.13.0/ipfs-webui%40v4.13.0.car";
const WEBUI_CAR_SHA256: &str =
    "0f26ce282643f26fc699451f3b71bcac6bacdd96c052612406337d69e8a5e55b";
const WEBUI_CAR_NAME: &str = "ipfs-webui-v4.13.0.car";

fn ensure_webui_asset(manifest_dir: &std::path::Path) {
    if env::var("KUBO_FFI_SKIP_WEBUI_ASSET").is_ok() {
        return;
    }
    let path = manifest_dir.join("assets").join(WEBUI_CAR_NAME);
    println!("cargo:rerun-if-changed={}", path.display());
    if path.exists() {
        return;
    }
    eprintln!("kubo-ffi: fetching WebUI CAR from {WEBUI_CAR_URL}");
    let tmp = path.with_extension("car.download");
    let status = Command::new("curl")
        .args(["-sSfL", "--retry", "3", WEBUI_CAR_URL, "-o"])
        .arg(&tmp)
        .status()
        .expect("failed to run curl; install it or set KUBO_FFI_SKIP_WEBUI_ASSET=1");
    assert!(status.success(), "curl download of WebUI CAR failed");
    let digest = sha256_hex(&std::fs::read(&tmp).expect("read downloaded CAR"));
    assert_eq!(
        digest, WEBUI_CAR_SHA256,
        "WebUI CAR sha256 mismatch: got {digest}, want {WEBUI_CAR_SHA256}"
    );
    std::fs::rename(&tmp, &path).expect("move CAR into place");
    eprintln!("kubo-ffi: WebUI CAR verified and placed at {}", path.display());
}

/// Minimal SHA-256 (FIPS 180-4) so the build script needs no dependencies.
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in w[..16].iter_mut().enumerate() {
            *word = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

fn parse_target(target: &str) -> (&str, &str) {
    let parts: Vec<&str> = target.split('-').collect();
    let arch = parts[0];

    let os = if parts.len() >= 3 && parts[1] == "apple" && parts[2].starts_with("ios") {
        "ios"
    } else {
        parts.get(2).copied().unwrap_or("unknown")
    };

    let goarch = match arch {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "i686" => "386",
        "arm" => "arm",
        "armv7" => "arm",
        "riscv64" => "riscv64",
        "wasm32" => panic!("wasm32 is not supported by CGO"),
        _ => arch,
    };

    (os, goarch)
}

fn apple_compiler(target: &str) -> Option<(String, String, String)> {
    if std::env::consts::OS != "macos" {
        return None;
    }

    let parts: Vec<&str> = target.split('-').collect();
    if parts.len() < 3 || parts[1] != "apple" {
        return None;
    }

    let arch = parts[0];
    let os = parts[2];
    let variant = parts.get(3).copied();

    let (sdk, target_flag): (&str, &str) = match (arch, os, variant) {
        ("aarch64", "ios", None) => ("iphoneos", "-target arm64-apple-ios"),
        ("x86_64", "ios", None) => ("iphonesimulator", "-target x86_64-apple-ios-simulator"),
        ("aarch64", "ios", Some("sim")) => ("iphonesimulator", "-target arm64-apple-ios-simulator"),
        ("aarch64", "ios", Some("macabi")) => ("macosx", "-target arm64-apple-ios-macabi"),
        ("x86_64", "ios", Some("macabi")) => ("macosx", "-target x86_64-apple-ios-macabi"),
        (_, "darwin", _) => return None,
        _ => return None,
    };

    let cc = format!("xcrun -sdk {} clang {}", sdk, target_flag)
        .trim()
        .to_string();
    let cxx = format!("xcrun -sdk {} clang++ {}", sdk, target_flag)
        .trim()
        .to_string();

    Some((sdk.to_string(), cc, cxx))
}
