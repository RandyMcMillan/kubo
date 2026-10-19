import CKubo
import Foundation

public enum KuboError: Error {
    case commandFailed(String)
}

public struct Kubo {
    public let repoPath: String

    public init(repoPath: String) {
        self.repoPath = repoPath
    }

    public static func version() -> String {
        guard let p = kubo_version() else { return "" }
        let s = String(cString: p)
        kubo_ffi_free_string(p)
        return s
    }

    public static func initRepo(_ repoPath: String) throws {
        let rc = repoPath.withCString { kubo_init_repo(UnsafeMutablePointer(mutating: $0)) }
        if rc != 0 {
            let msg: String
            if let p = kubo_ffi_last_error() {
                msg = String(cString: p)
                kubo_ffi_free_string(p)
            } else {
                msg = "init failed"
            }
            throw KuboError.commandFailed(msg)
        }
    }

    public func run(_ command: String) throws -> String {
        let result: UnsafeMutablePointer<CChar>? = repoPath.withCString { rp in
            command.withCString { cmd in
                kubo_run_command_line(UnsafeMutablePointer(mutating: rp), UnsafeMutablePointer(mutating: cmd))
            }
        }
        guard let p = result else {
            let msg: String
            if let e = kubo_ffi_last_error() {
                msg = String(cString: e)
                kubo_ffi_free_string(e)
            } else {
                msg = "command failed"
            }
            throw KuboError.commandFailed(msg)
        }
        let s = String(cString: p)
        kubo_ffi_free_string(p)
        return s
    }

    // MARK: - Node lifecycle (typed)
    public class Node {
        private let handle: UInt64
        public let repoPath: String

        public init(repoPath: String, online: Bool) throws {
            self.repoPath = repoPath
            let h = repoPath.withCString { rp in
                kubo_node_start(UnsafeMutablePointer(mutating: rp), online ? 1 : 0)
            }
            if h == 0 {
                let msg = Kubo.lastError() ?? "node start failed"
                throw KuboError.commandFailed(msg)
            }
            self.handle = h
        }

        public var peerId: String? {
            guard let p = kubo_node_peer_id(handle) else { return nil }
            let s = String(cString: p)
            kubo_ffi_free_string(p)
            return s
        }

        public var listeningAddrs: String? {
            guard let p = kubo_node_listening_addrs(handle) else { return nil }
            let s = String(cString: p)
            kubo_ffi_free_string(p)
            return s
        }

        public func stop() {
            kubo_node_stop(handle)
        }

        deinit { kubo_node_stop(handle) }
    }

    public static func lastError() -> String? {
        guard let p = kubo_ffi_last_error() else { return nil }
        let s = String(cString: p)
        kubo_ffi_free_string(p)
        return s
    }
}