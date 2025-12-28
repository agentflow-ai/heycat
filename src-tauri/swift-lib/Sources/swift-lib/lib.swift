import SwiftRs

/// A simple test function to verify Swift-Rust interop is working.
/// Returns "Hello from Swift!" as a SRString (Swift-Rs String type).
@_cdecl("swift_hello")
public func swiftHello() -> SRString {
    SRString("Hello from Swift!")
}
