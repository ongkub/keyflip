import AppKit
import ApplicationServices
import Carbon

/// Runs clipboard and accessibility work away from the event tap callback.
final class Selection {
    private let store: Store
    private let convert: (String) -> (text: String, changed: Int, lang: Lang?)?
    private let select: (Lang) -> Void
    private let completed: () -> Void
    private let lock = NSLock()
    private var running = false
    private let systemWide = AXUIElementCreateSystemWide()

    init(store: Store, convert: @escaping (String) -> (text: String, changed: Int, lang: Lang?)?,
         select: @escaping (Lang) -> Void, completed: @escaping () -> Void) {
        self.store = store; self.convert = convert; self.select = select; self.completed = completed
    }

    /// Claims the sole active slot and immediately schedules all waits on a worker queue.
    func begin() -> Bool {
        lock.lock(); defer { lock.unlock() }
        guard !running else { return false }
        running = true
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self else { return }
            defer { self.lock.lock(); self.running = false; self.lock.unlock() }
            self.run()
        }
        return true
    }

    private func run() {
        if selectedLength() == 0 { return }
        let deadline = Date().addingTimeInterval(0.6)
        while Date() < deadline {
            let flags = CGEventSource.flagsState(.combinedSessionState)
            if !flags.contains(.maskControl) && !flags.contains(.maskShift) { break }
            Thread.sleep(forTimeInterval: 0.01)
        }
        let before = snapshotPasteboard()
        guard let before else { dbg("selection clipboard snapshot failed"); return }
        let board = NSPasteboard.general
        let oldCount = board.changeCount
        postShortcut(8)
        let copied = waitForChange(from: oldCount, milliseconds: 400)
        guard copied else { dbg("selection copy unchanged"); return }
        guard let original = board.string(forType: .string), !original.isEmpty, original.count <= 5000 else {
            restore(before); dbg("selection copy invalid length"); return
        }
        guard let result = convert(original), result.changed > 0, result.text != original else {
            restore(before); dbg("selection conversion unchanged length=\(original.count)"); return
        }
        board.clearContents()
        let item = NSPasteboardItem()
        item.setString(result.text, forType: .string)
        item.setData(Data(), forType: NSPasteboard.PasteboardType("org.nspasteboard.TransientType"))
        item.setData(Data(), forType: NSPasteboard.PasteboardType("org.nspasteboard.ConcealedType"))
        board.writeObjects([item])
        let convertedCount = board.changeCount
        postShortcut(9)
        Thread.sleep(forTimeInterval: 0.5)   // slow apps (Word) read the pasteboard late; restoring early would paste the old content
        if board.changeCount == convertedCount { restore(before) }
        if let lang = result.lang { DispatchQueue.main.async { self.select(lang) } }
        DispatchQueue.main.async { self.completed() }
        dbg("selection converted in=\(original.count) out=\(result.text.count)")
    }

    private func selectedLength() -> Int? {
        AXUIElementSetMessagingTimeout(systemWide, 0.03)
        var focused: CFTypeRef?
        guard AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
              let focused, CFGetTypeID(focused) == AXUIElementGetTypeID() else { return nil }
        let element = focused as! AXUIElement
        AXUIElementSetMessagingTimeout(element, 0.03)
        var value: CFTypeRef?
        guard AXUIElementCopyAttributeValue(element, kAXSelectedTextRangeAttribute as CFString, &value) == .success,
              let value, CFGetTypeID(value) == AXValueGetTypeID() else { return nil }
        var range = CFRange()
        guard AXValueGetValue(value as! AXValue, .cfRange, &range) else { return nil }
        return range.length
    }

    private func snapshotPasteboard() -> [[(NSPasteboard.PasteboardType, Data)]]? {
        NSPasteboard.general.pasteboardItems?.map { item in
            item.types.compactMap { type in item.data(forType: type).map { (type, $0) } }
        }
    }

    private func restore(_ saved: [[(NSPasteboard.PasteboardType, Data)]]) {
        let board = NSPasteboard.general
        board.clearContents()
        let items: [NSPasteboardItem] = saved.map { entries in
            let item = NSPasteboardItem()
            for (type, data) in entries { item.setData(data, forType: type) }
            return item
        }
        if !items.isEmpty { board.writeObjects(items) }
    }

    private func waitForChange(from count: Int, milliseconds: Int) -> Bool {
        for _ in 0..<(milliseconds / 10) {
            if NSPasteboard.general.changeCount != count { return true }
            Thread.sleep(forTimeInterval: 0.01)
        }
        return NSPasteboard.general.changeCount != count
    }

    private func postShortcut(_ key: CGKeyCode) {
        for down in [true, false] {
            guard let event = CGEvent(keyboardEventSource: nil, virtualKey: key, keyDown: down) else { continue }
            event.flags = .maskCommand
            event.setIntegerValueField(.eventSourceUserData, value: 0x4B46)
            event.post(tap: .cgSessionEventTap)
        }
    }

    private func dbg(_ line: String) {
        if store.settings["debug"] as? Bool == true { store.debug(line) }
    }
}
