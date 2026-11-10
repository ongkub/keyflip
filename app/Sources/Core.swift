import Foundation

enum Lang: String { case th, en }
enum Key: Equatable { case char(String), space, tab, returnKey, backspace, undo, reset }
enum Action: Equatable { case pass; case replace(delete: Int, insert: String, switchTo: Lang); case revert(delete: Int, insert: String, switchTo: Lang) }
enum TrackerEvent {
    case runEnded(lang: String)
    case converted(kind: String, from: String, to: String, score: Double, key: Int?, prev: Lang?, word: String, result: String)
    case reverted(kind: String, from: String, to: String, word: String, result: String)
}
