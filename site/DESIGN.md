---
version: alpha
name: KeyFlip Keycap
description: Thai-first landing site for KeyFlip. The visual language is the Thai dual-legend keycap, and the one memorable moment is a word flipping from the wrong layout to the right one.
colors:
  primary: "#2347C6"
  primary-pressed: "#1A3699"
  on-primary: "#FFFFFF"
  ink: "#1B2233"
  slate: "#5E6678"
  neutral: "#F7F8FA"
  keycap: "#FFFFFF"
  keycap-edge: "#C9D1DD"
  rule: "#DDE2EA"
  highlight: "#FFE066"
typography:
  display:
    fontFamily: Anuphan
    fontSize: 56px
    fontWeight: 700
    lineHeight: 1.25
    letterSpacing: 0em
  headline:
    fontFamily: Anuphan
    fontSize: 32px
    fontWeight: 600
    lineHeight: 1.35
    letterSpacing: 0em
  title:
    fontFamily: Anuphan
    fontSize: 22px
    fontWeight: 600
    lineHeight: 1.45
    letterSpacing: 0em
  body:
    fontFamily: IBM Plex Sans Thai Looped
    fontSize: 18px
    fontWeight: 400
    lineHeight: 1.7
    letterSpacing: 0em
  body-strong:
    fontFamily: IBM Plex Sans Thai Looped
    fontSize: 18px
    fontWeight: 600
    lineHeight: 1.7
    letterSpacing: 0em
  small:
    fontFamily: IBM Plex Sans Thai Looped
    fontSize: 15px
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: 0em
  legend-thai:
    fontFamily: Anuphan
    fontSize: 28px
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: 0em
  legend-latin:
    fontFamily: Anuphan
    fontSize: 15px
    fontWeight: 500
    lineHeight: 1.2
    letterSpacing: 0em
rounded:
  control: 8px
  key: 10px
  panel: 18px
spacing:
  base: 4px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 48px
  xxl: 72px
  section: 112px
  text-measure: 680px
  page-max: 1120px
components:
  page:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.ink}"
    typography: "{typography.body}"
  fixed-word:
    backgroundColor: "{colors.highlight}"
    textColor: "{colors.ink}"
  faq-rule:
    backgroundColor: "{colors.rule}"
    height: 1px
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.body-strong}"
    rounded: "{rounded.key}"
    padding: 14px 24px
  button-primary-pressed:
    backgroundColor: "{colors.primary-pressed}"
    textColor: "{colors.on-primary}"
  button-secondary:
    backgroundColor: "{colors.keycap}"
    textColor: "{colors.ink}"
    typography: "{typography.body-strong}"
    rounded: "{rounded.key}"
    padding: 14px 24px
  keycap:
    backgroundColor: "{colors.keycap}"
    textColor: "{colors.ink}"
    rounded: "{rounded.key}"
    size: 64px
  keycap-edge:
    backgroundColor: "{colors.keycap-edge}"
    rounded: "{rounded.key}"
    height: 3px
  keycap-legend-thai:
    textColor: "{colors.primary}"
    typography: "{typography.legend-thai}"
  keycap-legend-latin:
    textColor: "{colors.slate}"
    typography: "{typography.legend-latin}"
  demo-input:
    backgroundColor: "{colors.keycap}"
    textColor: "{colors.ink}"
    typography: "{typography.title}"
    rounded: "{rounded.control}"
    padding: 16px 20px
  panel:
    backgroundColor: "{colors.keycap}"
    textColor: "{colors.ink}"
    rounded: "{rounded.panel}"
    padding: 32px
---

# KeyFlip Keycap

Read this file before changing anything under `site/`. Also read `design/motion.md` (motion and UI rules),
`design/thai.md` (Thai typography and Thai text animation) and use `design/tokens.css` for every value below.
Where this file and a task brief disagree, the brief wins; where this file is silent, choose the quieter option.

## Overview

**Subject.** KeyFlip fixes words typed in the wrong keyboard layout: someone forgets to switch to Thai, types
`l;ylfu`, and KeyFlip turns it into "สวัสดี" and switches the input source. The most characteristic object in this
world is the **Thai keycap with two legends**: Latin top-left, Thai bottom-right. Every key on a Thai keyboard already
carries both answers; KeyFlip just picks the right one. That keycap is the visual language of the whole site.

**Audience.** Thai, mostly non-technical computer users who switch between Thai and English all day (office work,
school, LINE and Facebook chats). They read on laptops and phones, often arrive from a video or a friend's link, and
are wary of an app that "reads the keyboard".

**Job of the page, in order.** (1) Show the fix in three seconds. (2) Earn trust: what KeyFlip sees, what it sends,
that the code is open. (3) Get the right download to the right computer. (4) Walk people through the first-open
warning and the permission step so they do not give up.

**Personality.** Calm, plain, quietly clever. A small daily annoyance, fixed. Light humour is allowed in the demo
(everyone in Thailand recognises `l;ylfu`), never in the privacy or install sections.

**Where the boldness goes.** One place only: the hero, where a row of real-looking keycaps flips from the Latin
legends to the Thai word. Everything else is quiet, left-aligned and generous.

## Colors

A cool, light keyboard-deck world with one ink, one legend colour and one highlighter.

- **Legend Blue (#2347C6), `primary`:** the colour of the Thai legend on our keycaps and of the main action. It means
  "the right layout". Use for primary buttons, Thai legends on keycaps, links. Pressed state #1A3699.
- **Deep Ink (#1B2233), `ink`:** all body text and headlines. Never use pure black or #111.
- **Wrong-layout Slate (#5E6678), `slate`:** Latin legends on keycaps, the gibberish before it is fixed, captions.
  Passes 4.5:1 on the page background; do not lighten it.
- **Deck (#F7F8FA), `neutral`:** page background. Cool, not cream.
- **Keycap (#FFFFFF) with Keycap Edge (#C9D1DD):** key tops, buttons, panels, inputs. The edge colour is the visible
  bottom edge of a key, not a shadow.
- **Rule (#DDE2EA):** hairlines between FAQ items and table rows only.
- **Highlighter (#FFE066), `highlight`:** only behind the word that was just fixed (like a text selection), and
  behind the currently relevant install step. Always with Deep Ink text on top. Never as a button or large fill.

No gradients, no glass, no blobs, no tinted section bands. Dark mode is out of scope for v1.

## Typography

Two Thai families with a clear, very Thai distinction: **loopless for display, looped for reading.**

- **Anuphan** (Cadson Demak, OFL), CSS family `var(--font-display)`: headlines and keycap legends. Contemporary
  loopless Thai with matching Latin. Weights 600–700 for headings, 500 for small Latin legends.
- **IBM Plex Sans Thai Looped** (IBM, OFL), `var(--font-text)`: body, buttons, inputs, FAQ. Looped Thai is easier for
  general readers at text sizes.
- `ui-monospace` only for literal machine strings: SHA-256 checksums and terminal commands. Not for labels.

Rules: never add letter-spacing to Thai; never `text-transform: uppercase` anything; body is 18px with line-height
1.7 because Thai stacks marks above and below; headings stay at line-height 1.25 or more so tone marks are never
clipped. Use `text-wrap: balance` on headings and keep text blocks within `text-measure` (680px). Font files are
self-hosted in `site/fonts/` (Thai + Latin subsets); never load fonts from a third-party CDN.

## Layout

Left-aligned throughout (centred text only inside keycaps and buttons). One column of reading width inside a wider
page; the hero is the only two-column section on desktop.

```
DESKTOP (page-max 1120px)
┌───────────────────────────────────────────────────────────────────┐
│ KeyFlip                                     ดาวน์โหลด   โค้ดบน GitHub │
│                                                                   │
│ พิมพ์ผิดภาษา                  ┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐   │
│ ไม่ต้องลบพิมพ์ใหม่              │l   ││;   ││y   ││l   ││f   ││u   │   │
│                              │  ส ││  ว ││ ◌ั ││  ส ││  ด ││ ◌ี │   │  keys flip once
│ [ลองพิมพ์ตรงนี้ ............]    └────┘└────┘└────┘└────┘└────┘└────┘   │
│ [ดาวน์โหลดสำหรับ Mac]  สำหรับ Windows ก็มี                              │
│ macOS 13 ขึ้นไป ขนาด 10 MB ฟรีและเปิดโค้ด                               │
├───────────────────────────────────────────────────────────────────┤
│ How it knows (short prose + one diagram, not cards)               │
│ What KeyFlip sees and sends (plain two-column list in a panel)    │
│ First open, step by step (OS tabs, numbered steps, screenshots)   │
│ Questions (FAQ, hairline rules)                                   │
│ Footer: source, report a problem, licence, version + checksum     │
└───────────────────────────────────────────────────────────────────┘

MOBILE (16px gutters)
  headline → keycap row (scaled, 6 keys) → demo input
  → "เปิดหน้านี้บนคอมพิวเตอร์เพื่อดาวน์โหลด" + copy link + share to LINE   (no dead-end download button)
  → sections stacked
```

Spacing follows a 4px base (`spacing` tokens). Sections are separated by `section` (112px desktop, 72px mobile),
not by background bands or dividers.

## Elevation & Depth

Depth comes from the **keycap edge**: key tops, buttons and inputs sit on a solid 3px bottom edge in Keycap Edge
(#C9D1DD), like a physical key. Pressing a button moves the top down 2px and shrinks the edge to 1px. There are no
soft drop shadows anywhere, and nothing floats.

## Shapes

Three radii by role, never one radius for everything: `control` 8px (inputs, small chips), `key` 10px (keycaps and
buttons, because buttons are keys), `panel` 18px (the privacy panel and install-step frames). No pills, no circles
except the OS logo.

## Components

- **Keycap.** 64px square (48px on mobile), white top, Keycap Edge bottom. Like a real Thai keycap it carries both
  legends: Latin top-left in Slate (15px), Thai bottom-right in Legend Blue (28px). Before the flip the Latin legend
  is the strong one; after the flip the Thai legend is (the other legend stays, quieter). A combining Thai mark shown alone on a key is drawn on a dotted
  circle (U+25CC), as on real keyboards: ◌ั ◌ี.
- **Flip.** The hero's six keys `l ; y l f u` flip once on load to show the Thai legends spelling สวัสดี, then the
  fixed word appears on a Highlighter background. Typing in the demo input replays it for the visitor's own text.
  Spec and timing live in `design/motion.md`.
- **Buttons.** Primary = Legend Blue key with white text. Secondary = white key with Deep Ink text. The label says
  exactly what happens: "ดาวน์โหลดสำหรับ Mac", not "เริ่มเลย".
- **Demo input.** Large (title size), white, control radius, with a plain Thai placeholder ("ลองพิมพ์ l;ylfu").
- **Install steps.** OS tabs (Mac / Windows); numbered steps because it is a real sequence; each step has one
  screenshot from a Thai-language system and one sentence.
- **Privacy panel.** Two plain lists: "สิ่งที่ KeyFlip เห็น" and "สิ่งที่ส่งออกไป". Must match the code; link the source files.

## Do's and Don'ts

Do:
- Ground every visual choice in keys, legends, typing and the moment of correction.
- Keep one memorable thing (the flip) and make everything else quiet and readable.
- Write copy from the user's side in plain Thai, sentence case, active verbs.
- Warn about the macOS / Windows first-open warning **before** the download, with the exact steps.
- Respect `prefers-reduced-motion`, show visible keyboard focus, keep 24px minimum hit targets, meet 4.5:1 contrast.

Don't (common tells of generated pages; adapted from Anthropic's `frontend-design` guidance, Apache-2.0):
- A warm cream background with a terracotta/clay accent; a near-black page with one acid-green or vermilion accent;
  broadsheet hairline columns with zero radius.
- The SaaS card kit: content chopped into identical rounded cards with the same soft grey shadow and gradient washes.
- Template chrome: an uppercase eyebrow label above every heading, meta strings joined with middle dots
  ("A · B · C"), "WORD — fragment" labels, a monospace face for small labels, "→" appended to links and buttons.
- Accenting one word of a headline in another colour, italic or weight.
- Numbered markers (01 / 02 / 03) on anything that is not a real sequence. Install steps are a sequence; features are not.
- Fade-and-slide-up on every section, hover lifts on every card, parallax backgrounds, smooth-scroll hijacking.
- Splitting Thai text per character for animation, letter-spacing on Thai, uppercase transforms, clipped tone marks.
- Stock 3D illustrations, emoji as bullets, gradient blobs, glassmorphism.
