fn main() {
    // Build Swift package and link the static library
    swift_rs::SwiftLinker::new("14")
        .with_package("swift-lib", "./swift-lib/")
        .link();

    tauri_build::build()
}
