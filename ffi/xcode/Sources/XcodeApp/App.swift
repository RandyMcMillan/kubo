import SwiftUI
import Kubo

@main
struct KuboXcodeApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    @State private var version: String = Kubo.version()
    @State private var repoPath: String = FileManager.default.temporaryDirectory.appendingPathComponent("kubo-xcode-demo").path
    @State private var command: String = "swarm peers"
    @State private var output: String = ""
    @State private var busy: Bool = false
    @State private var node: Kubo.Node? = nil
    @State private var peerId: String = ""
    @State private var addrs: String = ""
    @State private var peerCount: Int = 0
    @State private var logTimer: Timer? = nil
    @State private var peerTimer: Timer? = nil
    @State private var bootstraps: String = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Kubo IPFS FFI Demo").font(.title)
            Text("Kubo version: \(version)").font(.caption)
            TextField("Repo path", text: $repoPath)
            HStack {
                Button("Init Repo") { run("init") }
                Button("Start Node (online)") { startNode() }
                Button("Stop Node") { stopNode() }
            }
            if !peerId.isEmpty {
                Text("Peer ID: \(peerId)").font(.caption)
                Text("Listening: \(addrs)").font(.caption)
                Text("Connected peers: \(peerCount)").font(.caption).bold()
            }
            HStack {
                Button("Swarm Peers") { showPeers() }
                Button("Bootstrap List") { run("bootstrap list") }
                Button("Dial Bootstrap") { dialBootstrap() }
                Button("ID") { run("id") }
            }
            HStack {
                TextField("Command", text: $command)
                Button("Run") { run(command) }.disabled(busy)
            }
            if busy { ProgressView().scaleEffect(0.7) }
            ScrollView {
                Text(output).font(.system(.body, design: .monospaced)).frame(maxWidth: .infinity, alignment: .leading)
            }
            .frame(minHeight: 300)
            .background(Color.black.opacity(0.05))
            .cornerRadius(8)
        }
        .padding()
        .frame(minWidth: 720, minHeight: 640)
        .onAppear { startLogPolling(); autoStart() }
    }

    private func log(_ s: String) {
        let ts = ISO8601DateFormatter().string(from: Date())
        output = "[\(ts)] \(s)\n" + output
        let logFile = FileManager.default.currentDirectoryPath + "/kubo-xcode.log"
        if let h = FileHandle(forWritingAtPath: logFile) {
            h.seekToEndOfFile()
            h.write(("[\(ts)] \(s)\n").data(using: .utf8)!)
            h.closeFile()
        }
    }

    private func autoStart() {
        let logFile = FileManager.default.currentDirectoryPath + "/kubo-xcode.log"
        FileManager.default.createFile(atPath: logFile, contents: nil)
        func w(_ s: String) { if let h = FileHandle(forWritingAtPath: logFile) { h.seekToEndOfFile(); h.write((s+"\n").data(using: .utf8)!); h.closeFile() } }
        w("AUTO: starting node")
        DispatchQueue.global().async {
            do {
                if !FileManager.default.fileExists(atPath: repoPath) {
                    try Kubo.initRepo(repoPath)
                }
                let n = try Kubo.Node(repoPath: repoPath, online: true)
                DispatchQueue.main.async {
                    self.node = n
                    self.peerId = n.peerId ?? ""
                    self.addrs = n.listeningAddrs ?? ""
                    w("AUTO: peer=\(self.peerId)")
                    w("AUTO: addrs=\(self.addrs)")
                    self.log("[auto] node started peer=\(self.peerId)")
                    self.log("[auto] listening: \(self.addrs)")
                    self.startPeerPolling()
                    // dial bootstrap to show network activity
                    self.dialBootstrap()
                }
            } catch {
                DispatchQueue.main.async { w("AUTO ERROR: \(error)"); self.log("Error: \(error)") }
            }
        }
    }

    private func startLogPolling() {
        logTimer = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { _ in
            let chunk = Kubo.getLogs()
            if !chunk.isEmpty {
                output = chunk + output
            }
        }
    }

    private func startPeerPolling() {
        peerTimer?.invalidate()
        peerTimer = Timer.scheduledTimer(withTimeInterval: 4.0, repeats: true) { _ in
            guard let n = self.node else { return }
            let peers = n.swarmPeers()
            DispatchQueue.main.async {
                self.peerCount = peers.count
                if peers.count > 0 {
                    self.log("peers(\(peers.count)):\n\(peers.joined(separator: "\n"))")
                }
            }
        }
    }

    private func startNode() {
        busy = true
        DispatchQueue.global().async {
            do {
                if !FileManager.default.fileExists(atPath: repoPath) {
                    try Kubo.initRepo(repoPath)
                }
                let n = try Kubo.Node(repoPath: repoPath, online: true)
                DispatchQueue.main.async {
                    self.node = n
                    self.peerId = n.peerId ?? ""
                    self.addrs = n.listeningAddrs ?? ""
                    self.log("[node started] peer=\(self.peerId)")
                    self.log("listening: \(self.addrs)")
                    self.busy = false
                    self.startPeerPolling()
                }
            } catch {
                DispatchQueue.main.async { self.log("Error: \(error)"); self.busy = false }
            }
        }
    }

    private func stopNode() {
        peerTimer?.invalidate()
        peerTimer = nil
        node?.stop()
        node = nil
        peerId = ""
        addrs = ""
        peerCount = 0
        log("[node stopped]")
    }

    private func dialBootstrap() {
        busy = true
        log("$ dialing bootstrap peers")
        DispatchQueue.global().async {
            guard let n = self.node else {
                DispatchQueue.main.async { self.log("no node running"); self.busy = false }
                return
            }
            let bootstraps = [
                "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
                "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
            ]
            for p in bootstraps {
                let ok = n.connect(p)
                DispatchQueue.main.async { self.log("connect \(p): \(ok ? "OK" : "FAIL")") }
            }
            let peers = n.swarmPeers()
            DispatchQueue.main.async {
                self.peerCount = peers.count
                self.log("peers(\(peers.count)):\n\(peers.joined(separator: "\n"))")
                self.busy = false
            }
        }
    }

    private func showPeers() {
        guard let n = node else { log("no node running"); return }
        let peers = n.swarmPeers()
        peerCount = peers.count
        log("peers(\(peers.count)):\n\(peers.joined(separator: "\n"))")
    }

    private func run(_ cmd: String) {
        busy = true
        log("$ \(cmd)")
        DispatchQueue.global().async {
            do {
                let k = Kubo(repoPath: repoPath)
                let out = try k.run(cmd)
                DispatchQueue.main.async { self.log(out); self.busy = false }
            } catch {
                DispatchQueue.main.async { self.log("Error: \(error)"); self.busy = false }
            }
        }
    }
}