import Foundation

@_silgen_name("shared_greeting")
private func sharedGreeting(_ name: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

@_silgen_name("shared_lattice_score")
private func sharedLatticeScore(_ seed: Int32) -> Int32

@_silgen_name("shared_string_free")
private func sharedStringFree(_ value: UnsafeMutablePointer<CChar>?)

struct RustBridge {
    func greeting(name: String) -> String {
        name.withCString { pointer in
            guard let raw = sharedGreeting(pointer) else {
                return ""
            }
            defer { sharedStringFree(raw) }
            return String(cString: raw)
        }
    }

    func latticeScore(seed: Int32) -> Int32 {
        sharedLatticeScore(seed)
    }
}
