import SwiftUI
import Kubo

print("Kubo version: \(Kubo.version())")

struct ContentView: View {
    @State private var version: String = Kubo.version()
    @State private var repoPath: String = FileManager.default.temporaryDirectory.appendingPathComponent("kubo-xcode-demo").path
    @State private var command: String = "version"
    @State private var output: String = ""
    @State private var busy: Bool = false

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Kubo IPFS FFI Demo").font(.title)
            Text("Kubo version: \(version)").font(.caption)
            TextField("Repo path", text: $repoPath)
            HStack {
                Button("Init Repo") { run("init") }
                Button("ID") { run("id") }
                Button("Version") { run("version") }
            }
            HStack {
                TextField("Command", text: $command)
                Button("Run") { run(command) }.disabled(busy)
            }
            if busy { ProgressView().scaleEffect(0.7) }
            ScrollView {
                Text(output).font(.system(.body, design: .monospaced)).frame(maxWidth: .infinity, alignment: .leading)
            }
            .frame(minHeight: 200)
            .background(Color.black.opacity(0.05))
            .cornerRadius(8)
        }
        .padding()
        .frame(minWidth: 520, minHeight: 420)
    }

    private func run(_ cmd: String) {
        busy = true
        output = ""
        DispatchQueue.global().async {
            do {
                if cmd.trimmingCharacters(in: .whitespaces) == "init" {
                    try Kubo.initRepo(repoPath)
                    DispatchQueue.main.async { output = "repo initialized at \(repoPath)"; busy = false }
                    return
                }
                let k = Kubo(repoPath: repoPath)
                let out = try k.run(cmd)
                DispatchQueue.main.async { output = out; busy = false }
            } catch {
                DispatchQueue.main.async { output = "Error: \(error)"; busy = false }
            }
        }
    }
}

// SwiftUI app entry point
import AppKit
let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.run()

class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 560, height: 480),
                              styleMask: [.titled, .closable, .resizable],
                              backing: .buffered, defer: false)
        window.title = "Kubo IPFS FFI Demo"
        window.contentView = NSHostingView(rootView: ContentView())
        window.makeKeyAndOrderFront(nil)
    }
}