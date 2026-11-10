/**
 * KeyFlip Landing Page JavaScript
 * Static, vanilla, zero-dependency.
 */

(function () {
  'use strict';

  /* ==========================================================================
     1. Motion Preferences
     ========================================================================== */
  function isReducedMotion() {
    return window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  }

  /* ==========================================================================
     2. Standalone Combining Mark Formatter
     ========================================================================== */
  const THAI_COMBINING_REGEX = /^[\u0E31\u0E34-\u0E3A\u0E47-\u0E4E]$/;

  function formatStandalone(char) {
    if (THAI_COMBINING_REGEX.test(char)) {
      return '\u25CC' + char;
    }
    return char;
  }

  /* ==========================================================================
     3. Keycap Row Rendering & Flip Animation
     ========================================================================== */
  const DEFAULT_TYPED = ['l', ';', 'y', 'l', 'f', 'u'];
  const DEFAULT_OTHER = ['ส', 'ว', 'ั', 'ส', 'ด', 'ี'];
  const DEFAULT_COMPOSED = 'สวัสดี';

  let flipTimeouts = [];

  function clearFlipTimeouts() {
    for (const t of flipTimeouts) {
      clearTimeout(t);
    }
    flipTimeouts = [];
  }

  function playFlip() {
    clearFlipTimeouts();

    const row = document.getElementById('hero-keycap-row');
    if (!row) return;

    const keys = row.querySelectorAll('.keycap');
    const composedWord = document.getElementById('hero-composed-word');

    // Reset visual state
    keys.forEach((key) => key.classList.remove('is-flipped'));
    if (composedWord) {
      composedWord.classList.remove('is-visible');
    }

    // Force reflow so removing is-flipped applies before new animation starts
    void row.offsetHeight;

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

    const finalKeyStart = Math.max(0, keys.length - 1) * stagger;
    const finalKeyLand = finalKeyStart + flipDuration;
    const wordRevealTime = finalKeyLand + 120;

    const wt = setTimeout(() => {
      if (composedWord) {
        composedWord.classList.add('is-visible');
      }
    }, wordRevealTime);
    flipTimeouts.push(wt);
  }

  function renderKeycaps(typedChars, otherChars, isMostlyThai) {
    const row = document.getElementById('hero-keycap-row');
    if (!row) return;

    row.replaceChildren();

    for (let i = 0; i < typedChars.length; i++) {
      const typed = typedChars[i];
      const other = otherChars[i] || '';

      const key = document.createElement('div');
      key.className = 'keycap';
      key.setAttribute('data-key', String(i));

      const inner = document.createElement('div');
      inner.className = 'keycap-inner';

      const frontFace = document.createElement('div');
      frontFace.className = 'keycap-face keycap-face-front';

      const backFace = document.createElement('div');
      backFace.className = 'keycap-face keycap-face-back';

      const latinChar = isMostlyThai ? other : typed;
      const rawThaiChar = isMostlyThai ? typed : other;
      const thaiChar = formatStandalone(rawThaiChar);

      if (isMostlyThai) {
        // Front: what was typed (Thai in Ink, Latin small in Slate)
        const frontLatin = document.createElement('span');
        frontLatin.className = 'legend-latin legend-sub';
        frontLatin.textContent = latinChar;

        const frontThai = document.createElement('span');
        frontThai.className = 'legend-thai legend-lead-ink';
        frontThai.textContent = thaiChar;

        frontFace.appendChild(frontLatin);
        frontFace.appendChild(frontThai);

        // Back: converted layout (Latin large in Legend Blue, Thai small in Slate)
        const backLatin = document.createElement('span');
        backLatin.className = 'legend-latin legend-lead-primary';
        backLatin.textContent = latinChar;

        const backThai = document.createElement('span');
        backThai.className = 'legend-thai legend-sub';
        backThai.textContent = thaiChar;

        backFace.appendChild(backLatin);
        backFace.appendChild(backThai);
      } else {
        // Front: what was typed (Latin in Ink, Thai small in Slate)
        const frontLatin = document.createElement('span');
        frontLatin.className = 'legend-latin legend-lead-ink';
        frontLatin.textContent = latinChar;

        const frontThai = document.createElement('span');
        frontThai.className = 'legend-thai legend-sub';
        frontThai.textContent = thaiChar;

        frontFace.appendChild(frontLatin);
        frontFace.appendChild(frontThai);

        // Back: converted layout (Thai large in Legend Blue, Latin small in Slate)
        const backLatin = document.createElement('span');
        backLatin.className = 'legend-latin legend-sub';
        backLatin.textContent = latinChar;

        const backThai = document.createElement('span');
        backThai.className = 'legend-thai legend-lead-primary';
        backThai.textContent = thaiChar;

        backFace.appendChild(backLatin);
        backFace.appendChild(backThai);
      }

      inner.appendChild(frontFace);
      inner.appendChild(backFace);
      key.appendChild(inner);
      row.appendChild(key);
    }
  }

  /* ==========================================================================
     4. Kedmanee Mapping & Demo Input
     ========================================================================== */
  // Minimal fallback map for default keys
  const fallbackPairs = [
    ['l', 'ส'], [';', 'ว'], ['y', 'ั'], ['f', 'ด'], ['u', 'ี'],
    ['v', 'อ'], ['p', 'ย'], ['k', 'า'], ['d', 'ก'], ['w', 'ไ'],
    ['x', 'ป'], ['b', 'ิ'], ['o', 'น'], ['-', 'ข'], ['h', '้'],
    ['m', 'ท'], ['t', 'ะ'], ['r', 'พ'], ['e', 'ำ'], ['c', 'แ']
  ];

  let latinToThai = new Map(fallbackPairs);
  let thaiToLatin = new Map(fallbackPairs.map(([l, t]) => [t, l]));

  function loadPairs() {
    fetch('assets/pairs.json')
      .then((res) => {
        if (!res.ok) throw new Error('HTTP ' + res.status);
        return res.json();
      })
      .then((pairs) => {
        latinToThai = new Map(pairs);
        thaiToLatin = new Map(pairs.map(([l, t]) => [t, l]));
      })
      .catch((err) => {
        console.warn('Could not load assets/pairs.json, using fallback map:', err);
      });
  }

  function initDemoInput() {
    const input = document.getElementById('demo-text-input');
    const composedWord = document.getElementById('hero-composed-word');
    if (!input || !composedWord) return;

    let debounceTimer = null;

    function handleInput() {
      const raw = input.value;
      if (!raw || raw.trim() === '') {
        renderKeycaps(DEFAULT_TYPED, DEFAULT_OTHER, false);
        composedWord.textContent = DEFAULT_COMPOSED;
        playFlip();
        return;
      }

      // Count Latin vs Thai characters
      const latinCount = (raw.match(/[a-zA-Z]/g) || []).length;
      const thaiCount = (raw.match(/[\u0E00-\u0E7F]/g) || []).length;
      const isMostlyThai = thaiCount > latinCount;

      // Map full text for composed word
      let convertedText = '';
      for (const char of raw) {
        if (isMostlyThai) {
          convertedText += thaiToLatin.has(char) ? thaiToLatin.get(char) : char;
        } else {
          convertedText += latinToThai.has(char) ? latinToThai.get(char) : char;
        }
      }
      composedWord.textContent = convertedText;

      // Extract first 8 typed characters for the keycap row
      const chars = Array.from(raw).slice(0, 8);
      const typedSlice = [];
      const otherSlice = [];

      for (const char of chars) {
        typedSlice.push(char);
        if (isMostlyThai) {
          otherSlice.push(thaiToLatin.has(char) ? thaiToLatin.get(char) : char);
        } else {
          otherSlice.push(latinToThai.has(char) ? latinToThai.get(char) : char);
        }
      }

      renderKeycaps(typedSlice, otherSlice, isMostlyThai);
      playFlip();
    }

    input.addEventListener('input', () => {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(handleInput, 250);
    });
  }

  /* ==========================================================================
     5. Initialization
     ========================================================================== */
  function init() {
    loadPairs();
    initDemoInput();

    // Trigger flip once 500 ms after document.fonts.ready
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

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
