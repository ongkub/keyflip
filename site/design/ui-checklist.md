Adapted from emilkowalski/skills (MIT, © 2026 Emil Kowalski) and vercel-labs/web-interface-guidelines (MIT, © 2025 Vercel Labs).

# UI Quality Checklist

Concise yes/no checks for the KeyFlip static landing page. Verify each requirement before shipping template, content, or styling changes.

## 1. Accessibility

- [ ] Does the root element specify `lang="th"`?
- [ ] Is a visible, keyboard-accessible "Skip to content" link present as the first focusable element?
- [ ] Are headings structured in hierarchical order (`<h1>` through `<h3>`) with zero skipped levels?
- [ ] Do all interactive elements show a clear, unobscured `:focus-visible` ring (never `outline: none` without replacement)?
- [ ] Do hit targets measure at least 24 px on fine pointer devices and at least 44 px on touch devices?
- [ ] Does body and caption text meet at least 4.5:1 contrast against the background (e.g. Slate #5E6678 on Deck #F7F8FA)?
- [ ] Are real `<button>` elements used for actions and native `<a>` elements with `href` used for navigation?
- [ ] Do informative images include meaningful `alt` text, while decorative assets carry `aria-hidden="true"`?

## 2. Forms (Demo Input)

- [ ] Does the demo input provide an accessible name via associated `<label>` or `aria-label`?
- [ ] Are auto-corrective features explicitly disabled: `autocomplete="off"`, `autocapitalize="off"`, `autocorrect="off"`, and `spellcheck="false"`?
- [ ] Is `inputmode="text"` declared to prevent unintended mobile keyboard layouts?
- [ ] Can visitors freely Tab in and out of the demo input without focus trapping?
- [ ] Is pasting text into the demo input supported without interception or blocking?

## 3. Layout Stability

- [ ] Do all images and video containers specify explicit `width` and `height` attributes (or CSS `aspect-ratio`)?
- [ ] Does the hero keycap row occupy reserved layout dimensions before fonts finish loading or animations trigger?
- [ ] Are self-hosted fonts loaded with `font-display: swap` and matched fallbacks to avoid visible layout shifts?
- [ ] Does Cumulative Layout Shift (CLS) remain strictly at 0 during page load, font rendering, and keycap flipping?

## 4. Content and Copy

- [ ] Is copy formatted in sentence case without arbitrary mid-sentence capitalization?
- [ ] Is the single ellipsis character "…" (U+2026) used instead of three individual dots "..."?
- [ ] Is a non-breaking space (`&nbsp;`) placed between numbers and units (e.g., `10&nbsp;MB`, `13&nbsp;ขึ้นไป`)?
- [ ] Do button and link labels clearly state the resulting action (e.g., "ดาวน์โหลดสำหรับ Mac") instead of generic text like "คลิกที่นี่"?
- [ ] Are Thai typography guidelines followed (line-height ≥ 1.6 on body text; zero letter-spacing on Thai)?

## 5. Links and Downloads

- [ ] Do all installer download links clearly state the supported operating system and file size?
- [ ] Do external links (e.g., GitHub repository) inform the user that navigation leaves the site?
- [ ] On mobile devices where desktop binaries cannot run, does the interface show mobile-specific guidance (copy link, share to LINE) instead of dead-end download buttons?

## 6. Performance

- [ ] Are zero third-party scripts, analytics trackers, or external CDN assets loaded on the page?
- [ ] Are all raster images served in AVIF and WebP formats with intrinsic dimensions provided?
- [ ] Does the total JavaScript payload for the page stay under 30 KB gzipped (excluding an optional locally-vendored GSAP build)?
- [ ] Are fonts loaded locally from `site/fonts/` with zero third-party font network requests?
