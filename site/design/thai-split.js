/**
 * Thai text segmentation helpers for KeyFlip.
 * Source: site/design/thai.md §7
 * Plain browser script (no modules, no build).
 */

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
  spoken.className = 'visually-hidden';
  spoken.textContent = text;
  el.replaceChildren(spoken, visual);
}

function splitThaiGraphemes(el) {
  const text = el.textContent;
  const segmenter = new Intl.Segmenter('th', { granularity: 'grapheme' });
  const visual = document.createElement('span');
  visual.setAttribute('aria-hidden', 'true');
  for (const { segment } of segmenter.segment(text)) {
    if (/\s/.test(segment)) {
      visual.appendChild(document.createTextNode(segment));
    } else {
      const span = document.createElement('span');
      span.className = 'seg';
      span.textContent = segment;
      visual.appendChild(span);
    }
  }
  const spoken = document.createElement('span');
  spoken.className = 'visually-hidden';
  spoken.textContent = text;
  el.replaceChildren(spoken, visual);
}

if (typeof window !== 'undefined') {
  window.splitThaiWords = splitThaiWords;
  window.splitThaiGraphemes = splitThaiGraphemes;
}
if (typeof globalThis !== 'undefined') {
  globalThis.splitThaiWords = splitThaiWords;
  globalThis.splitThaiGraphemes = splitThaiGraphemes;
}
