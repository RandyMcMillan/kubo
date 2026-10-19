# Kubo Xcode

A sample Xcode/SwiftPM project that consumes the `Kubo` Swift package
(`../swift`), which wraps the Kubo C FFI layer.

## Prerequisites

Build the FFI archive first:

```sh
cd ffi/swift && make build
```

## Build

```sh
cd ffi/xcode && swift build \
  -Xlinker -L../swift -Xlinker -lkubo_ffi \
  -Xlinker -framework -Xlinker Security \
  -Xlinker -framework -Xlinker CoreFoundation \
  -Xlinker -lresolv -Xlinker -lpthread -Xlinker -ldl
```

Or open `Package.swift` in Xcode and build the `XcodeApp` target.

## Notes

- The archive `libkubo_ffi.a` is produced by `ffi/swift/Makefile` and is
  gitignored; rebuild it after changing the Go FFI sources.
- The package depends on `../swift` (relative path), so keep the directory
  layout intact.
