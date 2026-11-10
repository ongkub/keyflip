import Foundation
import Darwin

var failures = 0
func check(_ name: String, _ ok: Bool) { print("\(ok ? "PASS" : "FAIL") \(name)"); if !ok { failures += 1 } }
let root = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
let fixtures = root.appendingPathComponent("app/build/test-models", isDirectory: true)
let config: [String: Any] = ["threshold": -1000, "final_threshold": -1000, "min_len": 2, "orthography_auto": false, "orthography_keep": 0, "deleteMode": "scalar"]
let pairs = try! String(contentsOf: root.appendingPathComponent("app/build/test-pairs.json"), encoding: .utf8)
let bridge = EngineBridge(modelsDir: fixtures, config: config, pairsJSON: pairs)
if let error = bridge.error { print("engine error: \(error)") }
check("fixture engine created", bridge.error == nil)
var events: [TrackerEvent] = []
bridge.onEvents = { events.append($0) }
_ = bridge.handle(.char("a"))
let action = bridge.handle(.char("b"))
check("Thai replace action marshalled", action == .replace(delete: 1, insert: "กข", switchTo: .th))
_ = bridge.handle(.space)
check("converted event parsed", events.contains { if case .converted(_, let from, let to, _, _, _, let word, let result) = $0 { return from == "en" && to == "th" && word == "ab" && result == "กข" }; return false })
check("runEnded event parsed", events.contains { if case .runEnded(let lang) = $0 { return lang == "th" }; return false })
_ = bridge.handle(.undo)
check("never starts from empty", !bridge.never.isEmpty)
bridge.never = ["ไทย", "word"]
check("never get/set roundtrip", bridge.never == ["ไทย", "word"])
bridge.clearNever()
check("never clear", bridge.never.isEmpty)

let missing = EngineBridge(modelsDir: root.appendingPathComponent("app/build/does-not-exist"), config: config)
check("missing models fail safely with error", missing.error != nil && missing.handle(.char("x")) == .pass)

// Real shipping models (models/open), falling back to models/generic.
let realDir = ["models/open", "models/generic"].map { root.appendingPathComponent($0, isDirectory: true) }
    .first { FileManager.default.fileExists(atPath: $0.appendingPathComponent("th.kfm").path) && FileManager.default.fileExists(atPath: $0.appendingPathComponent("en.kfm").path) }
if let generic = realDir {
    let real = EngineBridge(modelsDir: generic, config: [:])
    // After a conversion the app switches the layout, so the same physical keys now produce Thai.
    let pairs = (try? JSONSerialization.jsonObject(with: Data(contentsOf: root.appendingPathComponent("pairs.json")))) as? [[String]] ?? []
    let toThai = Dictionary(pairs.map { ($0[0], $0[1]) }, uniquingKeysWith: { a, _ in a })
    var thaiLayout = false
    var buffer = ""
    for character in "vpkdwxdbo-hk;" {
        let typed = thaiLayout ? (toThai[String(character)] ?? String(character)) : String(character)
        let result = real.handle(.char(typed))
        if case .replace(let count, let insert, let target) = result { buffer = String(String.UnicodeScalarView(buffer.unicodeScalars.dropLast(count))) + insert; thaiLayout = target == .th }
        else { buffer += typed }
    }
    let tail = real.handle(.space)
    if case .replace(let count, let insert, _) = tail { buffer = String(String.UnicodeScalarView(buffer.unicodeScalars.dropLast(count))) + insert }
    else { buffer += " " }
    check("generic model phrase through bridge", buffer == "อยากไปกินข้าว ")
} else { print("SKIP generic model phrase (models/generic missing)") }

let statsJSON = try! JSONSerialization.data(withJSONObject: ["2026-11-10": ["runs_th": 3, "runs_en": 2, "auto_typing": 4, "auto_space": 1, "undo_shift": 2]])
let reportEvents = (0..<12).map { i in
    String(data: try! JSONSerialization.data(withJSONObject: ["kind": i == 11 ? "undo_typing" : "auto_typing", "word": String(repeating: "w", count: 50), "result": String(repeating: "r", count: 50)]), encoding: .utf8)!
}.joined(separator: "\n").data(using: .utf8)!
func makeReport(_ message: String, _ include: Bool) throws -> [String: Any] {
    try ReportBuilder.build(message: message, contact: "", includeRecentWords: include, version: "0.2.0", osVersion: "macOS test", model: "open", enabled: true, statsData: statsJSON, eventsData: reportEvents, day: "2026-11-10")
}
let baseReport = try! makeReport("ทดสอบ", false)
check("report schema keys and types", (baseReport["schema"] as? Int) == 1 && (baseReport["app"] as? String) == "keyflip" && (baseReport["platform"] as? String) == "macos" && (baseReport["enabled"] as? Bool) == true && (baseReport["stats_today"] as? [String: Int]) == ["runs": 5, "conversions": 5, "undos": 2] && baseReport["recent_words"] == nil)
let withWords = try! makeReport("ทดสอบ", true)
let recent = withWords["recent_words"] as? [[String: String]] ?? []
check("recent words capped at 10 and 40 chars", recent.count == 10 && recent.allSatisfy { ($0["word"]?.count ?? 99) == 40 && ($0["result"]?.count ?? 99) == 40 } && recent.last?["kind"] == "reverted")
check("report JSON stays within 16 KB", (try? ReportBuilder.jsonData(baseReport).count).map { $0 <= 16 * 1024 } == true)
check("empty and overlong report messages rejected", (try? makeReport(" \n", false)) == nil && (try? makeReport(String(repeating: "x", count: 2001), false)) == nil)
var oversizedReport = baseReport
oversizedReport["test_padding"] = String(repeating: "x", count: 20_000)
check("report JSON size limit enforced", (try? ReportBuilder.jsonData(oversizedReport)) == nil)

print("\(failures == 0 ? "All cases passed" : "\(failures) failed")")
exit(failures == 0 ? 0 : 1)
