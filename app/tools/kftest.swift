// Live test: post physical key codes (as if typed) into TextEdit, read the result back via AX.
import AppKit
import Carbon
import ApplicationServices

let plain: [Character: Int] = ["a":0,"s":1,"d":2,"f":3,"h":4,"g":5,"z":6,"x":7,"c":8,"v":9,"b":11,"q":12,"w":13,"e":14,"r":15,"y":16,"t":17,"1":18,"2":19,"3":20,"4":21,"6":22,"5":23,"=":24,"9":25,"7":26,"-":27,"8":28,"0":29,"]":30,"o":31,"u":32,"[":33,"i":34,"p":35,"l":37,"j":38,"'":39,"k":40,";":41,"\\":42,",":43,"/":44,"n":45,"m":46,".":47,"`":50," ":49]
let shifted: [Character: Character] = ["!":"1","@":"2","#":"3","$":"4","%":"5","^":"6","&":"7","*":"8","(":"9",")":"0","_":"-","+":"=","{":"[","}":"]",":":";","\"":"'","<":",",">":".","?":"/","|":"\\","~":"`"]
let src = CGEventSource(stateID: .hidSystemState)
var textEditPID: pid_t = 0
func frontOK() -> Bool { NSWorkspace.shared.frontmostApplication?.bundleIdentifier == "com.apple.TextEdit" }
func pause(_ ms: Int) { RunLoop.main.run(until: Date().addingTimeInterval(Double(ms) / 1000)) }  // keep TIS/notifications live
func key(_ code: Int, shift: Bool = false, cmd: Bool = false) {
    guard frontOK() else { print("ABORT: TextEdit is not frontmost"); exit(3) }
    var flags: CGEventFlags = []; if shift { flags.insert(.maskShift) }; if cmd { flags.insert(.maskCommand) }
    if shift { modifier(56, down: true) }
    for down in [true, false] { let e = CGEvent(keyboardEventSource: src, virtualKey: CGKeyCode(code), keyDown: down)!; e.flags = flags; e.post(tap: .cghidEventTap); pause(15) }
    if shift { modifier(56, down: false) }
    pause(45)
}
func modifier(_ code: Int, down: Bool) { let e = CGEvent(keyboardEventSource: src, virtualKey: CGKeyCode(code), keyDown: down)!; e.type = .flagsChanged; e.flags = down ? .maskShift : []; e.post(tap: .cghidEventTap); pause(20) }
func type(_ s: String) {
    var i = s.startIndex
    while i < s.endIndex {
        if s[i...].hasPrefix("<BS>") { key(51); i = s.index(i, offsetBy: 4); continue }
        if s[i...].hasPrefix("<SHIFT2>") { for _ in 0..<2 { modifier(56, down: true); pause(40); modifier(56, down: false); pause(80) }; pause(150); i = s.index(i, offsetBy: 8); continue }
        let c = s[i]
        if let k = plain[c] { key(k) } else if let base = shifted[c], let k = plain[base] { key(k, shift: true) }
        else if c.isUppercase, let k = plain[Character(c.lowercased())] { key(k, shift: true) } else { print("no key for", c); exit(2) }
        i = s.index(after: i)
    }
}
func select(_ id: String) {
    let list = TISCreateInputSourceList([kTISPropertyInputSourceID as String: id] as CFDictionary, false).takeRetainedValue() as NSArray
    if let s = list.firstObject { TISSelectInputSource((s as! TISInputSource)) }; pause(300)
}
func current() -> String { let s = TISCopyCurrentKeyboardInputSource().takeRetainedValue(); return Unmanaged<CFString>.fromOpaque(TISGetInputSourceProperty(s, kTISPropertyInputSourceID)).takeUnretainedValue() as String }
func readText() -> String {
    let app = AXUIElementCreateApplication(textEditPID); var focused: CFTypeRef?
    AXUIElementCopyAttributeValue(app, kAXFocusedUIElementAttribute as CFString, &focused)
    var value: CFTypeRef?; if let f = focused { AXUIElementCopyAttributeValue(f as! AXUIElement, kAXValueAttribute as CFString, &value) }
    return value as? String ?? "<unreadable>"
}
// setup: TextEdit with an empty plain-text file passed as argv[1]
let file = CommandLine.arguments[1]
NSWorkspace.shared.open([URL(fileURLWithPath: file)], withApplicationAt: URL(fileURLWithPath: "/System/Applications/TextEdit.app"), configuration: NSWorkspace.OpenConfiguration()) { app, _ in if let app { textEditPID = app.processIdentifier } }
for _ in 0..<50 where textEditPID == 0 || !frontOK() { RunLoop.current.run(until: Date().addingTimeInterval(0.1)) }
pause(800)
let cases: [(String, String, String, String, String)] = [   // name, layout, keys, expected text, expected layout
    ("A wrong-layout Thai sentence", "com.apple.keylayout.ABC", "vpkdwxdbo-hk; ", "อยากไปกินข้าว ", "com.apple.keylayout.Thai"),
    ("B English typed in Thai mode", "com.apple.keylayout.Thai", "meeting ", "meeting ", "com.apple.keylayout.ABC"),
    ("C short Thai word", "com.apple.keylayout.ABC", "l;ylfu ", "สวัสดี ", "com.apple.keylayout.Thai"),
    ("F correct Thai stays", "com.apple.keylayout.Thai", "l;ylfu8iy[ ", "สวัสดีครับ ", "com.apple.keylayout.Thai"),
    ("G correct English stays", "com.apple.keylayout.ABC", "hello world ", "hello world ", "com.apple.keylayout.ABC"),
    ("E backspace undo", "com.apple.keylayout.ABC", "vpkd<BS>", "vpkd", "com.apple.keylayout.ABC"),
    ("D shift-shift undo after more typing", "com.apple.keylayout.ABC", "wxdbo-hk;<SHIFT2>", "wxdbo-hk;", "com.apple.keylayout.ABC"),
]
var fails = 0
let only = CommandLine.arguments.count > 2 ? CommandLine.arguments[2] : ""
for (name, layout, keys, want, wantLayout) in cases where only.isEmpty || only.contains(name.first!) {
    key(0, cmd: true); key(51)            // ⌘A, delete
    select(layout); pause(200)
    let cleared = readText(); if !cleared.isEmpty { print("  not cleared:", cleared.debugDescription) }
    type(keys); pause(500)
    let got = readText(), lay = current()
    let ok = (got == want || got.prefix(1).lowercased() + got.dropFirst() == want) && lay == wantLayout; if !ok { fails += 1 }  // first letter: TextEdit auto-capitalizes
    print(ok ? "PASS" : "FAIL", name, "| got", got.debugDescription, "| layout", lay.replacingOccurrences(of: "com.apple.keylayout.", with: ""))
}
key(0, cmd: true); key(51); select("com.apple.keylayout.ABC")
print(fails == 0 ? "ALL PASS" : "\(fails) FAIL"); exit(fails == 0 ? 0 : 1)
