<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { getTransport } from '$lib/transport';

  interface ArtifactInfo {
    path: string;
    size: number;
  }

  interface Props {
    jobId: string;
    isRunning: boolean;
  }

  let { jobId, isRunning }: Props = $props();

  let files = $state<ArtifactInfo[]>([]);
  let outputDir = $state<string | null>(null);
  let lastFetchAt = $state<Date | null>(null);
  let recentlyAdded = $state<Set<string>>(new Set());
  let pollHandle: ReturnType<typeof setInterval> | null = null;
  let error = $state<string | null>(null);

  async function fetchOnce() {
    if (!settings.repoPath) return;
    try {
      const result = await getTransport().request<{
        output_dir: string | null;
        files: ArtifactInfo[];
      }>('GET', `/memos/${jobId}/artifacts`, { repoPath: settings.repoPath });

      // Diff to flag newly-arrived files for a brief highlight.
      const previous = new Set(files.map((f) => f.path));
      const fresh: string[] = [];
      for (const f of result.files) {
        if (!previous.has(f.path)) fresh.push(f.path);
      }
      if (fresh.length > 0 && files.length > 0) {
        // Don't flash on the very first fetch — that'd light up everything at once.
        recentlyAdded = new Set([...recentlyAdded, ...fresh]);
        const toClear = fresh.slice();
        setTimeout(() => {
          recentlyAdded = new Set([...recentlyAdded].filter((p) => !toClear.includes(p)));
        }, 3000);
      }

      files = result.files;
      outputDir = result.output_dir;
      lastFetchAt = new Date();
      error = null;
    } catch (e) {
      // Common when the job hasn't created its output dir yet (404 from the
      // dispatcher). Don't surface as a hard error; just keep polling.
      const status = (e as { status?: number })?.status;
      if (status !== 404) {
        error = (e as { message?: string })?.message ?? 'Failed to load files';
      }
    }
  }

  onMount(() => {
    fetchOnce();
    pollHandle = setInterval(fetchOnce, 3000);
  });

  onDestroy(() => {
    if (pollHandle) clearInterval(pollHandle);
  });

  // When the job finishes, do one final fetch then stop polling.
  $effect(() => {
    if (!isRunning && pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
      fetchOnce();
    }
  });

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  function topFolder(path: string): string {
    const i = path.indexOf('/');
    return i >= 0 ? path.slice(0, i) : '';
  }

  function shorten(p: string): string {
    const parts = p.split('/').filter(Boolean);
    if (parts.length <= 3) return p;
    return '…/' + parts.slice(-3).join('/');
  }

  function formatRelative(d: Date): string {
    const seconds = Math.floor((Date.now() - d.getTime()) / 1000);
    if (seconds < 5) return 'just now';
    if (seconds < 60) return `${seconds}s ago`;
    const minutes = Math.floor(seconds / 60);
    return `${minutes}m ago`;
  }

  let grouped = $derived(groupFiles(files));

  function groupFiles(items: ArtifactInfo[]) {
    const map = new Map<string, ArtifactInfo[]>();
    for (const item of items) {
      const folder = topFolder(item.path);
      const key = folder || '_root';
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(item);
    }
    // Sort folders so '_root' (top-level files) comes first, then alphabetical.
    return [...map.entries()].sort(([a], [b]) => {
      if (a === '_root') return -1;
      if (b === '_root') return 1;
      return a.localeCompare(b);
    });
  }
</script>

<div class="browser">
  <header class="head">
    {#if outputDir}
      <code class="dir" title={outputDir}>{shorten(outputDir)}</code>
    {:else}
      <span class="dir-empty">No output directory yet</span>
    {/if}
    <span class="count">{files.length} {files.length === 1 ? 'file' : 'files'}</span>
  </header>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="files">
    {#if files.length === 0}
      <div class="empty">
        {isRunning ? 'No files written yet — they\'ll appear here as the orchestrator works.' : 'No files were produced.'}
      </div>
    {:else}
      {#each grouped as [folder, items] (folder)}
        <section class="group">
          <h4 class="group-name">
            {folder === '_root' ? '/' : folder + '/'}
            <span class="group-count">{items.length}</span>
          </h4>
          <ul>
            {#each items as f (f.path)}
              <li class="file" class:fresh={recentlyAdded.has(f.path)}>
                <code class="name">{folder === '_root' ? f.path : f.path.slice(folder.length + 1)}</code>
                <span class="size">{formatSize(f.size)}</span>
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    {/if}
  </div>

  {#if lastFetchAt}
    <div class="footer-meta">
      Updated {formatRelative(lastFetchAt)}
      {#if isRunning}<span class="poll-dot" title="Polling every 3s"></span>{/if}
    </div>
  {/if}
</div>

<style>
  .browser {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.4rem 0 0.6rem;
    border-bottom: 1px solid #f3f4f6;
    margin-bottom: 0.5rem;
  }

  .dir {
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 0.78rem;
    color: #5b21b6;
    background: #f5f3ff;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    border: 1px solid #ddd6fe;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 70%;
  }

  .dir-empty {
    color: #9ca3af;
    font-size: 0.85rem;
    font-style: italic;
  }

  .count {
    font-size: 0.78rem;
    color: #6b7280;
    font-variant-numeric: tabular-nums;
  }

  .files {
    flex: 1;
    overflow-y: auto;
    padding-right: 0.25rem;
  }

  .empty {
    padding: 2rem 1rem;
    text-align: center;
    color: #6b7280;
    font-style: italic;
  }

  .group {
    margin-bottom: 0.85rem;
  }

  .group-name {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #6b7280;
    margin: 0 0 0.3rem;
    padding: 0 0.1rem;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-family: ui-monospace, SFMono-Regular, monospace;
  }

  .group-count {
    background: #f3f4f6;
    color: #6b7280;
    font-size: 0.65rem;
    padding: 0.05rem 0.35rem;
    border-radius: 999px;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .file {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.3rem 0.5rem;
    border-radius: 6px;
    transition: background 0.4s ease;
  }

  .file:hover {
    background: #f9fafb;
  }

  .file.fresh {
    background: #ecfccb;
    animation: flash 3s ease-out forwards;
  }

  @keyframes flash {
    from {
      background: #84cc16;
    }
    to {
      background: transparent;
    }
  }

  .name {
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 0.8rem;
    color: #0f0f0f;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    background: transparent;
    border: none;
    padding: 0;
  }

  .size {
    font-size: 0.72rem;
    color: #9ca3af;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .error {
    color: #c0392b;
    font-size: 0.85rem;
    margin: 0 0 0.5rem;
    padding: 0.4rem 0.6rem;
    background: #fef2f2;
    border-radius: 6px;
    border: 1px solid #fca5a5;
  }

  .footer-meta {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding-top: 0.5rem;
    margin-top: 0.5rem;
    border-top: 1px solid #f3f4f6;
    font-size: 0.72rem;
    color: #9ca3af;
  }

  .poll-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #10b981;
    animation: pulse 1.4s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.3;
    }
  }

  @media (prefers-color-scheme: dark) {
    .head {
      border-bottom-color: #3a3a3c;
    }

    .dir {
      background: #2a1f3d;
      border-color: #5b21b6;
      color: #c4b5fd;
    }

    .dir-empty {
      color: #6b7280;
    }

    .count {
      color: #9ca3af;
    }

    .empty {
      color: #9ca3af;
    }

    .group-name {
      color: #9ca3af;
    }

    .group-count {
      background: #2a2a2c;
      color: #9ca3af;
    }

    .file:hover {
      background: #2a2a2c;
    }

    .file.fresh {
      background: #064e3b;
    }

    @keyframes flash {
      from {
        background: #16a34a;
      }
      to {
        background: transparent;
      }
    }

    .name {
      color: #f6f6f6;
    }

    .size {
      color: #6b7280;
    }

    .error {
      background: #2a1c1c;
      border-color: #7f1d1d;
    }

    .footer-meta {
      border-top-color: #3a3a3c;
      color: #6b7280;
    }
  }
</style>
