// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "swift-lib",
    products: [
        .library(
            name: "swift-lib",
            type: .static,
            targets: ["swift-lib"]
        ),
    ],
    dependencies: [
        .package(url: "https://github.com/Brendonovich/swift-rs", from: "1.0.5"),
    ],
    targets: [
        .target(
            name: "swift-lib",
            dependencies: [
                .product(name: "SwiftRs", package: "swift-rs"),
            ],
            path: "Sources/swift-lib"
        ),
    ]
)
