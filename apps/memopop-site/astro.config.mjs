// @ts-check
import { defineConfig } from 'astro/config';
import pagefind from 'astro-pagefind';

// https://astro.build/config
//
// Hosted on GitHub Pages from the `lossless-group/memopop-ai` repo.
// Live URL: https://lossless-group.github.io/memopop-ai/
//
// If a custom domain is added later (e.g. memopop.lossless.group via a
// public/CNAME file), set `site` to that domain and `base` to '/'.
export default defineConfig({
  site: 'https://lossless-group.github.io',
  base: '/memopop-ai/',
  trailingSlash: 'ignore',

  // astro-pagefind runs Pagefind against `dist/` after `astro build` and copies
  // pagefind/* into the published output. Search runs entirely client-side from
  // the static index — no backend, no cost, mode-pivot-aware via theme tokens.
  integrations: [pagefind()],

  build: {
    // Pagefind needs a stable per-page URL — use directory output so each
    // entry's `data-pagefind-body` lands at /changelog/<from>/<slug>/index.html.
    format: 'directory',
  },
});
