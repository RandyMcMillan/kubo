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
    @State private var command: String = "version"
    @State private var output: String = ""
    @State private var busy: Bool = false
    @State private var node: Kubo.Node? = nil
    @State private var peerId: String = ""
    @State private var addrs: String = ""
    @State private var logTimer: Timer? = nil

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
            if !peerId.isEmpty { Text("Peer ID: \(peerId)").font(.caption) }
            if !addrs.isEmpty { Text("Addrs: \(addrs)").font(.caption) }
            HStack {
                TextField("Command", text: $command)
                Button("Run") { run(command) }.disabled(busy)
            }
            if busy { ProgressView().scaleEffect(0.7) }
            ScrollView {
                Text(output).font(.system(.body, design: .monospaced)).frame(maxWidth: .infinity, alignment: .leading)
            }
            .frame(minHeight: 240)
            .background(Color.black.opacity(0.05))
            .cornerRadius(8)
        }
        .padding()
        .frame(minWidth: 640, minHeight: 560)
        .onAppear { startLogPolling() }
    }

    private func log(_ s: String) { output = s + "\n" + output }

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
                    self.log("addrs=\(self.addrs)")
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
        peerId = ""
        addrs = ""
        log("[node stopped]")
    }

    private func startLogPolling() {
        logTimer = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { _ in
            let chunk = Kubo.getLogs()
            if !chunk.isEmpty {
                output = chunk + output
            }
        }
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