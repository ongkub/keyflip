# KeyFlip Auto

แอปแถบเมนูสำหรับแก้คำที่พิมพ์ด้วยผังคีย์บอร์ดผิดระหว่างไทย Kedmanee กับ ABC/US

ติดตั้งด้วย `bash app/build.sh --install` แล้วเปิด `~/Applications/KeyFlipAuto.app` และอนุญาต Accessibility เมื่อ macOS ถาม กด Shift สองครั้งติดกันเพื่อย้อนการแก้ หรือกด Backspace ทันทีหลังแก้

ข้อมูลอยู่ที่ `~/Library/Application Support/KeyFlipAuto/`: `settings.json` เก็บการตั้งค่า, `never.txt` เก็บคำที่ย้อน, `events.jsonl` เก็บเฉพาะรายการแปลง/ย้อน และ `stats.json` เก็บจำนวนรายวัน โมเดลส่วนตัววางใน `models/`

ปิดจากเมนูแถบสถานะ > `ออก` ถอนการติดตั้งโดยปิดแอปแล้วลบ `~/Applications/KeyFlipAuto.app`; ลบโฟลเดอร์ข้อมูลเดียวกันหากต้องการล้างข้อมูลด้วย

## ทดสอบพิมพ์จริง (สำหรับนักพัฒนา)
`app/tools/kftest.swift` กดปุ่มจริง (keycode) ลง TextEdit แล้วอ่านข้อความกลับมาเทียบ 7 เคส: แปลงไทย/อังกฤษ,
คำที่ถูกต้องไม่ถูกแตะ, ย้อนด้วย Backspace และ Shift สองครั้ง. ต้องรันจากโปรแกรมที่มีสิทธิ์ Accessibility และควร
สำรอง/ล้าง `never.txt` แล้วเปิดแอปใหม่ก่อนรัน (คำที่ถูกย้อนในรอบก่อนจะไม่ถูกแปลงอีก)
```
swiftc app/tools/kftest.swift -o /tmp/kftest && : > /tmp/kf.txt && /tmp/kftest /tmp/kf.txt        # ทุกเคส
/tmp/kftest /tmp/kf.txt ED                                                                       # เฉพาะเคส E, D
```
