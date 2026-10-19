# Kubo Xcode

A SwiftPM package (openable directly in Xcode) demonstrating the Kubo FFI
with a SwiftUI GUI, node lifecycle, peer networking, and an embedded WebUI.

## Open in Xcode (recommended for signing)

1. Open Xcode → **File → Open…** → select `ffi/xcode/Package.swift`
   (or open the `ffi/xcode` folder).
2. Xcode resolves the local `../swift` package automatically.
3. Select the `XcodeApp` target, set your **Signing & Capabilities** team
   in the target settings, and build/run (⌘R).

This gives you full access to Xcode's signing, entitlements, and debugger.

## Build from the command line

```sh
cd ffi/swift && make build          # build the Go FFI archive
cd ../xcode && make build          # swift build
make app                           # build a signed .app bundle
make run                           # build + open the .app
```

Or with xcodebuild directly:

```sh
cd ffi/swift && make build
cd ../xcode && xcodebuild -scheme KuboXcode -destination 'platform=macOS' build
```

## Entitlements

`Entitlements.plist` requests network client/server access and disables
the app sandbox. Adjust in Xcode's Signing & Capabilities tab as needed.

## Notes

- The archive `libkubo_ffi.a` is produced by `ffi/swift/Makefile` and is
  gitignored; rebuild it after changing Go FFI sources.
- The package depends on `../swift` (relative path), so keep the directory
  layout intact.
- The WebContent process (WKWebView) may show sandbox warnings on macOS 26
  with ad-hoc signing; open the logged WebUI URL in a browser if needed.
EOF
echo done