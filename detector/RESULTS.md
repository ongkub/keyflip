# Detector evaluation

Chosen config: `{"ctx_delta": 0.0, "dictionary_bonus": 0.0, "min_len": 2, "orthography_convert": 1.8, "orthography_keep": 1.8, "threshold": 4.0}`

## Dev threshold trade-off

| threshold | ctx_delta | FP/1000 | recall % |
|---:|---:|---:|---:|
| -2 | 0 | 247.239 | 99.39 |
| -2 | 0.25 | 245.245 | 99.39 |
| -1 | 0 | 190.184 | 99.37 |
| -1 | 0.25 | 192.485 | 99.37 |
| 0 | 0 | 172.546 | 99.36 |
| 0 | 0.25 | 171.472 | 99.36 |
| 0.5 | 0 | 150.153 | 99.36 |
| 0.5 | 0.25 | 146.472 | 99.36 |
| 1 | 0 | 134.969 | 99.34 |
| 1 | 0.25 | 134.969 | 99.33 |
| 1.5 | 0 | 132.362 | 99.31 |
| 1.5 | 0.25 | 129.908 | 99.31 |
| 2 | 0 | 120.245 | 99.31 |
| 2 | 0.25 | 119.479 | 99.31 |
| 3 | 0 | 102.914 | 99.28 |
| 3 | 0.25 | 101.994 | 99.28 |
| 4 | 0 | 82.209 | 99.13 |
| 4 | 0.25 | 87.730 | 99.13 |

## dev

| intended | runs | FP/1000 | sentences with FP % | recall % | median k | p90 k | within 4 % | avg ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| th | 3584 | 148.996 | 21.03 | 99.30 | 2 | 5 | 88.36 | 0.1319 |
| en | 2936 | 0.681 | 0.06 | 98.91 | 2.0 | 3 | 98.60 | 0.0728 |

### th worst false positives

- run='รหัสการเข้าถึงได้ถูกบันทึกไปยังการตั้งค่าบัญชีของคุณแล้ว'; prefix='รห'; prev=th; score=15.111
- run='รหัสผ่าน'; prefix='รห'; prev=th; score=15.111
- run='รหัสผ่านเหล่านี้จะถูกแจ้งขอเมื่อเปิดใช้การเชื่อมข้อมูลพวงกุญแจกับคอมพิวเตอร์อื่นเพิ่มเติม'; prefix='รห'; prev=th; score=15.111
- run='ฟื้นฟู'; prefix='ฟื'; prev=th; score=11.596
- run='ฟื้นฟู'; prefix='ฟื'; prev=th; score=11.596
- run='ทั้งหมด'; prefix='ทั'; prev=en; score=11.262
- run='ทั้งนี้'; prefix='ทั'; prev=en; score=11.262
- run='ทั่วโลกใน'; prefix='ทั'; prev=en; score=11.262
- run='ทั่วไป'; prefix='ทั'; prev=th; score=11.262
- run='สื่อ'; prefix='สื'; prev=th; score=9.988
- run='สภาพแวดล้อม'; prefix='สภ'; prev=None; score=9.372
- run='ฉัน'; prefix='ฉั'; prev=None; score=9.220
- run='สัปดาห์'; prefix='สั'; prev=None; score=8.942
- run='สัปดาห์'; prefix='สั'; prev=None; score=8.942
- run='สัปดาห์'; prefix='สั'; prev=None; score=8.942
- run='สัปดาห์'; prefix='สั'; prev=None; score=8.942
- run='สัญญาณชีพตอนกลางคืน'; prefix='สั'; prev=th; score=8.942
- run='สัญญาณชีพตอนกลางคืน'; prefix='สั'; prev=th; score=8.942
- run='สั้น'; prefix='สั'; prev=th; score=8.942
- run='กับ'; prefix='กั'; prev=en; score=8.613
- run='กังหันลม'; prefix='กั'; prev=None; score=8.613
- run='กันนาดา'; prefix='กั'; prev=None; score=8.613
- run='กันสาดหน้าต่าง'; prefix='กั'; prev=None; score=8.613
- run='กับ'; prefix='กั'; prev=None; score=8.613
- run='กับ'; prefix='กั'; prev=None; score=8.613

### th misses

- run='ถึง'; garbled="57'"; prev=en; score=0.000
- run='ถึง'; garbled="57'"; prev=en; score=0.000
- run='แทรก'; garbled='cmid'; prev=en; score=3.495
- run='สำเนา'; garbled='legok'; prev=None; score=3.478
- run='สำเนา'; garbled='legok'; prev=None; score=3.478
- run='แนะนำ'; garbled='cotoe'; prev=None; score=2.698
- run='ถึง'; garbled="57'"; prev=th; score=0.000
- run='ณ'; garbled='I'; prev=th; score=-1.470
- run='ชม'; garbled='=,'; prev=None; score=0.000
- run='น'; garbled='o'; prev=th; score=2.573
- run='น'; garbled='o'; prev=None; score=2.573
- run='ชม'; garbled='=,'; prev=None; score=0.000
- run='ใบ'; garbled='.['; prev=th; score=0.000
- run='ใบ'; garbled='.['; prev=th; score=0.000
- run='หนา'; garbled='sok'; prev=en; score=1.100
- run='ก'; garbled='d'; prev=None; score=0.145
- run='ถึง'; garbled="57'"; prev=th; score=0.000
- run='ฮ'; garbled='V'; prev=th; score=0.432
- run='ก'; garbled='d'; prev=None; score=0.145
- run='ฮ'; garbled='V'; prev=th; score=0.432
- run='กราฟ'; garbled='dika'; prev=None; score=0.704
- run='กราฟิก'; garbled='dikabd'; prev=None; score=3.370
- run='กราฟเส้น'; garbled='dikaglho'; prev=None; score=3.743
- run='สั้น'; garbled='lyho'; prev=th; score=3.421
- run='กำปั้น'; garbled='dexyho'; prev=None; score=3.652

### en worst false positives

- run='iur'; prefix='iu'; prev=en; score=10.010
- run='sinveni'; prefix='sinv'; prev=en; score=6.578

### en misses

- run='a'; garbled='ฟ'; prev=en; score=7.594
- run='d'; garbled='ก'; prev=th; score=-0.145
- run='x'; garbled='ป'; prev=None; score=3.423
- run='A'; garbled='ฤ'; prev=None; score=19.374
- run='D'; garbled='ฏ'; prev=None; score=36.689
- run='E'; garbled='ฎ'; prev=None; score=2.499
- run='pe'; garbled='ยำ'; prev=en; score=1.132
- run='K'; garbled='ษ'; prev=None; score=31.998
- run='L'; garbled='ศ'; prev=None; score=1.505
- run='a'; garbled='ฟ'; prev=en; score=7.594
- run='M'; garbled='?'; prev=None; score=0.000
- run='MMM'; garbled='???'; prev=None; score=0.000
- run='R'; garbled='ฑ'; prev=None; score=3.572
- run='iur'; garbled='รีพ'; prev=en; score=-4.659
- run='Vit'; garbled='ฮระ'; prev=en; score=3.683
- run='S'; garbled='ฆ'; prev=None; score=5.809
- run='T'; garbled='ธ'; prev=None; score=0.157
- run='X'; garbled=')'; prev=None; score=0.000
- run='X'; garbled=')'; prev=None; score=0.000
- run='X'; garbled=')'; prev=None; score=0.000
- run='d'; garbled='ก'; prev=None; score=-0.145
- run='dd'; garbled='กก'; prev=None; score=-1.023
- run='MMM'; garbled='???'; prev=en; score=0.000
- run='f'; garbled='ด'; prev=None; score=1.347
- run='g'; garbled='เ'; prev=None; score=23.403

## test

| intended | runs | FP/1000 | sentences with FP % | recall % | median k | p90 k | within 4 % | avg ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| th | 2811 | 209.178 | 30.83 | 98.01 | 3 | 4 | 91.14 | 0.0898 |
| en | 5162 | 3.100 | 0.64 | 96.55 | 2.0 | 3 | 96.18 | 0.0346 |

### th worst false positives

- run='ทั้งหมด'; prefix='ทั'; prev=None; score=11.262
- run='ทั้งหมด'; prefix='ทั'; prev=en; score=11.262
- run='ทั้งวัน'; prefix='ทั'; prev=None; score=11.262
- run='ทั้งหมด'; prefix='ทั'; prev=None; score=11.262
- run='ทั้งหมดตั้งแต่แรก'; prefix='ทั'; prev=en; score=11.262
- run='สื่อสารสองทิศทางพร้อมกัน'; prefix='สื'; prev=en; score=9.988
- run='สื่อสารสองทิศทางสลับกัน'; prefix='สื'; prev=en; score=9.988
- run='สื่อสารสองทิศทางพร้อมกัน'; prefix='สื'; prev=en; score=9.988
- run='สื่อสารสองทิศทางสลับกัน'; prefix='สื'; prev=en; score=9.988
- run='สื่อสารสองทิศทางพร้อมกัน'; prefix='สื'; prev=en; score=9.988
- run='สื่อสารสองทิศทางสลับกัน'; prefix='สื'; prev=en; score=9.988
- run='พื้นที่ไม่เพียงพอ'; prefix='พื'; prev=th; score=9.968
- run='สัปดาห์'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์ที่ผ่านมา'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์ที่ผ่านมา'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์ที่ผ่านมา'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์ที่ผ่านมา'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์ที่ผ่านมา'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์ที่ผ่านมา'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์ที่ผ่านมา'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์ที่ผ่านมา'; prefix='สั'; prev=th; score=8.942
- run='สัปดาห์'; prefix='สั'; prev=th; score=8.942

### th misses

- run='เธรด'; garbled='gTif'; prev=None; score=3.702
- run='ถึง'; garbled="57'"; prev=en; score=0.000
- run='ถึง'; garbled="57'"; prev=th; score=0.000
- run='ลบ'; garbled=']['; prev=th; score=0.000
- run='เกิน'; garbled='gdbo'; prev=th; score=3.772
- run='ใบ'; garbled='.['; prev=th; score=0.000
- run='ใบ'; garbled='.['; prev=th; score=0.000
- run='แก้โดย'; garbled='cdhFfp'; prev=None; score=3.430
- run='แนะนำ'; garbled='cotoe'; prev=None; score=2.698
- run='กำนดไฟล์'; garbled='deofwa]N'; prev=en; score=2.724
- run='ถึง'; garbled="57'"; prev=None; score=0.000
- run='น'; garbled='o'; prev=None; score=2.573
- run='ดีแคร์'; garbled='fuc8iN'; prev=th; score=2.572
- run='มม'; garbled=',,'; prev=en; score=0.000
- run='มม'; garbled=',,'; prev=en; score=0.000
- run='มม'; garbled=',,'; prev=None; score=0.000
- run='มม'; garbled=',,'; prev=en; score=0.000
- run='มม'; garbled=',,'; prev=None; score=0.000
- run='มม'; garbled=',,'; prev=en; score=0.000
- run='มม'; garbled=',,'; prev=en; score=0.000
- run='มม'; garbled=',,'; prev=None; score=0.000
- run='มม'; garbled=',,'; prev=None; score=0.000
- run='มม'; garbled=',,'; prev=en; score=0.000
- run='มม'; garbled=',,'; prev=en; score=0.000
- run='มม'; garbled=',,'; prev=en; score=0.000

### en worst false positives

- run='i5'; prefix='i5'; prev=en; score=15.326
- run='i5'; prefix='i5'; prev=en; score=15.326
- run='i9'; prefix='i9'; prev=en; score=7.608
- run='i9'; prefix='i9'; prev=en; score=7.608
- run='i9'; prefix='i9'; prev=en; score=7.608
- run='-v'; prefix='-v'; prev=None; score=6.261
- run='-v'; prefix='-v'; prev=None; score=6.261
- run='-v'; prefix='-v'; prev=None; score=6.261
- run='-v'; prefix='-v'; prev=None; score=6.261
- run='-vv'; prefix='-v'; prev=None; score=6.261
- run='E5'; prefix='E5'; prev=en; score=6.079
- run='E5'; prefix='E5'; prev=en; score=6.079
- run='E5'; prefix='E5'; prev=en; score=6.079
- run='E5'; prefix='E5'; prev=en; score=6.079
- run='[user(s)]'; prefix='[u'; prev=en; score=5.246
- run='-k'; prefix='-k'; prev=None; score=4.222

### en misses

- run='-I'; garbled='ขณ'; prev=None; score=2.828
- run='-I'; garbled='ขณ'; prev=None; score=2.828
- run='-I'; garbled='ขณ'; prev=None; score=2.828
- run='-h'; garbled='ข้'; prev=None; score=3.022
- run='-h'; garbled='ข้'; prev=None; score=3.022
- run='-h'; garbled='ข้'; prev=None; score=3.022
- run='-h'; garbled='ข้'; prev=None; score=3.022
- run='-i'; garbled='ขร'; prev=None; score=1.434
- run='-i'; garbled='ขร'; prev=None; score=1.434
- run='-i'; garbled='ขร'; prev=None; score=1.434
- run='-i'; garbled='ขร'; prev=None; score=1.434
- run='-i'; garbled='ขร'; prev=None; score=1.434
- run='-j'; garbled='ข่'; prev=None; score=-3.735
- run='-k'; garbled='ขา'; prev=None; score=-4.222
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433
- run='-o'; garbled='ขน'; prev=None; score=-3.433

## chat

| intended | runs | FP/1000 | sentences with FP % | recall % | median k | p90 k | within 4 % | avg ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| th | 360 | 186.111 | 38.96 | 95.00 | 3.0 | 4 | 85.83 | 0.0577 |
| en | 348 | 0.000 | 0.00 | 96.55 | 2.0 | 3 | 96.26 | 0.0324 |

### th worst false positives

- run='งั้นเอาตามนี้เลยค่ะ'; prefix='งั'; prev=None; score=11.305
- run='สัปดาห์นี้เพิ่มขึ้น'; prefix='สั'; prev=en; score=8.942
- run='สั้นๆ'; prefix='สั'; prev=en; score=8.942
- run='กับทีม'; prefix='กั'; prev=en; score=8.613
- run='กันแป๊บนึงได้ไหม'; prefix='กั'; prev=en; score=8.613
- run='กับลูกค้าตอนบ่ายสอง'; prefix='กั'; prev=en; score=8.613
- run='กับทีมตอนเที่ยง'; prefix='กั'; prev=en; score=8.613
- run='กับ'; prefix='กั'; prev=en; score=8.613
- run='ให้พี่ก่อนเที่ยงได้มั้ย'; prefix='ให'; prev=en; score=8.485
- run='ให้'; prefix='ให'; prev=en; score=8.485
- run='ให้แล้วนะ'; prefix='ให'; prev=en; score=8.485
- run='ให้ในแชต'; prefix='ให'; prev=en; score=8.485
- run='ให้หน่อยนะคะ'; prefix='ให'; prev=en; score=8.485
- run='ให้ลูกค้าดูได้เลยค่ะ'; prefix='ให'; prev=en; score=8.485
- run='ให้ครบ'; prefix='ให'; prev=en; score=8.485
- run='ให้หน่อย'; prefix='ให'; prev=en; score=8.485
- run='ให้แล้วนะ'; prefix='ให'; prev=en; score=8.485
- run='ให้ผู้บริหาร'; prefix='ให'; prev=en; score=8.485
- run='ให้ด้วย'; prefix='ให'; prev=en; score=8.485
- run='ให้ทาง'; prefix='ให'; prev=en; score=8.485
- run='ให้หน่อย'; prefix='ให'; prev=en; score=8.485
- run='ให้ด้วยค่ะ'; prefix='ให'; prev=en; score=8.485
- run='ให้หน่อย'; prefix='ให'; prev=en; score=8.485
- run='ให้ดูก่อน'; prefix='ให'; prev=en; score=8.485
- run='ใหญ่ด้วยครับ'; prefix='ให'; prev=en; score=8.485

### th misses

- run='ทำ'; garbled='me'; prev=None; score=-0.305
- run='งบ'; garbled="'["; prev=None; score=0.000
- run='ค่ะ'; garbled='8jt'; prev=th; score=1.257
- run='นะ'; garbled='ot'; prev=en; score=-3.511
- run='งบ'; garbled="'["; prev=None; score=0.000
- run='นัดคุย'; garbled='oyf86p'; prev=None; score=3.304
- run='นะคะ'; garbled='ot8t'; prev=en; score=1.840
- run='นะ'; garbled='ot'; prev=th; score=-3.511
- run='ค่ะ'; garbled='8jt'; prev=th; score=1.257
- run='ค่ะ'; garbled='8jt'; prev=en; score=1.257
- run='ค่ะ'; garbled='8jt'; prev=en; score=1.257
- run='นะ'; garbled='ot'; prev=en; score=-3.511
- run='ค่ะ'; garbled='8jt'; prev=en; score=1.257
- run='แผน'; garbled='czo'; prev=None; score=3.841
- run='ค่ะ'; garbled='8jt'; prev=en; score=1.257
- run='นะคะ'; garbled='ot8t'; prev=en; score=1.840
- run='เย้'; garbled='gph'; prev=th; score=-1.603
- run='แป๊บนึงนะ'; garbled="cxU[o7'ot"; prev=en; score=1.622

### en worst false positives

- none

### en misses

- run='crop'; garbled='แพนย'; prev=th; score=3.811
- run='group'; garbled='เพนีย'; prev=th; score=3.130
- run='me'; garbled='ทำ'; prev=en; score=0.305
- run='I'; garbled='ณ'; prev=None; score=1.470
- run='I'; garbled='ณ'; prev=None; score=1.470
- run='me'; garbled='ทำ'; prev=en; score=0.305
- run='a'; garbled='ฟ'; prev=en; score=7.594
- run='I'; garbled='ณ'; prev=None; score=1.470
- run='I'; garbled='ณ'; prev=None; score=1.470
- run='I'; garbled='ณ'; prev=None; score=1.470
- run='a'; garbled='ฟ'; prev=en; score=7.594
- run='a'; garbled='ฟ'; prev=en; score=7.594

## Known weaknesses

Corpus is limited to strings exposed by local macOS localization tables; short prefixes and names absent from training remain ambiguous. Run boundaries are heuristic and dictionary coverage depends on `/usr/share/dict/words`. Chat data is an illustrative fixed sample, not a representative user study.
