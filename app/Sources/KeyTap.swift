import AppKit
import Carbon
import ApplicationServices

final class KeyTap {
    private var tap: CFMachPort?
    private var source: CFRunLoopSource?
    private let tracker: RunTracker
    private let store: Store
    private var shiftDown: [Int64: Date] = [:]
    private var lastShift: Date?
    private var ownSourceChange = false
    var enabled = true
    init(tracker: RunTracker, store: Store) { self.tracker = tracker; self.store = store }
    func install() -> Bool {
        let mask = (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.flagsChanged.rawValue) | (1 << CGEventType.leftMouseDown.rawValue) | (1 << CGEventType.rightMouseDown.rawValue) | (1 << CGEventType.otherMouseDown.rawValue)
        guard let t = CGEvent.tapCreate(tap: .cgSessionEventTap, place: .headInsertEventTap, options: .defaultTap, eventsOfInterest: CGEventMask(mask), callback: { proxy, type, event, refcon in
            guard let refcon else { return Unmanaged.passUnretained(event) }; let me = Unmanaged<KeyTap>.fromOpaque(refcon).takeUnretainedValue(); return me.handle(proxy, type, event)
        }, userInfo: UnsafeMutableRawPointer(Unmanaged.passUnretained(self).toOpaque())) else { return false }
        tap = t; source = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, t, 0); CFRunLoopAddSource(CFRunLoopGetMain(), source, .commonModes); CGEvent.tapEnable(tap: t, enable: true); return true
    }
    private func handle(_ proxy: CGEventTapProxy, _ type: CGEventType, _ event: CGEvent) -> Unmanaged<CGEvent>? {
        if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput { if let tap { CGEvent.tapEnable(tap: tap, enable: true) }; return Unmanaged.passUnretained(event) }
        if event.getIntegerValueField(.eventSourceUserData) == 0x4B46 { return Unmanaged.passUnretained(event) }
        if type == .leftMouseDown || type == .rightMouseDown || type == .otherMouseDown { _ = tracker.handle(.reset); return Unmanaged.passUnretained(event) }
        if type == .flagsChanged {
            let kc = event.getIntegerValueField(.keyboardEventKeycode)
            if kc == 56 || kc == 60 {
                let down = event.flags.contains(.maskShift); if down { shiftDown[kc] = Date() } else if let start = shiftDown.removeValue(forKey: kc), Date().timeIntervalSince(start) <= 0.25 { let now = Date(); let limit = (store.settings["undoShiftTapMs"] as? Double ?? 400) / 1000; if let last = lastShift, now.timeIntervalSince(last) <= limit { _ = tracker.handle(.undo); lastShift = nil } else { lastShift = now } }
            }
            return Unmanaged.passUnretained(event)
        }
        guard type == .keyDown else { return Unmanaged.passUnretained(event) }
        if !enabled || IsSecureEventInputEnabled() || store.excluded(NSWorkspace.shared.frontmostApplication?.bundleIdentifier) || !supportedSource() { _ = tracker.handle(.reset); return Unmanaged.passUnretained(event) }
        let flags = event.flags
        if flags.contains(.maskCommand) || flags.contains(.maskControl) || flags.contains(.maskAlternate) { _ = tracker.handle(.reset); return Unmanaged.passUnretained(event) }
        let kc = event.getIntegerValueField(.keyboardEventKeycode)
        let key: Key
        switch kc { case 49: key = .space; case 48: key = .tab; case 36,76: key = .returnKey; case 51: key = .backspace; case 53,115,116,117,119,121,123,124,125,126: _ = tracker.handle(.reset); return Unmanaged.passUnretained(event); default:
            var length = 0; var chars = [UniChar](repeating: 0, count: 16); event.keyboardGetUnicodeString(maxStringLength: 16, actualStringLength: &length, unicodeString: &chars); if length == 0 { _ = tracker.handle(.reset); return Unmanaged.passUnretained(event) }; key = .char(String(utf16CodeUnits: chars, count: length))
        }
        let action = tracker.handle(key)
        switch action { case .pass: return Unmanaged.passUnretained(event); case .replace(let n, let text, let lang), .revert(let n, let text, let lang): inject(proxy, delete: n, text: text); select(lang); return nil }
    }
    private func inject(_ proxy: CGEventTapProxy, delete: Int, text: String) {
        func post(_ code: CGKeyCode, _ down: Bool, _ value: String? = nil) { guard let e = CGEvent(keyboardEventSource: nil, virtualKey: code, keyDown: down) else { return }; e.flags = []; e.setIntegerValueField(.eventSourceUserData, value: 0x4B46); if let value { let units = Array(value.utf16); e.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units) }; e.tapPostEvent(proxy) }
        for _ in 0..<max(0, delete) { post(51,true); post(51,false) }
        var chunk = "", units = 0
        for c in text { if units + c.utf16.count > 16 { post(0,true,chunk); post(0,false); chunk = ""; units = 0 }; chunk += String(c); units += c.utf16.count }
        if !chunk.isEmpty { post(0,true,chunk); post(0,false) }
    }
    private func supportedSource() -> Bool { guard let src = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(), let id = TISGetInputSourceProperty(src, kTISPropertyInputSourceID) else { return false }; let name = Unmanaged<CFString>.fromOpaque(id).takeUnretainedValue() as String; return name == "com.apple.keylayout.Thai" || name == "com.apple.keylayout.ABC" || name == "com.apple.keylayout.US" }
    private func select(_ lang: Lang) {
        let list = TISCreateInputSourceList([kTISPropertyInputSourceIsEnabled as String: true] as CFDictionary, false).takeRetainedValue() as NSArray
        let candidates = lang == .th ? ["com.apple.keylayout.Thai"] : ["com.apple.keylayout.ABC", "com.apple.keylayout.US"]
        for item in list { let src = item as! TISInputSource; guard let ptr = TISGetInputSourceProperty(src,kTISPropertyInputSourceID) else { continue }; let id = Unmanaged<CFString>.fromOpaque(ptr).takeUnretainedValue() as String; if candidates.contains(id) { ownSourceChange = true; TISSelectInputSource(src); DispatchQueue.main.asyncAfter(deadline: .now()+0.2) { self.ownSourceChange = false }; break } }
    }
    func sourceChangedByUser() { if !ownSourceChange { _ = tracker.handle(.reset) } }
}
