// @ts-check
import { defineConfig } from 'astro/config';

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
});
