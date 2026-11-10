# ตัวรับรายงานปัญหา KeyFlip

บริการนี้รับรายงานจากแอป KeyFlip ผ่าน HTTPS ที่
`https://keyflip.sparkth.io/v1/reports` และเก็บข้อมูลใน SQLite ที่
`/var/lib/keyflip-feedback/reports.db` โดยใช้ Python standard library เท่านั้น

## ข้อมูลและความเป็นส่วนตัว

ระบบเก็บข้อความที่ผู้ใช้ส่ง รุ่นแอปและระบบปฏิบัติการ แพลตฟอร์ม โมเดล สถานะ
สถิติการใช้งานวันนี้ ภาษา ช่องทางติดต่อที่เลือกให้ไว้ และคำที่แปลงล่าสุดเมื่อ
ผู้ใช้เลือกแนบเท่านั้น ไม่บันทึกเนื้อหา request ลง log. IP ดิบไม่ถูกเก็บ
ระบบสร้าง SHA-256 จาก salt และ IP เพื่อใช้จำกัดการส่ง 5 ครั้งต่อชั่วโมงและ
20 ครั้งต่อวันต่อ IP. User-Agent ถูกเก็บไม่เกิน 200 ตัวอักษร

ผู้ดูแลเปิด `/admin` เพื่อดู 200 รายงานล่าสุด หรือ `/admin/reports.json` เพื่อรับ
ข้อมูล JSON. ทั้งสองเส้นทางต้องยืนยัน HTTP Basic ด้วยชื่อผู้ใช้ `admin` และ
รหัสผ่านใน `/etc/keyflip-feedback.env`; หน้า HTML แสดงผลแบบ escape และส่ง
`Cache-Control: no-store`.

## ติดตั้งบน VPS ที่ใช้ Traefik ใน Docker (เครื่องปัจจุบัน)

พอร์ต 80/443 ของเครื่องเป็นของ Traefik (`spark-virtual-reverse-proxy-1`) ตัวรับรายงานจึงรันเป็น container
ที่ Traefik มองเห็นจาก label เอง ไม่ต้องแก้ config ของ Traefik หรือ stack อื่น

1. เปิด Hostinger → VPS → **Web console** (ล็อกอินเป็น root)
2. รัน: `curl -fsSL https://raw.githubusercontent.com/ongkub/keyflip/main/server/deploy-docker.sh | bash`
3. สคริปต์ดึงไฟล์ไปไว้ที่ `/opt/keyflip-feedback`, สร้างรหัสผ่าน admin แบบสุ่มใน `/opt/keyflip-feedback/.env`
   (อ่านด้วย `grep ADMIN /opt/keyflip-feedback/.env`), หาเครือข่ายของ Traefik เอง, build และรัน container
   (ผู้ใช้ไม่มีสิทธิ์พิเศษ, root filesystem อ่านอย่างเดียว, จำกัด RAM 128 MB) แล้วรอจนสถานะ healthy
4. ใบรับรอง HTTPS ได้จาก Let's Encrypt อัตโนมัติภายในราว 1 นาที
5. ถอดออก: `cd /opt/keyflip-feedback && docker compose --env-file proxy.env down` (เติม `-v` ถ้าจะลบรายงานด้วย)

## ติดตั้งแบบ systemd + Caddy (เครื่องที่ไม่มี Docker)

1. จากโฟลเดอร์ repo บนเครื่องเจ้าของ คัดลอกไฟล์ขึ้นเครื่อง (Linux ที่มี systemd, Caddy,
   Python 3, curl และ Caddyfile ที่ `/etc/caddy/Caddyfile`):
   `scp -r server root@HOST:/tmp/keyflip-feedback`
2. รัน `ssh root@HOST 'bash /tmp/keyflip-feedback/deploy.sh'` สคริปต์ติดตั้งโปรแกรมและ unit,
   สร้าง environment file พร้อมรหัสผ่านสุ่มเฉพาะเมื่อไฟล์ยังไม่มี, เช็กว่าพอร์ต 8787 ว่าง,
   สำรอง Caddyfile ก่อนเพิ่มบล็อก `keyflip.sparkth.io` และคืนค่าเดิมให้อัตโนมัติถ้า
   `caddy validate` ไม่ผ่าน (ไม่แตะเว็บอื่นบนเครื่อง) แล้วจึง reload Caddy.
3. อ่านรหัสผ่านผู้ดูแลจาก `/etc/keyflip-feedback.env` ผ่าน SSH ที่ปลอดภัย
   (สคริปต์ไม่แสดงรหัสผ่านบนหน้าจอ).
4. ตรวจ `/healthz` และเปิด `https://keyflip.sparkth.io/admin` ในเบราว์เซอร์
   โดยใช้ user `admin`.

## อัปเดต

คัดลอกไฟล์ `feedback_server.py`, unit และ `Caddyfile.snippet` เวอร์ชันใหม่ไปยัง
โฟลเดอร์ `server/` บนเครื่อง แล้วรันสคริปต์ติดตั้งอีกครั้ง. Environment file
เดิมจะถูกรักษาไว้. สคริปต์ไม่สำรอง Caddyfile ดังนั้นตรวจ diff ก่อนใช้งานจริง.

## ลบการติดตั้ง

ในฐานะ root ให้รัน `systemctl disable --now keyflip-feedback.service`, ลบ
`/etc/systemd/system/keyflip-feedback.service` แล้ว `systemctl daemon-reload`.
นำบล็อก `keyflip.sparkth.io` ที่เพิ่มใน Caddyfile ออก ตรวจด้วย
`caddy validate --config /etc/caddy/Caddyfile` แล้ว reload Caddy. เมื่อต้องการ
ลบข้อมูลถาวร ให้ลบ `/var/lib/keyflip-feedback/` และ `/etc/keyflip-feedback.env`
ด้วยตนเองหลังสำรองข้อมูลที่ต้องเก็บแล้ว.
