// สลับข้อความที่พิมพ์ผิดภาษา ระหว่างแป้น ABC กับ Thai (เกษมณีของ Mac)
// ใช้กับ: osascript -l JavaScript convert.js < input.txt
// ทดสอบ: osascript -l JavaScript convert.js --test
ObjC.import('Foundation');

// [แป้น ABC, ตัวที่แป้นเดียวกันพิมพ์ใน layout Thai] — อ่านจาก layout จริงของเครื่องด้วย dumplayout.swift
const PAIRS = [["a", "ฟ"], ["A", "ฤ"], ["s", "ห"], ["S", "ฆ"], ["d", "ก"], ["D", "ฏ"], ["f", "ด"], ["F", "โ"], ["h", "้"], ["H", "็"], ["g", "เ"], ["G", "ฌ"], ["z", "ผ"], ["Z", "("], ["x", "ป"], ["X", ")"], ["c", "แ"], ["C", "ฉ"], ["v", "อ"], ["V", "ฮ"], ["b", "ิ"], ["B", "ฺ"], ["q", "ๆ"], ["Q", "๐"], ["w", "ไ"], ["W", "\""], ["e", "ำ"], ["E", "ฎ"], ["r", "พ"], ["R", "ฑ"], ["y", "ั"], ["Y", "ํ"], ["t", "ะ"], ["T", "ธ"], ["1", "ๅ"], ["!", "+"], ["2", "/"], ["@", "๑"], ["3", "_"], ["#", "๒"], ["4", "ภ"], ["$", "๓"], ["6", "ุ"], ["^", "ู"], ["5", "ถ"], ["%", "๔"], ["=", "ช"], ["+", "๙"], ["9", "ต"], ["(", "๖"], ["7", "ึ"], ["&", "฿"], ["-", "ข"], ["_", "๘"], ["8", "ค"], ["*", "๕"], ["0", "จ"], [")", "๗"], ["]", "ล"], ["}", ","], ["o", "น"], ["O", "ฯ"], ["u", "ี"], ["U", "๊"], ["[", "บ"], ["{", "ฐ"], ["i", "ร"], ["I", "ณ"], ["p", "ย"], ["P", "ญ"], ["l", "ส"], ["L", "ศ"], ["j", "่"], ["J", "๋"], ["'", "ง"], ["\"", "."], ["k", "า"], ["K", "ษ"], [";", "ว"], [":", "ซ"], ["\\", "ฃ"], ["|", "ฅ"], [",", "ม"], ["<", "ฒ"], ["/", "ฝ"], ["?", "ฦ"], ["n", "ื"], ["N", "์"], ["m", "ท"], ["M", "?"], [".", "ใ"], [">", "ฬ"], ["`", "-"], ["~", "%"]];

const EN2TH = new Map();
const TH2EN = new Map();
for (const [en, th] of PAIRS) {
  EN2TH.set(en, th);
  if (!TH2EN.has(th)) TH2EN.set(th, en);
}

const isThai = (c) => c >= 'ก' && c <= '๛';
const isLatin = (c) => (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z');

// ตัวอักษรไทยกลับเป็นแป้นอังกฤษ ตัวอังกฤษกลับเป็นไทย
// ตัวเลข/เครื่องหมาย ดูจากตัวอักษรที่อยู่ใกล้ที่สุดในคำเดียวกัน
// คำที่ไม่มีตัวอักษรเลย (เช่น 2024, :), 10.5) ปล่อยไว้ตามเดิม
function convertToken(token) {
  const chars = Array.from(token);
  const script = chars.map((c) => (isThai(c) ? 'th' : isLatin(c) ? 'en' : null));
  if (!script.some(Boolean)) return token;
  return chars
    .map((c, i) => {
      let s = script[i];
      for (let j = i - 1; !s && j >= 0; j--) s = script[j];
      for (let j = i + 1; !s && j < chars.length; j++) s = script[j];
      const table = s === 'th' ? TH2EN : EN2TH;
      return table.has(c) ? table.get(c) : c;
    })
    .join('');
}

function convert(text) {
  return text.split(/(\s+)/).map((t) => (/^\s*$/.test(t) ? t : convertToken(t))).join('');
}

function selfTest() {
  const cases = [
    ['vpkdwxdbo-hk;', 'อยากไปกินข้าว'],
    ['vpdkwxdbo-hk;', 'อยกาไปกินข้าว'], // แปลงตามแป้นตรงตัว คำที่สลับแป้นกันจะไม่ถูกแก้ให้
    ['l;ylfu', 'สวัสดี'],
    ['้ำสสน', 'hello'],
    ['็ำสสน', 'Hello'],
    ['ไ้ำพำฦ', 'where?'],
    ['l;ylfu ้ำสสน 2024 :)', 'สวัสดี hello 2024 :)'],
    ['l;ylfu\n้ำสสน', 'สวัสดี\nhello'],
    ['2024', '2024'],
    ['  vpkdw  ', '  อยากไ  '],
  ];
  const out = [];
  let fail = 0;
  for (const [input, want] of cases) {
    const got = convert(input);
    const back = convert(got);
    const ok = got === want && back === input;
    if (!ok) fail++;
    out.push(`${ok ? 'PASS' : 'FAIL'}  ${JSON.stringify(input)} -> ${JSON.stringify(got)} (want ${JSON.stringify(want)}, back ${JSON.stringify(back)})`);
  }
  out.push(`${cases.length - fail}/${cases.length} passed, table=${PAIRS.length} keys`);
  return out.join('\n');
}

function run(argv) {
  if (argv && argv[0] === '--test') return selfTest();
  const data = $.NSFileHandle.fileHandleWithStandardInput.readDataToEndOfFile;
  const text = $.NSString.alloc.initWithDataEncoding(data, $.NSUTF8StringEncoding).js || '';
  const result = $.NSString.alloc.initWithUTF8String(convert(text));
  $.NSFileHandle.fileHandleWithStandardOutput.writeData(result.dataUsingEncoding($.NSUTF8StringEncoding));
}
