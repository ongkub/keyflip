/**
 * KeyFlip Design Bench JavaScript
 * Controls keycap flip, Kedmanee typing demo, Thai clipping grid,
 * Thai word segmentation, GSAP SplitText check, and reduced-motion toggle.
 */

(function () {
  'use strict';

  /* ==========================================================================
     1. Reduced Motion Detection & Switch (Section 8)
     ========================================================================== */
  const reduceMotionCheckbox = document.getElementById('reduce-motion-checkbox');

  function isReducedMotion() {
    return document.documentElement.classList.contains('reduce-motion') ||
      (window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches);
  }

  function initReducedMotion() {
    if (!reduceMotionCheckbox) return;

    const mq = window.matchMedia ? window.matchMedia('(prefers-reduced-motion: reduce)') : null;
    if (mq && mq.matches) {
      document.documentElement.classList.add('reduce-motion');
      reduceMotionCheckbox.checked = true;
    }

    reduceMotionCheckbox.addEventListener('change', () => {
      if (reduceMotionCheckbox.checked) {
        document.documentElement.classList.add('reduce-motion');
      } else {
        document.documentElement.classList.remove('reduce-motion');
      }
    });

    if (mq && mq.addEventListener) {
      mq.addEventListener('change', (e) => {
        if (!reduceMotionCheckbox.matches(':focus')) {
          reduceMotionCheckbox.checked = e.matches;
        }
        if (e.matches) {
          document.documentElement.classList.add('reduce-motion');
        } else if (!reduceMotionCheckbox.checked) {
          document.documentElement.classList.remove('reduce-motion');
        }
      });
    }
  }

  /* ==========================================================================
     2. Keycap Row + Flip Animation (Section 2)
     ========================================================================== */
  let flipTimeouts = [];

  function clearFlipTimeouts() {
    for (const t of flipTimeouts) {
      clearTimeout(t);
    }
    flipTimeouts = [];
  }

  function playFlip() {
    clearFlipTimeouts();

    const row = document.getElementById('keycap-row');
    if (!row) return;

    const keys = row.querySelectorAll('.keycap');
    const composedWord = document.getElementById('hero-composed-word');

    // Reset initial visual state
    keys.forEach((key) => key.classList.remove('is-flipped'));
    if (composedWord) {
      composedWord.classList.remove('is-visible');
    }

    const reduced = isReducedMotion();
    const stagger = reduced ? 0 : 55;
    const flipDuration = reduced ? 150 : 380;

    keys.forEach((key, index) => {
      const delay = index * stagger;
      const t = setTimeout(() => {
        key.classList.add('is-flipped');
      }, delay);
      flipTimeouts.push(t);
    });

    // Composed word reveals 120 ms after the last key lands
    const finalKeyStart = (keys.length - 1) * stagger;
    const finalKeyLand = finalKeyStart + flipDuration;
    const wordRevealTime = finalKeyLand + 120;

    const wt = setTimeout(() => {
      if (composedWord) {
        composedWord.classList.add('is-visible');
      }
    }, wordRevealTime);
    flipTimeouts.push(wt);
  }

  function initKeycaps() {
    const replayBtn = document.getElementById('replay-flip-btn');
    if (replayBtn) {
      replayBtn.addEventListener('click', () => {
        clearFlipTimeouts();
        const row = document.getElementById('keycap-row');
        if (row) {
          const keys = row.querySelectorAll('.keycap');
          keys.forEach((k) => k.classList.remove('is-flipped'));
        }
        const composedWord = document.getElementById('hero-composed-word');
        if (composedWord) {
          composedWord.classList.remove('is-visible');
        }
        // Small settle timeout before running replay
        const rt = setTimeout(playFlip, 80);
        flipTimeouts.push(rt);
      });
    }

    // Play once after document.fonts.ready + 500 ms
    if (document.fonts && document.fonts.ready) {
      document.fonts.ready.then(() => {
        setTimeout(playFlip, 500);
      }).catch(() => {
        setTimeout(playFlip, 500);
      });
    } else {
      setTimeout(playFlip, 500);
    }
  }

  /* ==========================================================================
     3. Demo Input with Kedmanee Conversion (Section 3)
     ========================================================================== */
  let kedmaneeMap = new Map();

  function convertKedmanee(text) {
    if (!kedmaneeMap.size) return text;
    let out = '';
    for (const char of text) {
      out += kedmaneeMap.has(char) ? kedmaneeMap.get(char) : char;
    }
    return out;
  }

  function initDemoInput() {
    const input = document.getElementById('demo-text-input');
    const output = document.getElementById('demo-result-output');
    if (!input || !output) return;

    let debounceTimer = null;

    function updateOutput() {
      const converted = convertKedmanee(input.value);
      output.textContent = converted;
    }

    input.addEventListener('input', () => {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(updateOutput, 250);
    });

    // Fetch Kedmanee mapping pairs from ../../pairs.json
    fetch('../../pairs.json')
      .then((res) => {
        if (!res.ok) throw new Error('HTTP ' + res.status);
        return res.json();
      })
      .then((pairs) => {
        kedmaneeMap = new Map(pairs);
        if (input.value) {
          updateOutput();
        }
      })
      .catch((err) => {
        console.warn('Could not load pairs.json, using fallback Kedmanee map:', err);
        // Minimal fallback for standard demo keys
        kedmaneeMap = new Map([
          ['l', 'ส'], [';', 'ว'], ['y', 'ั'], ['f', 'ด'], ['u', 'ี'],
          ['v', 'อ'], ['p', 'ย'], ['k', 'า'], ['d', 'ก'], ['w', 'ไ'],
          ['x', 'ป'], ['b', 'ิ'], ['o', 'น'], ['-', 'ข'], ['h', '้']
        ]);
        if (input.value) {
          updateOutput();
        }
      });
  }

  /* ==========================================================================
     4. Thai Clipping Grid (Section 5)
     ========================================================================== */
  const fallbackTestStrings = [
    'ปั้น ปี่ ฟื้น ญี่ปุ่น ฎีกา ฐาน ฤๅษี น้ำ กำ เกียรติ ก็ ปู่ป้าน้าอา พี่น้อง ป่า ฝั่ง ฝ้าย ป่ำ ฟ้า ผู้หญิง ขี้ผึ้ง เสื้อผ้า กิ้งก่า เนื้อที่ ขี้เกียจ',
    'ฎุ ฏู ญู ฐุ ปุ่ม สู่ ครุฑ',
    'เก แก โก ใจ ไป เกือก แก้ว โต๊ะ ใหญ่ ไม้ ทำ น้ำ คำ',
    'เร็วๆ นี้ ฯลฯ กรุงเทพฯ',
    '๑๒๓ ราคา 0 บาท ขนาด 10 MB macOS 13 ขึ้นไป',
    'KeyFlip ตรวจจับคำที่คุณลืมสลับแป้นพิมพ์แล้วแก้เป็นภาษาไทยให้อัตโนมัติในพริบตาโดยไม่ต้องกดลบคำเดิมเพื่อพิมพ์ใหม่แม้แต่ตัวอักษรเดียว',
    'l;ylfu -> สวัสดี',
    'vpkdwxdbo-hk; -> อยากไปกินข้าว',
    'พิมพ์ผิดภาษา ไม่ต้องลบพิมพ์ใหม่',
    'ลืมเปลี่ยนภาษา ก็ไม่ต้องพิมพ์ใหม่',
    'เปิด KeyFlip บน macOS แล้วอนุญาต Accessibility ครั้งเดียว'
  ];

  function renderClippingGrid(strings) {
    const container = document.getElementById('clipping-grid-container');
    if (!container) return;

    container.innerHTML = '';

    const fonts = [
      { name: 'KF Display', cssVar: 'var(--font-display)' },
      { name: 'KF Text', cssVar: 'var(--font-text)' }
    ];
    const lineHeights = [1.1, 1.25, 1.5, 1.7];

    strings.forEach((str, strIdx) => {
      const group = document.createElement('div');
      group.className = 'clipping-string-group';

      const title = document.createElement('div');
      title.className = 'clipping-string-title';
      title.textContent = `ข้อความทดสอบ #${strIdx + 1}: ${str}`;
      group.appendChild(title);

      const grid = document.createElement('div');
      grid.className = 'clipping-cards-grid';

      fonts.forEach((font) => {
        lineHeights.forEach((lh) => {
          const card = document.createElement('div');
          card.className = 'clipping-card';

          const label = document.createElement('div');
          label.className = 'clipping-card-label';
          label.textContent = `${font.name} · lh: ${lh}`;

          const box = document.createElement('div');
          box.className = 'clipping-box';
          box.style.fontFamily = font.cssVar;
          box.style.lineHeight = String(lh);
          box.textContent = str;

          card.appendChild(label);
          card.appendChild(box);
          grid.appendChild(card);
        });
      });

      group.appendChild(grid);
      container.appendChild(group);
    });
  }

  /* ==========================================================================
     5. Thai Word Segmentation (Section 6)
     ========================================================================== */
  function renderSegmentation(paragraph) {
    const textEl = document.getElementById('segment-output-text');
    const badgeEl = document.getElementById('segment-count-badge');
    if (!textEl || !badgeEl) return;

    textEl.textContent = paragraph;

    if (typeof window.splitThaiWords === 'function') {
      window.splitThaiWords(textEl);
      const segs = textEl.querySelectorAll('.seg');
      badgeEl.textContent = `จำนวน segment ที่ตัดได้: ${segs.length} คำ`;
    } else {
      badgeEl.textContent = 'ไม่พบฟังก์ชัน window.splitThaiWords';
    }
  }

  function initTestStrings() {
    fetch('thai-test-strings.txt')
      .then((res) => {
        if (!res.ok) throw new Error('HTTP ' + res.status);
        return res.text();
      })
      .then((text) => {
        const lines = text
          .split('\n')
          .map((line) => line.trim())
          .filter((line) => line.length > 0 && !line.startsWith('#'));

        renderClippingGrid(lines.length ? lines : fallbackTestStrings);

        // Find the long paragraph for segmentation
        const longParagraph = lines.find((l) => l.includes('ตรวจจับคำ') || l.length > 80) ||
          fallbackTestStrings[5];
        renderSegmentation(longParagraph);
      })
      .catch((err) => {
        console.warn('Could not load thai-test-strings.txt, using fallbacks:', err);
        renderClippingGrid(fallbackTestStrings);
        renderSegmentation(fallbackTestStrings[5]);
      });
  }

  /* ==========================================================================
     6. GSAP SplitText Verification (Section 7)
     ========================================================================== */
  function initGSAPSplitText() {
    const statusEl = document.getElementById('splittext-status');
    const resultsEl = document.getElementById('splittext-results');
    if (!resultsEl) return;

    if (typeof window.gsap === 'undefined' || typeof window.SplitText === 'undefined') {
      if (statusEl) {
        statusEl.style.display = 'block';
        statusEl.textContent = 'ไม่สามารถโหลด GSAP 3.15.1 หรือ SplitText ได้จาก CDN (ไม่มีการเชื่อมต่อเครือข่าย หรือ CDN ภายนอกถูกบล็อก)';
      }
      return;
    }

    try {
      window.gsap.registerPlugin(window.SplitText);
    } catch (e) {
      console.warn('gsap.registerPlugin failed:', e);
    }

    const testCases = [
      {
        title: 'Headline — "ลืมเปลี่ยนภาษา ก็ไม่ต้องพิมพ์ใหม่"',
        text: 'ลืมเปลี่ยนภาษา ก็ไม่ต้องพิมพ์ใหม่',
        configs: [
          { name: 'type: "chars"', options: { type: 'chars' } },
          { name: 'type: "words"', options: { type: 'words' } },
          { name: 'type: "chars" + smartWrap: true', options: { type: 'chars', smartWrap: true } }
        ]
      },
      {
        title: 'Word — "น้ำปั่นญี่ปุ่น"',
        text: 'น้ำปั่นญี่ปุ่น',
        configs: [
          { name: 'type: "chars"', options: { type: 'chars' } },
          { name: 'type: "words"', options: { type: 'words' } },
          { name: 'type: "chars" + smartWrap: true', options: { type: 'chars', smartWrap: true } }
        ]
      }
    ];

    resultsEl.innerHTML = '';

    testCases.forEach((tc) => {
      tc.configs.forEach((cfg) => {
        const item = document.createElement('div');
        item.className = 'splittext-test-item';

        const label = document.createElement('div');
        label.className = 'splittext-label';
        label.textContent = `${tc.title} [${cfg.name}]`;

        const visual = document.createElement('div');
        visual.className = 'splittext-visual';
        visual.textContent = tc.text;

        item.appendChild(label);
        item.appendChild(visual);
        resultsEl.appendChild(item);

        try {
          const split = new window.SplitText(visual, cfg.options);
          const spans = visual.querySelectorAll('div, span');
          const spanTexts = Array.from(spans).map((s) => s.textContent);

          const log = document.createElement('div');
          log.className = 'splittext-log';
          log.textContent = `จำนวน spans: ${spans.length} | ผลลัพธ์: ${spanTexts.join(' | ')}`;
          item.appendChild(log);
        } catch (err) {
          const errDiv = document.createElement('div');
          errDiv.className = 'splittext-log';
          errDiv.textContent = `ข้อผิดพลาดขณะรัน SplitText: ${err.message}`;
          item.appendChild(errDiv);
        }
      });
    });
  }

  /* ==========================================================================
     7. DOMContentLoaded Bootstrapper
     ========================================================================== */
  function init() {
    initReducedMotion();
    initKeycaps();
    initDemoInput();
    initTestStrings();
    initGSAPSplitText();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
