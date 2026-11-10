Adapted from emilkowalski/skills (MIT, © 2026 Emil Kowalski) and vercel-labs/web-interface-guidelines (MIT, © 2025 Vercel Labs).

# Motion Rules

Concrete motion standards for the KeyFlip website. KeyFlip is a static, Thai-first landing page with exactly one orchestrated moment: the hero keycap flip.

## 1. When to Animate

- Motion must serve a clear purpose: provide feedback on user action, clarify a state change, or direct attention once.
- Elements viewed frequently animate less or not at all. Keyboard shortcuts and focus moves get no animation; the one exception is the hero demo, where typing replays the flip (see §5).
- No entrance animations on page sections (no fade-ins, no slide-ups per section, no staggered reveals of static content).
- No hover elevation lifts on cards, buttons, or keycaps.
- No parallax effects, scroll hijacking, or smooth-scroll libraries (Lenis is strictly prohibited).

## 2. Timing and Easing Table

| Interaction | Duration | Easing Curve | Purpose / Rule |
| --- | --- | --- | --- |
| Press feedback (`:active`) | 80 ms | `ease-out` / `cubic-bezier(0.2, 0.8, 0.2, 1)` | Tactile key sink on click/press |
| Small UI state change | 150–200 ms | `cubic-bezier(0.23, 1, 0.32, 1)` | Tooltip, badge, or chip transition |
| Larger reveals / hero flip | 200–380 ms | `cubic-bezier(0.2, 0.8, 0.2, 1)` | Hero keycap flip (380 ms) and word reveal (200 ms) |
| Exits and dismissals | 120–150 ms | `cubic-bezier(0.4, 0, 1, 1)` or fast ease-out | Exits must always be faster than entrances |
| Stagger interval | 30–80 ms | Fixed or decelerating | 55 ms left-to-right between hero keycaps |

- **Easing rules**:
  - Entering elements: strong ease-out (`cubic-bezier(0.23, 1, 0.32, 1)` or `cubic-bezier(0.2, 0.8, 0.2, 1)`).
  - Exiting elements: fast ease-out or accelerating exit; never linger.
  - Moving or morphing on screen: `cubic-bezier(0.77, 0, 0.175, 1)`.
  - Never use `linear` except for continuous progress indicators.
  - Never use `ease-in` on UI entrances; it delays initial feedback and makes interactions feel sluggish.

## 3. What May Be Animated

- Animate `transform` and `opacity` only. Both run on the compositor thread and skip layout and paint.
- `filter` (such as blur) may be used sparingly (e.g. `blur(2px)` during crossfades; never exceed 20px).
- Never animate layout properties: `width`, `height`, `top`, `left`, `right`, `bottom`, `margin`, or `padding`.
- Apply `will-change: transform` only during active animation or to resolve a measured 1 px subpixel shift; never leave it permanent.
- All animations must be interruptible without visual jump or broken state.

## 4. Reduced Motion

- Decorative motion must be wrapped inside `@media (prefers-reduced-motion: no-preference)`.
- JavaScript must evaluate `window.matchMedia('(prefers-reduced-motion: reduce)')` and listen for changes. When GSAP is used, wrap timelines in `gsap.matchMedia()`.
- Reduced-motion fallback: immediate final state or an opacity crossfade of 150 ms or less without translation, rotation, or scaling.

## 5. The KeyFlip Flip (The Only Orchestrated Moment)

The hero section features the single orchestrated animation on the entire site:

- **Faces**: Six keycaps display Latin legends `l`, `;`, `y`, `l`, `f`, `u`, which flip to Thai legends `ส`, `ว`, `◌ั`, `ส`, `ด`, `◌ี` (combining marks drawn on U+25CC dotted circle).
- **Trigger**: Starts 500 ms after `document.fonts.ready` resolves. Plays once per page load.
- **Keycap flip animation**:
  - Keycap row perspective: `perspective: 600px`.
  - Individual keycap rotation: `rotateX(0deg → -180deg)` over 380 ms.
  - Both Latin and Thai keycap faces have `backface-visibility: hidden`.
  - Easing: `cubic-bezier(0.2, 0.8, 0.2, 1)`.
  - Stagger: 55 ms per key, running left to right across the 6 keys.
- **Composed word reveal**:
  - 120 ms after the final key lands, the resolved Thai word "สวัสดี" appears below on a Highlighter background (`var(--color-highlight)`).
  - Transition: `opacity: 0 → 1` and `transform: scale(0.98) → scale(1)` over 200 ms with `ease-out`.
- **Demo input interaction**: Typing in the demo input replays the flip for the visitor's text, debounced at 250 ms, with at most one replay sequence in flight.
- **Reduced motion handling**: No 3D rotation (`rotateX`). Faces swap via a 150 ms crossfade. The composed word appears with opacity only (no scale change).
- **Implementation**: Vanilla CSS transitions toggled by classes and minimal vanilla JavaScript. If GSAP is used, vendor the library locally; never load from a CDN in production.

## 6. Press Feedback

- Buttons and keycaps: On `:active`, the surface moves down 2 px (`transform: translateY(2px)`) and the bottom edge shrinks from 3 px to 1 px (`--key-edge` 3 px to `--key-edge-pressed` 1 px) over 80 ms.
- Hover states (`:hover`): Change color or edge color only. Never apply vertical elevation lifts or expanding drop shadows.

## 7. Scroll

- Version 1 includes zero scroll-triggered animations.
- Any future scroll effect must use native CSS scroll-driven animations inside `@supports (animation-timeline: view())` and `@media (prefers-reduced-motion: no-preference)`. Never hijack scrolling.

## 8. Performance Budget

- Frame target: Smooth 60 fps (16.7 ms budget) on low-end laptops and mobile devices.
- Layout stability: Cumulative Layout Shift (CLS) = 0 from motion. Reserve container dimensions before motion triggers.
- Main thread: Handlers remain lightweight to keep Interaction to Next Paint (INP) under 100 ms.
- Visibility: Pause or skip animation updates when the tab or element is off-screen.

## 9. Review Checklist

Before shipping any motion code, verify:

- [ ] Does this animation serve feedback, state clarification, or one-time orientation?
- [ ] Are animated properties strictly restricted to `transform` and `opacity`?
- [ ] Are layout properties (`width`, `height`, `top`, `left`, `margin`) completely avoided?
- [ ] Do UI animations stay under 300 ms (or exactly 380 ms for the hero flip)?
- [ ] Are exits faster than entrances?
- [ ] Is `ease-in` completely absent on UI entrances?
- [ ] Is `transition: all` avoided in favor of explicit property names?
- [ ] Is `scale(0)` avoided, starting from `scale(0.95)` or higher instead?
- [ ] Is `@media (prefers-reduced-motion: no-preference)` honored with an instant or ≤150 ms crossfade fallback?
- [ ] Are hover effects gated behind `@media (hover: hover) and (pointer: fine)` without elevation lift?
- [ ] Is the animation fully interruptible without visual jumps or timer pile-ups?
- [ ] Does the page maintain CLS = 0 and avoid dropped frames on low-end hardware?
