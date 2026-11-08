import Foundation

final class Fake: Decider {
    var calls: [(String, Lang?, Bool)] = []; var body: (String, Bool) -> Decision
    init(_ body: @escaping (String, Bool) -> Decision) { self.body = body }
    func decide(run: String, prev: Lang?, complete: Bool) -> Decision { calls.append((run,prev,complete)); return body(run,complete) }
}
let root = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
let mapper = try! TableMapper(pairsJSON: root.appendingPathComponent("pairs.json"))
let yes = Decision(convert:true,score:0.9,candidate:"อยาก",target:.th), keep = Decision(convert:false,score:0,candidate:"",target:.en)
var failures = 0
func check(_ name:String,_ condition:Bool) { print("\(condition ? "PASS" : "FAIL") \(name)"); if !condition { failures += 1 } }
func test1() -> (Fake,RunTracker) { let f=Fake { run, _ in run == "vpkd" ? yes : keep }; let t=RunTracker(decider:f,mapper:mapper,never:[]); for c in ["v","p","k"] { check("typing prefix \(c)",t.handle(.char(c)) == .pass) }; check("typing convert",t.handle(.char("d")) == .replace(delete:3,insert:"อยาก",switchTo:.th)); check("converted no more check",t.handle(.char("ไ")) == .pass); check("converted space pass",t.handle(.space) == .pass); check("single decision",f.calls.count == 4); return (f,t) }
let (f1,t1)=test1()
let fu=Fake { run, _ in run == "vpkd" ? yes : keep }; let tu=RunTracker(decider:fu,mapper:mapper,never:[]); for c in "vpkd" { _=tu.handle(.char(String(c))) }; check("backspace undo",tu.handle(.backspace) == .revert(delete:4,insert:"vpkd",switchTo:.en)); check("one backspace undo does not block",!tu.never.contains("vpkd")); let n=fu.calls.count; _=tu.handle(.char("v")); check("no check after undo",fu.calls.count == n)
let f3=Fake { run, complete in run == "l;ylfu" && complete ? Decision(convert:true,score:1,candidate:"สวัสดี",target:.th) : keep }; let t3=RunTracker(decider:f3,mapper:mapper,never:[]); for c in "l;ylfu" { _=t3.handle(.char(String(c))) }; check("complete conversion",t3.handle(.space) == .replace(delete:6,insert:"สวัสดี ",switchTo:.th)); check("shift undo",t3.handle(.undo) == .revert(delete:7,insert:"l;ylfu ",switchTo:.en))
let f4=Fake { _,_ in keep }; let t4=RunTracker(decider:f4,mapper:mapper,never:["vpkd"]); for c in "vpkd" { _=t4.handle(.char(String(c))) }; _=t4.handle(.space); check("never skip",f4.calls.isEmpty)
let f5=Fake { _,_ in keep }; let t5=RunTracker(decider:f5,mapper:mapper,never:[]); for c in ["a","b","ก"] { _=t5.handle(.char(c)) }; check("mixed script skip",f5.calls.count == 2)
let f6=Fake { _,_ in keep }; let t6=RunTracker(decider:f6,mapper:mapper,never:[]); _=t6.handle(.char("v")); _=t6.handle(.char("p")); _=t6.handle(.reset); _=t6.handle(.char("k")); _=t6.handle(.char("d")); check("reset starts fresh",f6.calls.map{$0.0} == ["v","vp","k","kd"] && f6.calls[2].1 == nil)
let f7=Fake { _,_ in keep }; let t7=RunTracker(decider:f7,mapper:mapper,never:[]); for c in "hello" { _=t7.handle(.char(String(c))) }; _=t7.handle(.space); _=t7.handle(.char("ท")); check("prev language",f7.calls.last?.1 == .en)
let f8=Fake { _,_ in keep }; let t8=RunTracker(decider:f8,mapper:mapper,never:[]); for c in "vpx" { _=t8.handle(.char(String(c))) }; _=t8.handle(.backspace); _=t8.handle(.char("k")); check("backspace removes scalar",f8.calls.last?.0 == "vpk")
let f9=Fake { _,_ in keep }; let t9=RunTracker(decider:f9,mapper:mapper,never:[]); _=t9.handle(.char("a")); _=t9.handle(.returnKey); check("return incomplete no call",f9.calls.count == 1 && f9.calls.allSatisfy{ !$0.2 }); _=t9.handle(.char("ท")); check("return resets prev",f9.calls.last?.1 == nil)
let f10=Fake { _,_ in keep }; let t10=RunTracker(decider:f10,mapper:mapper,never:[]); for _ in 0..<21 { _=t10.handle(.char("a")) }; check("typing cap",f10.calls.count == 20)
check("map to Thai",mapper.convert("vpkdwxdbo-hk;",to:.th) == "อยากไปกินข้าว"); check("map to ABC",mapper.convert("อยากไปกินข้าว",to:.en) == "vpkdwxdbo-hk;")
let fg=Fake { run,_ in run == "ที่ก" ? Decision(convert:true,score:1,candidate:"ข",target:.en) : keep }; let tg=RunTracker(decider:fg,mapper:mapper,never:[],deleteMode:"grapheme"); _=tg.handle(.char("ที่")); check("grapheme count",tg.handle(.char("ก")) == .replace(delete:1,insert:"ข",switchTo:.en))
let fs=Fake { run,_ in run == "ที่ก" ? Decision(convert:true,score:1,candidate:"ข",target:.en) : keep }; let ts=RunTracker(decider:fs,mapper:mapper,never:[],deleteMode:"scalar"); _=ts.handle(.char("ที่")); check("scalar count",ts.handle(.char("ก")) == .replace(delete:3,insert:"ข",switchTo:.en))
let f13=Fake { run,_ in run == "vpkd" ? yes : keep }; let t13=RunTracker(decider:f13,mapper:mapper,never:[]); for c in "vpkd" { _=t13.handle(.char(String(c))) }; _=t13.handle(.char("ไ")); check("shift undo after continued typing",t13.handle(.undo) == .revert(delete:5,insert:"vpkdw",switchTo:.en)); check("continued typing never word",t13.never.contains("vpkdw"))
let f14=Fake { run,_ in run == "vpkd" ? yes : keep }; let t14=RunTracker(decider:f14,mapper:mapper,never:[]); for c in "vpkd" { _=t14.handle(.char(String(c))) }; _=t14.handle(.char("ไ")); check("backspace after continued typing passes",t14.handle(.backspace) == .pass); check("backspace shrinks converted run",t14.handle(.undo) == .revert(delete:4,insert:"vpkd",switchTo:.en))
let f15=Fake { run,_ in run == "vpkd" ? yes : keep }; let t15=RunTracker(decider:f15,mapper:mapper,never:[]); for c in "vpkd" { _=t15.handle(.char(String(c))) }; _=t15.handle(.space); check("undo typing conversion after space",t15.handle(.undo) == .revert(delete:5,insert:"vpkd ",switchTo:.en))
let f16=Fake { run,_ in run == "vpkd" ? yes : keep }; let t16=RunTracker(decider:f16,mapper:mapper,never:[]); for c in "vpkd" { _=t16.handle(.char(String(c))) }; _=t16.handle(.space); _=t16.handle(.char("ก")); check("typing after converted space expires undo",t16.handle(.undo) == .pass)
var convertedWords: (String,String)?; var revertedWords: (String,String)?
let f17=Fake { run,_ in run == "vpkd" ? yes : keep }; let t17=RunTracker(decider:f17,mapper:mapper,never:[]); t17.onEvent = { event in switch event { case .converted(_,_,_,_,_,_,let word,let result): convertedWords = (word,result); case .reverted(_,_,_,let word,let result): revertedWords = (word,result); case .runEnded: break } }; for c in "vpkd" { _=t17.handle(.char(String(c))) }; _=t17.handle(.char("ไ")); _=t17.handle(.undo); check("conversion event words",convertedWords?.0 == "vpkd" && convertedWords?.1 == "อยาก"); check("reversion event words",revertedWords?.0 == "อยากไ" && revertedWords?.1 == "vpkdw")
// 18: after a typing conversion + space, the NEXT word is still checked and the converted run is counted
var ended18: [String] = []
let f18=Fake { run,_ in run == "vpkd" ? yes : keep }; let t18=RunTracker(decider:f18,mapper:mapper,never:[]); t18.onEvent = { if case .runEnded(let l) = $0 { ended18.append(l) } }
for c in "vpkd" { _=t18.handle(.char(String(c))) }; _=t18.handle(.space); let n18=f18.calls.count; _=t18.handle(.char("l")); check("next word checked after typing conversion + space", f18.calls.count == n18 + 1 && ended18 == ["th"])
// 19: undo after space-ended run, then the next word is checked
let f19=Fake { run,_ in run == "vpkd" ? yes : keep }; let t19=RunTracker(decider:f19,mapper:mapper,never:[]); for c in "vpkd" { _=t19.handle(.char(String(c))) }; _=t19.handle(.space); _=t19.handle(.undo); let n19=f19.calls.count; _=t19.handle(.char("x")); check("next word checked after undo of ended run", f19.calls.count == n19 + 1 && f19.calls.last?.0 == "x")
// 20: the same word undone with Backspace twice is blocked; 21: shift-undo blocks at once; 22: clearNever
let f20=Fake { run,_ in run == "vpkd" ? yes : keep }; let t20=RunTracker(decider:f20,mapper:mapper,never:[])
for _ in 0..<2 { _=t20.handle(.reset); for c in "vpkd" { _=t20.handle(.char(String(c))) }; _=t20.handle(.backspace) }
check("second backspace undo blocks", t20.never.contains("vpkd"))
let f21=Fake { run,_ in run == "vpkd" ? yes : keep }; let t21=RunTracker(decider:f21,mapper:mapper,never:[]); for c in "vpkd" { _=t21.handle(.char(String(c))) }; _=t21.handle(.undo)
check("shift undo blocks", t21.never.contains("vpkd")); t21.clearNever(); check("clearNever empties", t21.never.isEmpty)
print("\(failures == 0 ? "All cases passed" : "\(failures) failed")"); exit(failures == 0 ? 0 : 1)
