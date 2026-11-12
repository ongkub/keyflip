import Foundation

/// Owns the single-threaded Rust engine on the thread that creates it (the app main thread).
final class EngineBridge {
    private var engine: OpaquePointer?
    private(set) var error: String?
    var onEvents: ((TrackerEvent) -> Void)?

    convenience init(modelsDir: URL, config: [String: Any]) { self.init(modelsDir: modelsDir, config: config, pairsJSON: nil) }

    init(modelsDir: URL, config: [String: Any], pairsJSON: String?) {
        do {
            let data = try JSONSerialization.data(withJSONObject: config, options: [.sortedKeys])
            let json = String(data: data, encoding: .utf8) ?? "{}"
            let create: (UnsafePointer<CChar>, UnsafePointer<CChar>) -> Void = { modelPath, configJSON in
                let make: (UnsafePointer<CChar>?) -> Void = { pairs in
                    var err: UnsafeMutablePointer<CChar>?
                    self.engine = kf_engine_new(modelPath, configJSON, pairs, &err)
                    if let err {
                        self.error = String(cString: err)
                        kf_string_free(err)
                    }
                }
                if let pairsJSON { pairsJSON.withCString { make($0) } }
                else { make(nil) }
            }
            modelsDir.path.withCString { modelPath in
                json.withCString { configJSON in
                    create(modelPath, configJSON)
                }
            }
        } catch { self.error = "invalid engine config: \(error)" }
    }

    deinit { kf_engine_free(engine) }

    var never: Set<String> {
        get {
            guard let engine, let json = kf_never_get(engine) else { return [] }
            defer { kf_string_free(json) }
            guard let data = String(cString: json).data(using: .utf8), let words = try? JSONSerialization.jsonObject(with: data) as? [String] else { return [] }
            return Set(words)
        }
        set {
            guard let engine, let data = try? JSONSerialization.data(withJSONObject: Array(newValue).sorted()), let json = String(data: data, encoding: .utf8) else { return }
            json.withCString { kf_never_set(engine, $0) }
        }
    }

    func clearNever() { if let engine { kf_never_clear(engine) } }

    func convertText(_ text: String) -> (text: String, changed: Int, lang: Lang?)? {
        guard let engine else { return nil }
        let raw: UnsafeMutablePointer<CChar>? = text.withCString { kf_convert_text(engine, $0) }
        guard let raw else { return nil }
        defer { kf_string_free(raw) }
        guard let data = String(cString: raw).data(using: .utf8),
              let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let converted = object["text"] as? String,
              let changed = (object["changed"] as? NSNumber)?.intValue else { return nil }
        let lang = (object["lang"] as? String).flatMap(Lang.init(rawValue:))
        return (converted, changed, lang)
    }

    func handle(_ key: Key) -> Action {
        guard let engine else { return .pass }
        let kind: UInt32
        let text: String?
        switch key {
        case .char(let value): kind = 0; text = value
        case .space: kind = 1; text = nil
        case .tab: kind = 2; text = nil
        case .returnKey: kind = 3; text = nil
        case .backspace: kind = 4; text = nil
        case .undo: kind = 5; text = nil
        case .reset: kind = 6; text = nil
        }
        let raw: KFAction
        if let text { raw = text.withCString { kf_key(engine, kind, $0) } }
        else { raw = kf_key(engine, kind, nil) }
        // The ABI's insert pointer is borrowed and invalidated by the next kf_key.
        let insert = raw.insert.map { String(cString: $0) } ?? ""
        let action: Action
        switch raw.kind {
        case 1: action = .replace(delete: Int(raw.delete_count), insert: insert, switchTo: raw.switch_to == 1 ? .en : .th)
        case 2: action = .revert(delete: Int(raw.delete_count), insert: insert, switchTo: raw.switch_to == 1 ? .en : .th)
        default: action = .pass
        }
        takeEvents().forEach { onEvents?($0) }
        return action
    }

    func takeEvents() -> [TrackerEvent] {
        guard let engine, let raw = kf_take_events(engine) else { return [] }
        defer { kf_string_free(raw) }
        guard let data = String(cString: raw).data(using: .utf8), let items = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else { return [] }
        return items.compactMap { item in
            guard let event = item["event"] as? String else { return nil }
            switch event {
            case "runEnded": return (item["lang"] as? String).map(TrackerEvent.runEnded(lang:))
            case "converted":
                guard let kind = item["kind"] as? String, let from = item["from"] as? String, let to = item["to"] as? String, let score = item["score"] as? Double, let word = item["word"] as? String, let result = item["result"] as? String else { return nil }
                let key = (item["key"] as? NSNumber)?.intValue
                let prev = (item["prev"] as? String).flatMap(Lang.init(rawValue:))
                return .converted(kind: kind, from: from, to: to, score: score, key: key, prev: prev, word: word, result: result)
            case "reverted":
                guard let kind = item["kind"] as? String, let from = item["from"] as? String, let to = item["to"] as? String, let word = item["word"] as? String, let result = item["result"] as? String else { return nil }
                return .reverted(kind: kind, from: from, to: to, word: word, result: result)
            default: return nil
            }
        }
    }
}
