/**
 * Smart `openPath` wrapper that honors the user's preferred markdown editor
 * (single .md files) and markdown notebook / workspace app (directories).
 *
 * `openPath(path, withApp?)` from @tauri-apps/plugin-opener takes an optional
 * second argument that on macOS maps to `open -a {with} {path}`. We pick the
 * right `with` based on the target shape and the user's settings; if neither
 * preference is set or the target shape doesn't match, we fall through to
 * the OS default handler (the same behavior as plain openPath).
 */

import { openPath } from '@tauri-apps/plugin-opener';
import { settings } from '$lib/stores/settings.svelte';

const MARKDOWN_EXTENSIONS = ['md', 'mdx', 'markdown'];

export interface OpenOptions {
  /** Treat the path as a directory regardless of its on-disk shape. */
  isDir?: boolean;
}

function isMarkdownPath(path: string): boolean {
  const dot = path.lastIndexOf('.');
  if (dot < 0) return false;
  const ext = path.slice(dot + 1).toLowerCase();
  return MARKDOWN_EXTENSIONS.includes(ext);
}

/**
 * Open a path with the user's preferred app when applicable, or the OS
 * default otherwise. Throws on launch failure (caller should surface).
 */
export async function openWithPreferred(
  path: string,
  opts: OpenOptions = {}
): Promise<void> {
  const isDir = opts.isDir === true;
  const isMd = !isDir && isMarkdownPath(path);

  let withApp: string | undefined;
  if (isDir && settings.markdownNotebook) {
    withApp = settings.markdownNotebook;
  } else if (isMd && settings.markdownEditor) {
    withApp = settings.markdownEditor;
  }

  // openPath's second param is optional; passing undefined uses the OS default.
  await openPath(path, withApp);
}
