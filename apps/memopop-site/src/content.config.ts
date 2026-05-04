import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

// Treat empty strings and nulls as "not provided" before coercing.
const lenientDate = z.preprocess(
  (v) => (v === '' || v === null ? undefined : v),
  z.coerce.date().optional(),
);

const lenientString = z.preprocess(
  (v) => (v === '' || v === null ? undefined : v),
  z.string().optional(),
);

const lenientStringArray = z.preprocess(
  (v) => {
    if (v === '' || v === null || v === undefined) return undefined;
    if (Array.isArray(v)) return v.map(String);
    if (typeof v === 'string') return [v];
    return v;
  },
  z.array(z.string()).optional(),
);

const lenientBoolean = z.preprocess(
  (v) => (v === '' || v === null ? undefined : v),
  z.boolean().optional(),
);

const lenientNumber = z.preprocess(
  (v) => (v === '' || v === null ? undefined : v),
  z.number().optional(),
);

const changelogSchema = z
  .object({
    title: lenientString,
    lede: lenientString,
    summary: lenientString,
    description: lenientString,

    date: lenientDate,
    date_authored_initial_draft: lenientDate,
    date_authored_current_draft: lenientDate,
    date_authored_final_draft: lenientDate,
    date_first_published: lenientDate,
    date_last_updated: lenientDate,
    date_created: lenientDate,
    date_modified: lenientDate,

    category: lenientString,
    status: lenientString,
    at_semantic_version: lenientString,
    augmented_with: lenientString,
    app: lenientString,
    publish: lenientBoolean,
    usage_index: lenientNumber,

    tags: lenientStringArray,
    authors: lenientStringArray,
    files_added: lenientStringArray,
    files_modified: lenientStringArray,
    files_removed: lenientStringArray,

    image: lenientString,
    image_prompt: lenientString,
    image_text: lenientString,
  })
  .passthrough();

// Project-root-relative paths (project root = apps/memopop-site/).
// Top-level *.md only — skips orchestrator's `releases/` subfolder for now.
const monorepo = defineCollection({
  loader: glob({ pattern: '*.md', base: '../../changelog' }),
  schema: changelogSchema,
});

const orchestrator = defineCollection({
  loader: glob({ pattern: '*.md', base: '../memopop-orchestrator/changelog' }),
  schema: changelogSchema,
});

const site = defineCollection({
  loader: glob({ pattern: '*.md', base: './changelog' }),
  schema: changelogSchema,
});

const native = defineCollection({
  loader: glob({ pattern: '*.md', base: '../memopop-native/changelog' }),
  schema: changelogSchema,
});

export const collections = {
  'changelog-monorepo': monorepo,
  'changelog-orchestrator': orchestrator,
  'changelog-site': site,
  'changelog-native': native,
};
