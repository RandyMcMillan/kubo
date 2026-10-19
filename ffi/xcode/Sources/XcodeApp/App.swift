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
    @State private var version: String = ""
    @State private var repoPath: String = FileManager.default.temporaryDirectory.appendingPathComponent("kubo-xcode-demo").path
    @State private var command: String = "id"
    @State private var output: String = ""
    @State private var busy: Bool = false
    @State private var peerAddr: String = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Kubo IPFS FFI Demo").font(.title)
            Text("Kubo version: \(version)").font(.caption)
            TextField("Repo path", text: $repoPath)
            HStack {
                Button("Init Repo") { run("init") }
                Button("ID") { run("id") }
                Button("Version") { run("version") }
                Button("Swarm Addrs") { run("swarm addrs listen") }
            }
            HStack {
                TextField("Peer multiaddr to connect", text: $peerAddr)
                Button("Connect") { run("swarm connect \(peerAddr)") }
            }
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
        .frame(minWidth: 600, minHeight: 520)
        .onAppear { version = Kubo.version() }
    }

    private func run(_ cmd: String) {
        busy = true
        let ts = ISO8601DateFormatter().string(from: Date())
        output = "[\(ts)] $ \(cmd)\n" + output
        DispatchQueue.global().async {
            do {
                if cmd.trimmingCharacters(in: .whitespaces) == "init" {
                    try Kubo.initRepo(repoPath)
                    DispatchQueue.main.async { output = "[done] repo initialized at \(repoPath)\n" + output; busy = false }
                    return
                }
                let k = Kubo(repoPath: repoPath)
                let out = try k.run(cmd)
                DispatchQueue.main.async { output = out + "\n" + output; busy = false }
            } catch {
                DispatchQueue.main.async { output = "Error: \(error)\n" + output; busy = false }
            }
        }
    }
}