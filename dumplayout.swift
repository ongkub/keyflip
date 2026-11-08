import Carbon
import Foundation

func layoutData(_ id: String) -> UnsafePointer<UCKeyboardLayout> {
    let filter = [kTISPropertyInputSourceID as String: id] as CFDictionary
    guard let list = TISCreateInputSourceList(filter, true)?.takeRetainedValue() as? [TISInputSource],
          let src = list.first,
          let raw = TISGetInputSourceProperty(src, kTISPropertyUnicodeKeyLayoutData) else {
        fatalError("no layout \(id)")
    }
    let data = Unmanaged<CFData>.fromOpaque(raw).takeUnretainedValue() as Data
    let p = UnsafeMutablePointer<UInt8>.allocate(capacity: data.count)
    data.copyBytes(to: p, count: data.count)
    return UnsafeRawPointer(p).assumingMemoryBound(to: UCKeyboardLayout.self)
}

func char(_ l: UnsafePointer<UCKeyboardLayout>, _ code: UInt16, _ shift: Bool) -> String {
    var dead: UInt32 = 0
    var buf = [UniChar](repeating: 0, count: 4)
    var len = 0
    let mods: UInt32 = shift ? UInt32(shiftKey >> 8) & 0xFF : 0
    UCKeyTranslate(l, code, UInt16(kUCKeyActionDown), mods, UInt32(LMGetKbdType()),
                   OptionBits(kUCKeyTranslateNoDeadKeysBit), &dead, 4, &len, &buf)
    return String(utf16CodeUnits: buf, count: len)
}

let en = layoutData("com.apple.keylayout.ABC")
let th = layoutData("com.apple.keylayout.Thai")
var pairs: [[String]] = []
for code in UInt16(0)..<UInt16(51) {
    for s in [false, true] {
        let e = char(en, code, s), t = char(th, code, s)
        guard e.unicodeScalars.count == 1, t.unicodeScalars.count >= 1,
              let es = e.unicodeScalars.first, es.value > 0x20, es.value < 0x7F else { continue }
        pairs.append([e, t])
    }
}
let json = try! JSONSerialization.data(withJSONObject: pairs, options: [])
print(String(data: json, encoding: .utf8)!)
