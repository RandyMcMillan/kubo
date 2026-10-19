import Foundation
import Kubo

print("Kubo version: \(Kubo.version())")
let dir = FileManager.default.temporaryDirectory.appendingPathComponent("kubo-xcode-test").path
try? FileManager.default.removeItem(atPath: dir)
do {
    try Kubo.initRepo(dir)
    let k = Kubo(repoPath: dir)
    let out = try k.run("version")
    print(out)
} catch {
    print("Error: \(error)")
}
