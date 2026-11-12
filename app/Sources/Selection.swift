import AppKit
import ApplicationServices
import Carbon

/// Runs the copy-convert-paste flow away from the event tap callback. Waits happen on a worker queue;
/// every pasteboard access hops to the main thread (AppKit), and the text itself is never logged.
final class Selection {
    private let store: Store
    private let convert: (String) -> (text: String, changed: Int, lang: Lang?)?
    private let select: (Lang) -> Void
    private let completed: () -> Void
    private let lock = NSLock()
    private var running = false
    private var debug = false
    private let systemWide = AXUIElementCreateSystemWide()

    init(store: Store, convert: @escaping (String) -> (text: String, changed: Int, lang: Lang?)?,
         select: @escaping (Lang) -> Void, completed: @escaping () -> Void) {
        self.store = store; self.convert = convert; self.select = select; self.completed = completed
    }

    /// Called on the main thread by the tap. Claims the sole active slot and schedules the flow on a worker queue.
    func begin() -> Bool {
        lock.lock(); defer { lock.unlock() }
        guard !running else { return false }
        running = true
        debug = store.settings["debug"] as? Bool == true   // read here, on the main thread
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
        guard let before = onMain({ self.snapshotPasteboard() }) else { dbg("selection clipboard snapshot failed"); return }
        let oldCount = onMain { NSPasteboard.general.changeCount }
        postShortcut(8)
        guard waitForChange(from: oldCount, milliseconds: 400) else { dbg("selection copy unchanged"); return }
        Thread.sleep(forTimeInterval: 0.03)   // let the app finish writing all its types
        let (copied, copiedCount) = onMain { (NSPasteboard.general.string(forType: .string), NSPasteboard.general.changeCount) }
        // From here on, give the user's pasteboard back only while it still holds our copy (or our converted text):
        // they may have copied something else in the meantime.
        let giveBack = { (expected: Int) in self.onMain { if NSPasteboard.general.changeCount == expected { self.restore(before) } } }
        guard let original = copied, !original.isEmpty, original.count <= 5000 else {
            giveBack(copiedCount); dbg("selection copy invalid length"); return
        }
        guard let result = convert(original), result.changed > 0, result.text != original else {
            giveBack(copiedCount); dbg("selection conversion unchanged length=\(original.count)"); return
        }
        let convertedCount: Int? = onMain {
            let board = NSPasteboard.general
            guard board.changeCount == copiedCount else { return nil }
            board.clearContents()
            let item = NSPasteboardItem()
            item.setString(result.text, forType: .string)
            item.setData(Data(), forType: NSPasteboard.PasteboardType("org.nspasteboard.TransientType"))
            item.setData(Data(), forType: NSPasteboard.PasteboardType("org.nspasteboard.ConcealedType"))
            board.writeObjects([item])
            return board.changeCount
        }
        guard let convertedCount else { dbg("selection pasteboard changed during conversion"); return }
        postShortcut(9)
        Thread.sleep(forTimeInterval: 0.5)   // slow apps (Word) read the pasteboard late; restoring early would paste the old content
        giveBack(convertedCount)
        if let lang = result.lang { DispatchQueue.main.async { self.select(lang) } }
        DispatchQueue.main.async { self.completed() }
        dbg("selection converted in=\(original.count) out=\(result.text.count)")
    }

    private func onMain<T>(_ work: () -> T) -> T { Thread.isMainThread ? work() : DispatchQueue.main.sync(execute: work) }

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
            if onMain({ NSPasteboard.general.changeCount }) != count { return true }
            Thread.sleep(forTimeInterval: 0.01)
        }
        return onMain { NSPasteboard.general.changeCount } != count
    }

    private func postShortcut(_ key: CGKeyCode) {
        for down in [true, false] {
            guard let event = CGEvent(keyboardEventSource: nil, virtualKey: key, keyDown: down) else { continue }
            event.flags = .maskCommand
            event.setIntegerValueField(.eventSourceUserData, value: 0x4B46)
            event.post(tap: .cgSessionEventTap)
        }
    }

    /// Lengths and outcomes only, never the text.
    private func dbg(_ line: String) {
        if debug { DispatchQueue.main.async { self.store.debug(line) } }
    }
}
