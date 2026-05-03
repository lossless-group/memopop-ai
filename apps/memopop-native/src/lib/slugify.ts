/**
 * Slugify a firm name into snake_case for use as a directory name.
 *
 * Mirrors the Rust `slugify` in src-tauri/src/api/actions.rs so the UI can
 * preview the resulting path before the user confirms.
 *
 * Rules:
 *   - Lowercase
 *   - Spaces, hyphens, dots → underscores
 *   - Strip everything that isn't alphanumeric or underscore
 *   - Collapse runs of underscores
 *   - Trim leading/trailing underscores
 */
export function slugify(input: string): string {
  const lowered = input.toLowerCase();
  let out = '';
  let lastWasUnderscore = false;

  for (const ch of lowered) {
    if (/[a-z0-9]/.test(ch)) {
      out += ch;
      lastWasUnderscore = false;
    } else if (ch === ' ' || ch === '-' || ch === '_' || ch === '.') {
      if (!lastWasUnderscore && out.length > 0) {
        out += '_';
        lastWasUnderscore = true;
      }
    }
  }

  return out.replace(/^_+|_+$/g, '');
}
