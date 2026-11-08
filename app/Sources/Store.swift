import Foundation

final class Store {
    let root = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Library/Application Support/KeyFlipAuto", isDirectory: true)
    var settings: [String: Any] = [:]
    var never: Set<String> = []
    private var stats: [String: [String: Int]] = [:]
    private var lastWrite = Date.distantPast
    private let prefixes = ["com.apple.Terminal", "com.googlecode.iterm2", "dev.warp.", "com.microsoft.VSCode", "com.todesktop.230313mzl4w4u92", "com.apple.dt.Xcode", "com.jetbrains.", "com.sublimetext.", "com.1password.", "com.agilebits.", "com.bitwarden.", "com.apple.keychainaccess"]
    init() {
        try? FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        let settingsURL = root.appendingPathComponent("settings.json")
        if let data = try? Data(contentsOf: settingsURL), let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] { settings = obj }
        else { settings = ["enabled": true, "python": "/usr/bin/python3", "modelsDir": NSNull(), "undoShiftTapMs": 400, "deleteMode": "scalar", "excludedBundlePrefixes": prefixes]; saveSettings() }
        let neverURL = root.appendingPathComponent("never.txt"); if let text = try? String(contentsOf: neverURL, encoding: .utf8) { never = Set(text.split(whereSeparator: \.isNewline).map(String.init)) }
        if let data = try? Data(contentsOf: root.appendingPathComponent("stats.json")), let obj = try? JSONSerialization.jsonObject(with: data) as? [String: [String: Int]] { stats = obj }
    }
    func saveSettings() { if let data = try? JSONSerialization.data(withJSONObject: settings, options: [.prettyPrinted, .sortedKeys]) { try? data.write(to: root.appendingPathComponent("settings.json"), options: .atomic) } }
    func addNever(_ word: String) { guard !word.isEmpty, never.insert(word).inserted else { return }; let url = root.appendingPathComponent("never.txt"); if let h = try? FileHandle(forWritingTo: url) { try? h.seekToEnd(); try? h.write(contentsOf: Data((word + "\n").utf8)); try? h.close() } else { try? Data((word + "\n").utf8).write(to: url) } }
    func increment(_ key: String) { let date = ISO8601DateFormatter(); date.formatOptions = [.withFullDate]; let day = date.string(from: Date()); stats[day, default: [:]][key, default: 0] += 1; if Date().timeIntervalSince(lastWrite) >= 30 { flush() } }
    func flush() { lastWrite = Date(); if let data = try? JSONSerialization.data(withJSONObject: stats, options: [.prettyPrinted, .sortedKeys]) { try? data.write(to: root.appendingPathComponent("stats.json"), options: .atomic) } }
    func appendEvent(_ obj: [String: Any]) { guard JSONSerialization.isValidJSONObject(obj), let data = try? JSONSerialization.data(withJSONObject: obj, options: [.sortedKeys]) else { return }; let url = root.appendingPathComponent("events.jsonl"); if let h = try? FileHandle(forWritingTo: url) { try? h.seekToEnd(); try? h.write(contentsOf: data + Data([10])); try? h.close() } else { try? (data + Data([10])).write(to: url) } }
    func models(bundleResources: URL) -> URL { if let s = settings["modelsDir"] as? String { return URL(fileURLWithPath: s) }; let personal = root.appendingPathComponent("models"); if FileManager.default.fileExists(atPath: personal.appendingPathComponent("th.json.gz").path), FileManager.default.fileExists(atPath: personal.appendingPathComponent("en.json.gz").path) { return personal }; return bundleResources.appendingPathComponent("detector/models") }
    func debug(_ line: String) { let url = root.appendingPathComponent("debug.log"); let stamp = ISO8601DateFormatter().string(from: Date()); let data = Data((stamp + " " + line + "\n").utf8); if let h = try? FileHandle(forWritingTo: url) { try? h.seekToEnd(); try? h.write(contentsOf: data); try? h.close() } else { try? data.write(to: url) } }
    func excluded(_ bundle: String?) -> Bool { guard let bundle else { return false }; let p = settings["excludedBundlePrefixes"] as? [String] ?? prefixes; return p.contains { bundle.hasPrefix($0) } }
}
