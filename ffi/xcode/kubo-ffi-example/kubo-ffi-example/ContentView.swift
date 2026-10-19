import SwiftUI
import Kubo

struct ContentView: View {
    @State private var version: String = Kubo.version()
    @State private var repoPath: String = FileManager.default.temporaryDirectory.appendingPathComponent("kubo-xcode-demo").path
    @State private var command: String = "version"
    @State private var output: String = ""
    @State private var busy: Bool = false
    @State private var node: Kubo.Node? = nil
    @State private var peerId: String = ""
    @State private var addrs: String = ""
    @State private var peerCount: Int = 0
    @State private var webUrl: String = ""

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
                TextField("Command", text: $command)
                Button("Run") { run(command) }.disabled(busy)
            }
            if busy { ProgressView() }
            ScrollView {
                Text(output).font(.system(.body, design: .monospaced)).frame(maxWidth: .infinity, alignment: .leading)
            }
            .frame(minHeight: 200)
            if !webUrl.isEmpty {
                Link("Open WebUI", destination: URL(string: webUrl)!).font(.caption)
            }
        }
        .padding()
        .frame(minWidth: 520, minHeight: 480)
    }

    private func log(_ s: String) {
        let ts = ISO8601DateFormatter().string(from: Date())
        output = "[\(ts)] \(s)\n" + output
    }

    private func startNode() {
        busy = true
        DispatchQueue.global().async {
            do {
                if !FileManager.default.fileExists(atPath: repoPath) { try Kubo.initRepo(repoPath) }
                let n = try Kubo.Node(repoPath: repoPath, online: true)
                let apiUrl = n.startApi("/ip4/127.0.0.1/tcp/0") ?? ""
                let parts = apiUrl.split(separator: "/")
                var host = "127.0.0.1"; var port = "5001"; var i = 0
                while i < parts.count {
                    if parts[i] == "ip4" && i + 1 < parts.count { host = String(parts[i+1]) }
                    if parts[i] == "tcp" && i + 1 < parts.count { port = String(parts[i+1]) }
                    i += 1
                }
                DispatchQueue.main.async {
                    self.node = n
                    self.peerId = n.peerId ?? ""
                    self.addrs = n.listeningAddrs ?? ""
                    self.webUrl = "http://\(host):\(port)/webui"
                    self.log("[node started] peer=\(self.peerId)")
                    self.log("webui: \(self.webUrl)")
                    self.busy = false
                }
            } catch {
                DispatchQueue.main.async { self.log("Error: \(error)"); self.busy = false }
            }
        }
    }

    private func stopNode() {
        node?.stop()
        node = nil
        peerId = ""; addrs = ""; webUrl = ""
        log("[node stopped]")
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

@main
struct KuboFFIExampleApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}