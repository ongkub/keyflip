# Thai Typography & Text Animation Rules

Concrete rules for typesetting and animating Thai on the KeyFlip website.
Fonts: **KF Display** (Anuphan, loopless) for headings and keycaps; **KF Text** (IBM Plex Sans Thai Looped) for body and UI.

## 1. Document Setup
- Declare language on root: `<html lang="th">` so browsers apply Thai dictionary-based line breaking.
- Wrap inline English in `<span lang="en">` only when required for screen reader pronunciation; product names (`KeyFlip`, `macOS`, `Windows`) can remain unwrapped.
- Load order: import `fonts/fonts.css` first, then `design/tokens.css`. Never load fonts from third-party CDNs.
- Preload only two critical Thai woff2 subsets (paths relative to `site/`):
  ```html
  <link rel="preload" href="fonts/anuphan-400-700-thai.woff2" as="font" type="font/woff2" crossorigin>
  <link rel="preload" href="fonts/ibm-plex-sans-thai-looped-400-thai.woff2" as="font" type="font/woff2" crossorigin>
  ```
- Set `font-synthesis: none;` on all Thai elements to prevent browser-synthesized faux bold and faux italic.

## 2. Line-Height and Clipping
Thai stacks diacritics up to three levels above the baseline and descenders below (e.g. ปั้น, ญี่ปุ่น, ฎีกา, ฐุ).
- **Measured in the bench:** KF Text (Plex Looped) lines collide at 1.1 and are tight at 1.25 — keep body at 1.7 and
  never below 1.5 for KF Text; KF Display (Anuphan) is clean at 1.25.
- **Minimum line-heights:** body text minimum `1.6` (KeyFlip standard is `1.7`), headings minimum `1.25`, buttons/inputs/keycaps minimum `1.3`. Never go below `1.2` anywhere.
- **Clipping avoidance:** Elements using `overflow: hidden`, `-webkit-line-clamp`, or tight fixed heights will clip tone marks and below-base vowels. Provide extra `padding-block` or eliminate tight overflow bounds. Validate against `site/design/thai-test-strings.txt`.

## 3. Spacing and Case
- **No letter-spacing:** Never add `letter-spacing` (tracking) to Thai text (`letter-spacing: 0` or omitted). It detaches combining marks and ruins legibility. Only allowed on pure Latin strings, and even then avoided.
- **No text-transform:** Never use `text-transform: uppercase` or `lowercase`; Thai has no letter case and transforms corrupt mixed Latin/Thai passages.
- **No faux or true italic:** Do not use italic styling for Thai emphasis. Use weight 600 (`var(--type-body-strong)`) or the Highlighter background token (`var(--color-highlight)`).

## 4. Line Breaking & Punctuation
Thai words are written continuously without spaces; browsers break lines using built-in dictionary segmentation.
- Never use `word-break: break-all`, `overflow-wrap: anywhere`, or `hyphens` on Thai text.
- Use `text-wrap: balance;` on headings and `text-wrap: pretty;` on body paragraphs.
- Insert `<wbr>` or U+200B (zero-width space) to hint break points in long compounds or product names that break awkwardly.
- Use U+00A0 (no-break space, `&nbsp;`) between numbers and units (e.g. `10&nbsp;MB`, `0&nbsp;บาท`) and inside short fixed phrases.
- In Thai, a standard space marks a clause or sentence boundary, never a word boundary. Never insert spaces purely to force wrapping.

## 5. Mixed Thai/English & Keycaps
- Insert a single standard space before and after Latin words and numbers inside Thai sentences (e.g. "เปิด KeyFlip บน macOS 13").
- Always use standard Arabic numerals (`0-9`), not Thai digits (`๑, ๒, ๓`), in product copy.
- Keyboard keys and typed mistyped gibberish must be rendered as keycap components or within `<kbd>` elements (e.g. `<kbd>l;ylfu</kbd>`), never formatted as plain monospace inline text.

## 6. Standalone Marks
- **Anuphan (KF Display) has no U+25CC glyph**: it renders ◌ั as empty boxes. `--font-display` therefore lists
  'KF Text' second so the browser draws the dotted circle from IBM Plex Looped. Keep that order.
- Combining vowels and tone marks shown alone without a consonant (in diagrams, legend keys, or explanations) must sit on U+25CC DOTTED CIRCLE: `◌ั`, `◌ี`, `◌้`, `◌่`. Never render a bare combining mark.

## 7. Thai Text Animation
- **Never split Thai text** with `str.split('')`, `Array.from()`, `for...of`, or per code point. Doing so detaches vowels and tone marks and isolates leading vowels (เ, แ, โ, ใ, ไ).
- Split by word using `new Intl.Segmenter('th', { granularity: 'word' })` (Baseline 2024; dictionary-based, may be imperfect) or split by line. Grapheme granularity is reserved only when a per-cluster effect is strictly needed.
- **Tested 2026-11-10 in `design/bench.html` (GSAP 3.15.1, Chromium):** SplitText `type: "chars"` splits Thai into
  grapheme clusters, so tone marks and upper/lower vowels stay on their consonant (`ลื | ม | เ | ป | ลี่`, `น้ำ` whole);
  leading vowels (เ แ โ ใ ไ) become their own pieces. `type: "words"` splits only at spaces, so a Thai phrase stays
  one chunk (`ลืมเปลี่ยนภาษา`). Use `splitThaiWords()` for word effects; SplitText chars is acceptable but rarely
  looks good on Thai.
- Use this helper (shipped as `design/thai-split.js`, a plain script, no modules). Screen readers get the full
  text from a visually hidden copy; the animated pieces are hidden from them. Do not rely on `aria-label` on a
  `<p>`/`<div>`/`<span>`: it is not reliably announced on generic elements.
  ```javascript
  function splitThaiWords(el) {
    const text = el.textContent;
    const segmenter = new Intl.Segmenter('th', { granularity: 'word' });
    const visual = document.createElement('span');
    visual.setAttribute('aria-hidden', 'true');
    for (const { segment, isWordLike } of segmenter.segment(text)) {
      if (isWordLike) {
        const span = document.createElement('span');
        span.className = 'seg';
        span.textContent = segment;
        visual.appendChild(span);
      } else {
        visual.appendChild(document.createTextNode(segment));
      }
    }
    const spoken = document.createElement('span');
    spoken.className = 'visually-hidden';   /* standard clip-rect visually-hidden class */
    spoken.textContent = text;
    el.replaceChildren(spoken, visual);
  }
  ```

## 8. Copy Conventions
- **Tone:** Sentence-style Thai, polite, plain, and conversational. Do not use polite particles ("ครับ/ค่ะ") in UI labels; use them sparingly in FAQ answers if needed.
- **Action-oriented labels:** Button labels must specify the concrete action (e.g. "ดาวน์โหลดสำหรับ Mac"), not vague generic phrases ("เริ่มเลย" or "คลิกที่นี่").
- **Loanwords:** Avoid English loanwords when a natural, common Thai equivalent exists. Retain established tech terminology (แอป, ดาวน์โหลด, macOS, Windows).

## 9. Quality Checklist
1. Is `<html lang="th">` declared on the root document? [ ]
2. Are `fonts/fonts.css` and `design/tokens.css` imported in order, with the two Thai subsets preloaded? [ ]
3. Is `font-synthesis: none;` set to prevent faux bold/italic on Thai glyphs? [ ]
4. Are body line-heights ≥ 1.6 (KeyFlip uses 1.7) and headings ≥ 1.25, with no clipped tone marks? [ ]
5. Are `letter-spacing` and `text-transform` strictly avoided on Thai text? [ ]
6. Is emphasis expressed via weight 600 or the highlighter token instead of italics? [ ]
7. Are normal spaces placed around Latin words and numbers in Thai sentences, using Arabic numerals? [ ]
8. Do standalone Thai combining marks sit on U+25CC DOTTED CIRCLE (`◌ั`, `◌ี`)? [ ]
9. Are text animations split by word (`Intl.Segmenter`) or line, with a visually hidden full-text copy and `aria-hidden` pieces? [ ]
10. Have all typography styles and tight containers passed inspection against `site/design/thai-test-strings.txt`? [ ]
