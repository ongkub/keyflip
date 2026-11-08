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
let fu=Fake { run, _ in run == "vpkd" ? yes : keep }; let tu=RunTracker(decider:fu,mapper:mapper,never:[]); for c in "vpkd" { _=tu.handle(.char(String(c))) }; check("backspace undo",tu.handle(.backspace) == .revert(delete:4,insert:"vpkd",switchTo:.en)); check("undo never",tu.never.contains("vpkd")); let n=fu.calls.count; _=tu.handle(.char("v")); check("no check after undo",fu.calls.count == n)
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
print("\(failures == 0 ? "All cases passed" : "\(failures) failed")"); exit(failures == 0 ? 0 : 1)
