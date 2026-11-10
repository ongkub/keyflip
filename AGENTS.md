# Notes for coding agents

## Website (`site/`)
Before creating or changing anything under `site/`, read these files and follow them:
1. `site/DESIGN.md` — visual direction, tokens, components, do's and don'ts (lint: `npx @google/design.md lint site/DESIGN.md`)
2. `site/design/motion.md` and `site/design/ui-checklist.md` — motion and UI quality rules
3. `site/design/thai.md` — Thai typography and Thai text animation rules; test with `site/design/thai-test-strings.txt`
Use the CSS variables in `site/design/tokens.css` and the self-hosted fonts in `site/fonts/`; never load fonts or scripts from third-party CDNs in the shipped site.
