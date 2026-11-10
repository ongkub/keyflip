// Demo helper for filming KeyFlip: drives one app with physical key codes, slowly enough to film.
// It only presses the keys you pass and reads the focused field of the app you name.
//   swiftc -O app/tools/kfdemo.swift -o /tmp/kfdemo
//   /tmp/kfdemo activate com.apple.Notes            bring an app to the front
//   /tmp/kfdemo place com.apple.Notes 120 110 1240 760   move/resize its focused window (points)
//   /tmp/kfdemo keys com.apple.Notes cmd+n          press shortcuts (cmd/shift/alt/ctrl + key, return, tab, delete, escape)
//   /tmp/kfdemo layout ABC|Thai                     select an input source
//   /tmp/kfdemo type com.apple.Notes "oyf meeting "  type plain keys; '~' = 1 s pause
//   /tmp/kfdemo read com.apple.Notes                print the focused field's text and the current layout
//   /tmp/kfdemo park                                move the pointer to the bottom-left corner
import AppKit
import Carbon
import ApplicationServices

let plain: [Character: Int] = ["a":0,"s":1,"d":2,"f":3,"h":4,"g":5,"z":6,"x":7,"c":8,"v":9,"b":11,"q":12,"w":13,"e":14,"r":15,"y":16,"t":17,"1":18,"2":19,"3":20,"4":21,"6":22,"5":23,"=":24,"9":25,"7":26,"-":27,"8":28,"0":29,"]":30,"o":31,"u":32,"[":33,"i":34,"p":35,"l":37,"j":38,"'":39,"k":40,";":41,"\\":42,",":43,"/":44,"n":45,"m":46,".":47,"`":50," ":49]
let shifted: [Character: Character] = ["!":"1","@":"2","#":"3","$":"4","%":"5","^":"6","&":"7","*":"8","(":"9",")":"0","_":"-","+":"=","{":"[","}":"]",":":";","\"":"'","<":",",">":".","?":"/","|":"\\","~":"`"]
let named: [String: Int] = ["return": 36, "tab": 48, "delete": 51, "backspace": 51, "escape": 53, "esc": 53, "forwarddelete": 117, "space": 49]
let src = CGEventSource(stateID: .hidSystemState)
func pause(_ ms: Int) { RunLoop.main.run(until: Date().addingTimeInterval(Double(ms) / 1000)) }
func app(_ id: String) -> NSRunningApplication? { NSRunningApplication.runningApplications(withBundleIdentifier: id).first }
func requireFront(_ id: String) { if NSWorkspace.shared.frontmostApplication?.bundleIdentifier != id { print("ABORT: \(id) is not frontmost (front: \(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "-"))"); exit(3) } }
// Translate with the layout that is active *now*, like a physical keyboard does. Synthetic events otherwise carry
// characters from this process's stale layout cache, so after KeyFlip switches layouts the app would see the
// old layout's characters while the screen shows the new one.
func unicode(_ code: Int, shift: Bool) -> [UniChar] {
    guard let s = TISCopyCurrentKeyboardLayoutInputSource()?.takeRetainedValue(),
          let ptr = TISGetInputSourceProperty(s, kTISPropertyUnicodeKeyLayoutData) else { return [] }
    let data = Unmanaged<CFData>.fromOpaque(ptr).takeUnretainedValue()
    let layout = unsafeBitCast(CFDataGetBytePtr(data), to: UnsafePointer<UCKeyboardLayout>.self)
    var dead: UInt32 = 0, length = 0; var chars = [UniChar](repeating: 0, count: 8)
    let mods: UInt32 = shift ? UInt32((shiftKey >> 8) & 0xFF) : 0
    UCKeyTranslate(layout, UInt16(code), UInt16(kUCKeyActionDown), mods, UInt32(LMGetKbdType()), OptionBits(kUCKeyTranslateNoDeadKeysBit), &dead, 8, &length, &chars)
    return Array(chars.prefix(length))
}
func post(_ code: Int, flags: CGEventFlags, hold: Int = 30) {
    let text = flags.contains(.maskCommand) || flags.contains(.maskControl) || named.values.contains(code) && code != 49 ? [] : unicode(code, shift: flags.contains(.maskShift))
    for down in [true, false] {
        let e = CGEvent(keyboardEventSource: src, virtualKey: CGKeyCode(code), keyDown: down)!
        e.flags = flags
        if !text.isEmpty { e.keyboardSetUnicodeString(stringLength: text.count, unicodeString: text) }
        e.post(tap: .cghidEventTap); pause(down ? hold : 10)
    }
}
func shiftEdge(_ down: Bool) { let e = CGEvent(keyboardEventSource: src, virtualKey: 56, keyDown: down)!; e.type = .flagsChanged; e.flags = down ? .maskShift : []; e.post(tap: .cghidEventTap); pause(20) }
func typeChar(_ c: Character) {
    if let k = plain[c] { post(k, flags: []) }
    else if let base = shifted[c], let k = plain[base] { shiftEdge(true); post(k, flags: .maskShift); shiftEdge(false) }
    else if c.isUppercase, let k = plain[Character(c.lowercased())] { shiftEdge(true); post(k, flags: .maskShift); shiftEdge(false) }
    else { print("no key for", c); exit(2) }
}
func combo(_ spec: String) {
    var flags: CGEventFlags = []; var code: Int? = nil
    for part in spec.lowercased().split(separator: "+").map(String.init) {
        switch part {
        case "cmd": flags.insert(.maskCommand)
        case "shift": flags.insert(.maskShift)
        case "alt", "opt": flags.insert(.maskAlternate)
        case "ctrl": flags.insert(.maskControl)
        default: code = named[part] ?? (part.count == 1 ? plain[Character(part)] : nil)
        }
    }
    guard let k = code else { print("unknown key", spec); exit(2) }
    post(k, flags: flags, hold: 40); pause(120)
}
func select(_ name: String) {
    let id = name.hasPrefix("com.apple.") ? name : "com.apple.keylayout.\(name)"
    let list = TISCreateInputSourceList([kTISPropertyInputSourceID as String: id] as CFDictionary, false).takeRetainedValue() as NSArray
    if let s = list.firstObject { TISSelectInputSource((s as! TISInputSource)) }; pause(300)
}
func current() -> String { let s = TISCopyCurrentKeyboardInputSource().takeRetainedValue(); return (Unmanaged<CFString>.fromOpaque(TISGetInputSourceProperty(s, kTISPropertyInputSourceID)).takeUnretainedValue() as String).replacingOccurrences(of: "com.apple.keylayout.", with: "") }
func readField(_ id: String) -> String {
    guard let pid = app(id)?.processIdentifier else { return "<not running>" }
    let a = AXUIElementCreateApplication(pid); var focused: CFTypeRef?
    AXUIElementCopyAttributeValue(a, kAXFocusedUIElementAttribute as CFString, &focused)
    var value: CFTypeRef?; if let f = focused { AXUIElementCopyAttributeValue(f as! AXUIElement, kAXValueAttribute as CFString, &value) }
    return value as? String ?? "<unreadable>"
}

let a = CommandLine.arguments
guard a.count >= 2 else { print("usage: see header"); exit(2) }
switch a[1] {
case "activate":
    guard a.count >= 3, let r = app(a[2]) else { print("not running"); exit(1) }
    r.activate(); for _ in 0..<30 where NSWorkspace.shared.frontmostApplication?.bundleIdentifier != a[2] { pause(100) }; pause(300)
    print("front:", NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "-")
case "place":
    guard a.count >= 7, let pid = app(a[2])?.processIdentifier, let x = Double(a[3]), let y = Double(a[4]), let w = Double(a[5]), let h = Double(a[6]) else { print("bad args"); exit(2) }
    let ax = AXUIElementCreateApplication(pid); var win: CFTypeRef?
    AXUIElementCopyAttributeValue(ax, kAXFocusedWindowAttribute as CFString, &win)
    guard let window = win else { print("no focused window"); exit(1) }
    var p = CGPoint(x: x, y: y), s = CGSize(width: w, height: h)
    AXUIElementSetAttributeValue(window as! AXUIElement, kAXPositionAttribute as CFString, AXValueCreate(.cgPoint, &p)!)
    AXUIElementSetAttributeValue(window as! AXUIElement, kAXSizeAttribute as CFString, AXValueCreate(.cgSize, &s)!)
    print("placed")
case "keys":
    guard a.count >= 4 else { exit(2) }; requireFront(a[2]); for spec in a[3...] { combo(spec) }
case "layout":
    guard a.count >= 3 else { exit(2) }; select(a[2]); print("layout:", current())
case "type":
    guard a.count >= 4 else { exit(2) }; requireFront(a[2])
    let per = a.count >= 5 ? Int(a[4]) ?? 115 : 115
    for (i, c) in a[3].enumerated() {
        if c == "~" { pause(1000); continue }
        requireFront(a[2])
        if c == " " { pause(260) }
        typeChar(c); pause(per + [0, 25, -15, 10, -20, 30][i % 6])
        if c == " " { pause(420) }   // people pause after a word; also lets a layout switch settle
    }
case "read":
    guard a.count >= 3 else { exit(2) }; print("text:", readField(a[2]).debugDescription, "| layout:", current())
case "frame":   // print the focused window's frame so it can be restored after filming
    guard a.count >= 3, let pid = app(a[2])?.processIdentifier else { exit(2) }
    let ax = AXUIElementCreateApplication(pid); var win: CFTypeRef?
    AXUIElementCopyAttributeValue(ax, kAXFocusedWindowAttribute as CFString, &win)
    guard let window = win else { print("no focused window"); exit(1) }
    var pv: CFTypeRef?, sv: CFTypeRef?; var p = CGPoint.zero, s = CGSize.zero
    AXUIElementCopyAttributeValue(window as! AXUIElement, kAXPositionAttribute as CFString, &pv)
    AXUIElementCopyAttributeValue(window as! AXUIElement, kAXSizeAttribute as CFString, &sv)
    if let pv { AXValueGetValue(pv as! AXValue, .cgPoint, &p) }; if let sv { AXValueGetValue(sv as! AXValue, .cgSize, &s) }
    print(Int(p.x), Int(p.y), Int(s.width), Int(s.height))
case "park":
    CGWarpMouseCursorPosition(CGPoint(x: 2, y: 978)); print("parked")
default:
    print("unknown command"); exit(2)
}
