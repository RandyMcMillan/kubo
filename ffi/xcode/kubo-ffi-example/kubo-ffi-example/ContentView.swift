import SwiftUI
import WebKit
import Kubo

struct ContentView: View {
    @State private var version: String = Kubo.version()
    @State private var repoPath: String = FileManager.default.temporaryDirectory.appendingPathComponent("kubo-xcode-demo").path
    @State private var command: String = "id"
    @State private var output: String = ""
    @State private var busy: Bool = false
    @State private var node: Kubo.Node? = nil
    @State private var peerId: String = ""
    @State private var addrs: String = ""
    @State private var peerCount: Int = 0
    @State private var logTimer: Timer? = nil
    @State private var peerTimer: Timer? = nil
    @State private var webUrl: String = ""
    @State private var selectedSection: Section = .status

    enum Section: String, CaseIterable, Identifiable {
        case status = "Status"
        case files = "Files"
        case peers = "Peers"
        case settings = "Settings"
        case diagnostics = "Diagnostics"
        case webui = "WebUI"
        var id: String { rawValue }
    }

    var body: some View {
        NavigationSplitView {
            List(selection: $selectedSection) {
                ForEach(Section.allCases) { section in
                    Label(section.rawValue, systemImage: icon(for: section)).tag(section)
                }
            }
            .navigationTitle("Kubo")
        } detail: {
            VStack(alignment: .leading, spacing: 12) {
                Text("Kubo IPFS FFI").font(.title)
                Text("Kubo version: \(version)").font(.caption)
                TextField("Repo path", text: $repoPath)
                HStack {
                    Button("Init Repo") { run("init") }
                    Button("Start Node") { startNode() }
                    Button("Stop Node") { stopNode() }
                }
                if !peerId.isEmpty {
                    Text("Peer ID: \(peerId)").font(.caption)
                    Text("Listening: \(addrs)").font(.caption)
                    Text("Connected peers: \(peerCount)").font(.caption).bold()
                }
                switch selectedSection {
                case .status:
                    HStack {
                        Button("ID") { run("id") }
                        Button("Version") { run("version") }
                        Button("Repo Stat") { run("repo stat") }
                        Button("API: version") { runApi("version") }
                    }
                case .files:
                    HStack {
                        Button("Add (stdin)") { run("add --help") }
                        Button("Files ls /") { run("files ls /") }
                        Button("Pin ls") { run("pin ls") }
                    }
                case .peers:
                    HStack {
                        Button("Swarm Peers") { showPeers() }
                        Button("Dial Bootstrap") { dialBootstrap() }
                        Button("Bootstrap List") { run("bootstrap list") }
                    }
                case .settings:
                    HStack {
                        Button("Config show") { run("config show") }
                        Button("API: config") { runApi("config/show") }
                    }
                case .diagnostics:
                    HStack {
                        Button("Diag") { run("diag sys") }
                        Button("Log level") { run("log level") }
                        Button("Stats bw") { run("stats bw") }
                    }
                case .webui:
                    if !webUrl.isEmpty {
                        WebView(urlString: webUrl)
                    } else {
                        Text("Start a node to load the WebUI").foregroundColor(.secondary)
                    }
                }
                if busy { ProgressView() }
                ScrollView {
                    Text(output).font(.system(.body, design: .monospaced)).frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(minHeight: 200)
                .background(Color.black.opacity(0.05))
                .cornerRadius(8)
            }
            .padding()
        }
        .onAppear { startLogPolling() }
    }

    private func icon(for section: Section) -> String {
        switch section {
        case .status: return "info.circle"
        case .files: return "doc"
        case .peers: return "network"
        case .settings: return "gear"
        case .diagnostics: return "stethoscope"
        case .webui: return "globe"
        }
    }

    private func log(_ s: String) {
        let ts = ISO8601DateFormatter().string(from: Date())
        output = "[\(ts)] \(s)\n" + output
    }

    private func startLogPolling() {
        Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { _ in
            let chunk = Kubo.getLogs()
            if !chunk.isEmpty { output = chunk + output }
        }
    }

    private func startPeerPolling() {
        Timer.scheduledTimer(withTimeInterval: 4.0, repeats: true) { _ in
            guard let n = self.node else { return }
            let peers = n.swarmPeers()
            DispatchQueue.main.async {
                self.peerCount = peers.count
                if peers.count > 0 { self.log("peers(\(peers.count))") }
            }
        }
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
                    self.log("listening: \(self.addrs)")
                    self.log("webui: \(self.webUrl)")
                    self.busy = false
                    self.startPeerPolling()
                    self.dialBootstrap()
                }
            } catch {
                DispatchQueue.main.async { self.log("Error: \(error)"); self.busy = false }
            }
        }
    }

    private func stopNode() {
        node?.stop()
        node = nil
        peerId = ""; addrs = ""; peerCount = 0; webUrl = ""
        log("[node stopped]")
    }

    private func dialBootstrap() {
        busy = true
        log("$ dialing bootstrap peers")
        DispatchQueue.global().async {
            guard let n = self.node else { DispatchQueue.main.async { self.log("no node"); self.busy = false }; return }
            let bootstraps = ["/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN"]
            for p in bootstraps {
                let ok = n.connect(p)
                DispatchQueue.main.async { self.log("connect \(p): \(ok ? "OK" : "FAIL")") }
            }
            let peers = n.swarmPeers()
            DispatchQueue.main.async { self.peerCount = peers.count; self.log("peers(\(peers.count))"); self.busy = false }
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

    private func runApi(_ endpoint: String) {
        busy = true
        log("$ api \(endpoint)")
        guard let node = node else { log("no node"); busy = false; return }
        Task {
            do {
                let out = try await node.apiCall(endpoint)
                await MainActor.run { self.log(out); self.busy = false }
            } catch {
                await MainActor.run { self.log("Error: \(error)"); self.busy = false }
            }
        }
    }
}

struct WebView: NSViewRepresentable {
    let urlString: String
    func makeNSView(context: Context) -> WKWebView {
        let wv = WKWebView()
        if let url = URL(string: urlString) { wv.load(URLRequest(url: url)) }
        return wv
    }
    func updateNSView(_ nsView: WKWebView, context: Context) {
        if let url = URL(string: urlString), nsView.url != url {
            nsView.load(URLRequest(url: url))
        }
    }
}