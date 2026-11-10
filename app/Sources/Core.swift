import Foundation

enum ReportBuilder {
    enum ReportError: Error { case invalidMessage }
    static let maxBodyBytes = 16 * 1024

    static func build(message: String, contact: String, includeRecentWords: Bool,
                      version: String, osVersion: String, model: String, enabled: Bool,
                      statsData: Data?, eventsData: Data?, day: String) throws -> [String: Any] {
        let trimmed = message.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, trimmed.count <= 2000 else { throw ReportError.invalidMessage }
        let allStats = (try? JSONSerialization.jsonObject(with: statsData ?? Data())) as? [String: [String: Int]] ?? [:]
        let today = allStats[day] ?? [:]
        var result: [String: Any] = ["schema": 1, "app": "keyflip", "platform": "macos", "app_version": version,
            "os_version": osVersion, "model": model, "enabled": enabled,
            "stats_today": ["runs": (today["runs_th"] ?? 0) + (today["runs_en"] ?? 0) + (today["runs_neutral"] ?? 0),
                            "conversions": (today["auto_typing"] ?? 0) + (today["auto_space"] ?? 0),
                            "undos": (today["undo_backspace"] ?? 0) + (today["undo_shift"] ?? 0)],
            "message": trimmed]
        let cleanContact = String(contact.trimmingCharacters(in: .whitespacesAndNewlines).prefix(200))
        if !cleanContact.isEmpty { result["contact"] = cleanContact }
        if includeRecentWords {
            let lines = (eventsData.flatMap { String(data: $0, encoding: .utf8) } ?? "").split(whereSeparator: \.isNewline)
            let words: [[String: String]] = lines.reversed().compactMap { line in
                guard let data = String(line).data(using: .utf8), let item = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                      let kind = item["kind"] as? String, ["typing", "space", "auto_typing", "auto_space", "undo_typing", "undo_space", "undo_backspace", "undo_shift"].contains(kind),
                      let word = item["word"] as? String, let value = item["result"] as? String else { return nil }
                return ["word": String(word.prefix(40)), "result": String(value.prefix(40)), "kind": kind.hasPrefix("undo_") ? "reverted" : "typing"]
            }.prefix(10).reversed()
            result["recent_words"] = words
        }
        return result
    }

    static func jsonData(_ report: [String: Any]) throws -> Data {
        let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys, .withoutEscapingSlashes])
        guard data.count <= maxBodyBytes else { throw NSError(domain: "KeyFlipReport", code: 413, userInfo: [NSLocalizedDescriptionKey: "รายงานมีขนาดใหญ่เกินไป"]) }
        return data
    }
}

enum Lang: String { case th, en }
enum Key: Equatable { case char(String), space, tab, returnKey, backspace, undo, reset }
enum Action: Equatable { case pass; case replace(delete: Int, insert: String, switchTo: Lang); case revert(delete: Int, insert: String, switchTo: Lang) }
enum TrackerEvent {
    case runEnded(lang: String)
    case converted(kind: String, from: String, to: String, score: Double, key: Int?, prev: Lang?, word: String, result: String)
    case reverted(kind: String, from: String, to: String, word: String, result: String)
}
