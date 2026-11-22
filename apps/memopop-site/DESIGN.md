---
name: MemoPop AI — Site
description: The marketing site for MemoPop AI.
colors:
  dark-text: "#f1f5f9"
  dark-accent: "#04e5e5"
  light-text: "#0a0a14"
  light-accent: "#028f8f"
typography:
  display:
    fontFamily: Fraunces
  mono:
    fontFamily: JetBrains Mono
---

# MemoPop AI — Site

> **Partial.** Written 2026-11-22 to record the trademark pack. Only the tokens below were verified against `src/styles/theme.css`, which stays the source of truth. Colors beyond text and accent, Typography detail, Layout, Elevation & Depth, Shapes, Components and Do's and Don'ts are not written yet; see the `maintain-design-md` skill.

## Overview

The marketing site for MemoPop AI.

## Imagery

### Trademark pack

In `public/`, every piece outlined to paths (an `<img>` cannot load web fonts).

| Piece | File |
|---|---|
| appIcon | `appIcon__MemoPop.svg` (same tile as `favicon.svg`; `MemoMark.astro` draws the mark inline) |
| favicon | `favicon.svg` |
| wordmark | `wordmark__MemoPop--{Dark,Light}-Mode.svg`: *memopop* in JetBrains Mono 500 with *.ai* in cyan, as the header sets it |
| trademark | `trademark__MemoPop--{Dark,Light}-Mode.svg`: appIcon + wordmark |
