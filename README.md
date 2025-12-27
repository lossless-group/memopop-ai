# MemoPop AI

AI-powered investment memo generation for venture capital firms.

## Monorepo Structure

```
memopop-ai/
├── apps/
│   ├── memopop-site/              # Astro - Marketing & SEO
│   ├── memopop-web-app/           # Svelte 5 - Interactive Web App
│   └── memopop-agent-orchestrator/ # Python / LangGraph- AI Agent Orchestrator
│
├── packages/
│   ├── shared-styles/             # Tailwind config & design tokens
│   ├── shared-types/              # TypeScript types (future)
│   └── api-client/                # API client SDK (future)
│
├── infrastructure/
│   └── docker/                    # Containerization
│
└── docs/                          # Documentation
```

## Quick Start

```bash
# Install Bun (if not installed)
curl -fsSL https://bun.sh/install | bash

# Install all dependencies
bun install

# Run marketing site
bun run dev:site

# Run web app
bun run dev:app
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Package Manager | Bun |
| Marketing Site | Astro (no React) |
| Web App | SvelteKit + Svelte 5 |
| Agent Engine | Python + LangGraph |
| Styling | Tailwind CSS (shared config) |
| Deployment | Vercel |
| Auth | SuperTokens (Railway) |
| Database | Baserow (Railway) |

## Apps

### memopop-site
Marketing site, changelog, thought-leadership, documentation. Built with Astro for optimal SEO and performance.

### memopop-web-app
Interactive web application for memo generation. Built with Svelte 5 (runes) for modern reactivity.

### memopop-agent-orchestrator
Python backend with 12+ specialized AI agents for research, writing, citation, and validation.

## Shared Packages

### @memopop/shared-styles
Tailwind configuration and design tokens shared across Astro and Svelte apps.

```js
// Import in your app's tailwind.config.js
import sharedConfig from '@memopop/shared-styles/tailwind';

export default {
  presets: [sharedConfig],
  content: ['./src/**/*.{astro,html,js,svelte,ts}'],
};
```

## Development

Each app has its own changelog in `apps/{app-name}/changelog/`. The marketing site aggregates all changelogs via GitHub raw API.

## License

MIT
