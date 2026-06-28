import Foundation
import SharedRust

struct RustBridge {
    func greeting(name: String) -> String {
        name.withCString { pointer in
            guard let raw = shared_greeting(pointer) else {
                return ""
            }
            defer { shared_string_free(raw) }
            return String(cString: raw)
        }
    }

    func latticeScore(seed: Int32) -> Int32 {
        shared_lattice_score(seed)
    }
}

