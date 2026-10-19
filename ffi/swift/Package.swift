// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "KuboFFI",
    products: [
        .library(name: "Kubo", targets: ["Kubo"]),
    ],
    targets: [
        .systemLibrary(
            name: "CKubo",
            path: "Sources/CKubo",
            pkgConfig: nil,
            providers: nil
        ),
        .target(
            name: "Kubo",
            dependencies: ["CKubo"],
            path: "Sources/Kubo"
        ),
        .testTarget(
            name: "KuboTests",
            dependencies: ["Kubo"],
            path: "Tests/KuboTests"
        ),
    ]
)
