import Foundation

enum Lang: String { case th, en }
struct Decision { let convert: Bool; let score: Double; let candidate: String; let target: Lang }
protocol Decider: AnyObject { func decide(run: String, prev: Lang?, complete: Bool) -> Decision }
protocol LayoutMapper { func convert(_ s: String, to: Lang) -> String }

final class TableMapper: LayoutMapper {
    private var abcToThai: [String: String] = [:]
    private var thaiToABC: [String: String] = [:]
    init(pairsJSON: URL) throws {
        let data = try Data(contentsOf: pairsJSON)
        let pairs = try JSONSerialization.jsonObject(with: data) as! [[String]]
        for pair in pairs where pair.count == 2 { abcToThai[pair[0]] = pair[1]; thaiToABC[pair[1]] = pair[0] }
    }
    func convert(_ s: String, to: Lang) -> String {
        let map = to == .th ? abcToThai : thaiToABC
        return s.unicodeScalars.map { map[String($0)] ?? String($0) }.joined()
    }
}

enum Key: Equatable { case char(String), space, tab, returnKey, backspace, undo, reset }
enum Action: Equatable { case pass; case replace(delete: Int, insert: String, switchTo: Lang); case revert(delete: Int, insert: String, switchTo: Lang) }
enum TrackerEvent { case runEnded(lang: String); case converted(kind: String, from: String, to: String, score: Double, key: Int?, prev: Lang?, word: String, result: String); case reverted(kind: String, from: String, to: String, word: String, result: String) }

final class RunTracker {
    private let decider: Decider
    private let mapper: LayoutMapper
    private let deleteMode: String
    private var run = ""
    private var prev: Lang?
    private var checkedStopped = false
    private var undoRecord: (source: Lang, target: Lang, original: String, currentRun: String, suffix: String, active: Bool, fresh: Bool)?
    private var lastEndedUndo: Bool { undoRecord?.active == true }
    var onEvent: ((TrackerEvent) -> Void)?
    private(set) var never: Set<String>
    private var backspaceUndos: [String: Int] = [:]   // a quick Backspace is often just editing
    func clearNever() { never.removeAll(); backspaceUndos.removeAll() }
    init(decider: Decider, mapper: LayoutMapper, never: Set<String>, deleteMode: String = "scalar") { self.decider = decider; self.mapper = mapper; self.never = never; self.deleteMode = deleteMode }
    private func lang(_ s: String) -> Lang? {
        var th = false, en = false
        for scalar in s.unicodeScalars { if (0x0E01...0x0E5B).contains(Int(scalar.value)) { th = true }; if (65...90).contains(Int(scalar.value)) || (97...122).contains(Int(scalar.value)) { en = true } }
        return th == en ? nil : (th ? .th : .en)
    }
    private func count(_ s: String) -> Int { deleteMode == "grapheme" ? s.count : s.unicodeScalars.count }
    private func finish() { let l = lang(run)?.rawValue ?? "neutral"; onEvent?(.runEnded(lang: l)); if let x = lang(run) { prev = x }; run = ""; checkedStopped = false }
    private func shouldSkip(_ complete: Bool) -> Bool { let hasTh = run.unicodeScalars.contains { (0x0E01...0x0E5B).contains(Int($0.value)) }; let hasEn = run.unicodeScalars.contains { (65...90).contains(Int($0.value)) || (97...122).contains(Int($0.value)) }; return run.isEmpty || checkedStopped || (hasTh && hasEn) || (complete ? run.unicodeScalars.count > 40 || never.contains(run) : run.unicodeScalars.count > 20 || never.contains(where: { $0.hasPrefix(run) })) }
    func handle(_ key: Key) -> Action {
        switch key {
        case .reset: run = ""; prev = nil; undoRecord = nil; checkedStopped = false; return .pass
        case .undo:
            guard let u = undoRecord, u.active else { return .pass }
            return revert(kind: "shift", u: u)
        case .backspace:
            if let u = undoRecord, u.active, u.fresh { return revert(kind: "backspace", u: u) }
            if run.isEmpty { prev = nil; undoRecord = nil; checkedStopped = false }
            else { run = String(run.unicodeScalars.dropLast()); undoRecord?.currentRun = run; undoRecord?.fresh = false; checkedStopped = false }
            return .pass
        case .returnKey: finish(); undoRecord = nil; prev = nil; return .pass
        case .space, .tab:
            let suffix = key == .space ? " " : "\t"
            if undoRecord?.active == true, !run.isEmpty {
                // a typing-converted run ends here: remember the suffix for undo, count the run, check the next one
                undoRecord?.suffix += suffix
                onEvent?(.runEnded(lang: lang(run)?.rawValue ?? "neutral")); if let x = lang(run) { prev = x }
                run = ""; checkedStopped = false
                return .pass
            }
            if run.isEmpty, undoRecord?.active == true { undoRecord?.active = false }
            if !shouldSkip(true) { let d = decider.decide(run: run, prev: prev, complete: true); if d.convert { let source = lang(run) ?? (d.target == .th ? .en : .th); let original = run; let inserted = d.candidate + suffix; let n = count(run); onEvent?(.runEnded(lang: d.target.rawValue)); onEvent?(.converted(kind: "space", from: source.rawValue, to: d.target.rawValue, score: d.score, key: nil, prev: prev, word: original, result: d.candidate)); undoRecord = (source,d.target,original,d.candidate,suffix,true,true); run = ""; checkedStopped = true; prev = d.target; return .replace(delete: n, insert: inserted, switchTo: d.target) } }
            finish(); return .pass
        case .char(let s):
            if s.isEmpty || s.unicodeScalars.contains(where: { CharacterSet.controlCharacters.contains($0) }) { run = ""; prev = nil; undoRecord = nil; checkedStopped = false; return .pass }
            if undoRecord != nil { undoRecord?.fresh = false; if run.isEmpty { undoRecord?.active = false; undoRecord = nil } }
            run += s
            if undoRecord?.active == true { undoRecord?.currentRun = run }
            guard !shouldSkip(false) else { return .pass }
            let d = decider.decide(run: run, prev: prev, complete: false)
            if d.convert {
                let source = lang(run) ?? (d.target == .th ? .en : .th); let original = run; let n = count(run)
                onEvent?(.converted(kind: "typing", from: source.rawValue, to: d.target.rawValue, score: d.score, key: n, prev: prev, word: original, result: d.candidate))
                undoRecord = (source,d.target,original,d.candidate,"",true,true); run = d.candidate; prev = d.target; checkedStopped = true
                return .replace(delete: max(0,n-1), insert: d.candidate, switchTo: d.target)
            }
            return .pass
        }
    }
    private func revert(kind: String, u: (source: Lang, target: Lang, original: String, currentRun: String, suffix: String, active: Bool, fresh: Bool)) -> Action {
        let word = u.currentRun; let back = mapper.convert(word, to: u.source); let shown = word + u.suffix
        if kind == "backspace" { backspaceUndos[back, default: 0] += 1 }
        if kind != "backspace" || backspaceUndos[back, default: 0] >= 2 { never.insert(back) }
        onEvent?(.reverted(kind: kind, from: u.target.rawValue, to: u.source.rawValue, word: word, result: back)); undoRecord = nil; prev = u.source
        if u.suffix.isEmpty { run = back; checkedStopped = true } else { run = ""; checkedStopped = false }  // run already ended by space/tab
        return .revert(delete: count(shown), insert: back + u.suffix, switchTo: u.source)
    }
}
