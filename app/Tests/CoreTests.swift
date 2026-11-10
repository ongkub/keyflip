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

print("\(failures == 0 ? "All cases passed" : "\(failures) failed")")
exit(failures == 0 ? 0 : 1)
