<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { settings } from '$lib/stores/settings.svelte';
  import { getTransport } from '$lib/transport';

  interface Props {
    firm: string;
    deal: string;
  }

  let { firm, deal }: Props = $props();

  interface Version {
    name: string;
    output_dir: string;
    last_modified_ms: number;
  }

  interface FileInfo {
    path: string;
    size: number;
  }

  let versions = $state<Version[]>([]);
  let selectedVersion = $state<string | null>(null);
  let files = $state<FileInfo[]>([]);
  let outputDir = $state<string | null>(null);

  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let collapsed = $state<Set<string>>(new Set());

  // Initial load: pull versions, then fetch the most recent (or ?v= override).
  onMount(async () => {
    if (!settings.repoPath) return;
    await loadVersions();
    const requested = page.url.searchParams.get('v');
    if (requested && versions.some((v) => v.name === requested)) {
      selectedVersion = requested;
    } else if (versions.length > 0) {
      selectedVersion = versions[0].name;
    }
    if (selectedVersion) {
      await loadFiles(selectedVersion);
    }
    loading = false;
  });

  async function loadVersions() {
    try {
      const result = await getTransport().request<{ versions: Version[] }>(
        'GET',
        `/firms/${encodeURIComponent(firm)}/deals/${encodeURIComponent(deal)}/versions`,
        { repoPath: settings.repoPath }
      );
      versions = result.versions;
    } catch (e) {
      loadError = (e as { message?: string })?.message ?? 'Failed to load versions';
    }
  }

  async function loadFiles(version: string) {
    try {
      const result = await getTransport().request<{
        version_dir: string;
        files: FileInfo[];
      }>(
        'GET',
        `/firms/${encodeURIComponent(firm)}/deals/${encodeURIComponent(deal)}/versions/${encodeURIComponent(version)}/files`,
        { repoPath: settings.repoPath }
      );
      outputDir = result.version_dir;
      files = result.files;
    } catch (e) {
      loadError = (e as { message?: string })?.message ?? 'Failed to load files';
      files = [];
    }
  }

  async function selectVersion(name: string) {
    selectedVersion = name;
    files = [];
    // Reflect in URL so reloads / shares preserve the choice.
    const url = new URL(page.url);
    url.searchParams.set('v', name);
    history.replaceState(null, '', url.toString());
    await loadFiles(name);
  }

  async function revealOutputDir() {
    if (!outputDir) return;
    try {
      await openPath(outputDir);
    } catch (e) {
      loadError = `Couldn't open: ${e}`;
    }
  }

  async function openFile(rel: string) {
    if (!outputDir) return;
    const abs = `${outputDir}/${rel}`;
    try {
      await openPath(abs);
    } catch (e) {
      loadError = `Couldn't open ${rel}: ${e}`;
    }
  }

  function toggleFolder(folderPath: string) {
    const next = new Set(collapsed);
    if (next.has(folderPath)) next.delete(folderPath);
    else next.add(folderPath);
    collapsed = next;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  function formatRelTime(ms: number): string {
    const diff = Date.now() - ms;
    const minutes = Math.floor(diff / 60000);
    if (minutes < 1) return 'just now';
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    if (days < 30) return `${days}d ago`;
    return new Date(ms).toLocaleDateString();
  }

  // ----- Tree construction (mirrors ArtifactBrowser's logic) -----

  type FileNode = { kind: 'file'; name: string; fullPath: string; size: number };
  type FolderNode = {
    kind: 'folder';
    name: string;
    fullPath: string;
    children: TreeNode[];
  };
  type TreeNode = FileNode | FolderNode;

  function buildTree(items: FileInfo[]): TreeNode[] {
    const root: FolderNode = { kind: 'folder', name: '', fullPath: '', children: [] };
    for (const f of items) {
      const parts = f.path.split('/').filter(Boolean);
      let cursor: FolderNode = root;
      for (let i = 0; i < parts.length; i++) {
        const name = parts[i];
        const isFile = i === parts.length - 1;
        const fullPath = parts.slice(0, i + 1).join('/');
        let child: TreeNode | undefined = cursor.children.find((c) => c.name === name);
        if (!child) {
          child = isFile
            ? { kind: 'file', name, fullPath, size: f.size }
            : { kind: 'folder', name, fullPath, children: [] };
          cursor.children.push(child);
        }
        if (child.kind === 'folder') cursor = child;
      }
    }
    sortFolder(root);
    return root.children;
  }

  function sortFolder(folder: FolderNode) {
    folder.children.sort((a, b) => {
      if (a.kind !== b.kind) return a.kind === 'folder' ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
    for (const c of folder.children) {
      if (c.kind === 'folder') sortFolder(c);
    }
  }

  type FlatRow = { node: TreeNode; depth: number };

  function flatten(nodes: TreeNode[], depth: number, out: FlatRow[]) {
    for (const n of nodes) {
      out.push({ node: n, depth });
      if (n.kind === 'folder' && !collapsed.has(n.fullPath)) {
        flatten(n.children, depth + 1, out);
      }
    }
  }

  function folderFileCount(node: TreeNode): number {
    if (node.kind !== 'folder') return 0;
    let count = 0;
    for (const c of node.children) {
      if (c.kind === 'file') count++;
      else count += folderFileCount(c);
    }
    return count;
  }

  let tree = $derived(buildTree(files));
  let rows = $derived.by(() => {
    const out: FlatRow[] = [];
    flatten(tree, 0, out);
    return out;
  });
</script>

<section class="page">
  <header class="head">
    <button type="button" class="back" onclick={() => goto('/')}>← Home</button>
    <div class="title-block">
      <h1>{deal}</h1>
      <p class="subtitle">
        <code>{firm}</code>
        {#if versions.length > 0}
          · {versions.length} {versions.length === 1 ? 'version' : 'versions'}
        {/if}
      </p>
    </div>

    <div class="head-actions">
      {#if versions.length > 0}
        <select
          class="version-select"
          value={selectedVersion ?? ''}
          onchange={(e) => selectVersion((e.currentTarget as HTMLSelectElement).value)}
          disabled={loading}
        >
          {#each versions as v (v.name)}
            <option value={v.name}>
              {v.name} · {formatRelTime(v.last_modified_ms)}
            </option>
          {/each}
        </select>
      {/if}
      <button
        type="button"
        class="finder-btn"
        onclick={revealOutputDir}
        disabled={!outputDir}
        title="Reveal this version's folder in Finder"
      >
        📁 Open in Finder
      </button>
    </div>
  </header>

  {#if loading}
    <div class="empty">Loading workspace…</div>
  {:else if loadError}
    <div class="empty error">{loadError}</div>
  {:else if versions.length === 0}
    <div class="empty">
      <strong>No runs yet for {deal}.</strong>
      <p class="hint">Generate a memo from this deal's outline to see artifacts here.</p>
    </div>
  {:else if !selectedVersion}
    <div class="empty">Pick a version above.</div>
  {:else}
    <div class="content">
      <header class="content-head">
        <span class="version-badge">{selectedVersion}</span>
        <span class="file-count">
          {files.length} {files.length === 1 ? 'file' : 'files'}
        </span>
        {#if outputDir}
          <code class="path">{outputDir}</code>
        {/if}
      </header>

      <div class="tree">
        {#if rows.length === 0}
          <div class="empty">
            <p>No files in this version.</p>
          </div>
        {:else}
          {#each rows as { node, depth } (node.fullPath)}
            {#if node.kind === 'folder'}
              <button
                type="button"
                class="row folder"
                style:padding-left="{depth * 14 + 6}px"
                onclick={() => toggleFolder(node.fullPath)}
              >
                <span class="caret">{collapsed.has(node.fullPath) ? '▶' : '▼'}</span>
                <span class="folder-name">{node.name}/</span>
                <span class="folder-count">{folderFileCount(node)}</span>
              </button>
            {:else}
              <button
                type="button"
                class="row file"
                style:padding-left="{depth * 14 + 22}px"
                onclick={() => openFile(node.fullPath)}
                title="Open {node.fullPath}"
              >
                <span class="file-name">{node.name}</span>
                <span class="size">{formatSize(node.size)}</span>
              </button>
            {/if}
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .page {
    max-width: 1100px;
    margin: 2rem auto;
    padding: 0 1.5rem 4rem;
    display: flex;
    flex-direction: column;
    height: calc(100vh - 5rem);
    min-height: 0;
  }

  .head {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 1rem;
    padding: 0 0.25rem 1.25rem;
    border-bottom: 1px solid #e5e7eb;
    margin-bottom: 1rem;
    flex-shrink: 0;
  }

  .back {
    background: transparent;
    border: 1px solid #d1d5db;
    color: #4b5563;
    padding: 0.4rem 0.8rem;
    border-radius: 6px;
    font: inherit;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .back:hover {
    background: #f3f4f6;
    color: #0f0f0f;
  }

  .title-block h1 {
    margin: 0 0 0.2rem;
    font-size: 1.4rem;
    font-weight: 700;
  }

  .subtitle {
    margin: 0;
    color: #6b7280;
    font-size: 0.85rem;
  }

  .subtitle code {
    font-family: ui-monospace, SFMono-Regular, monospace;
    color: #5b21b6;
    background: #f5f3ff;
    padding: 0.05rem 0.3rem;
    border-radius: 3px;
  }

  .head-actions {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .version-select {
    padding: 0.4rem 0.6rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: white;
    font: inherit;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .finder-btn {
    background: transparent;
    border: 1px solid #d1d5db;
    color: #4b5563;
    padding: 0.4rem 0.8rem;
    border-radius: 6px;
    font: inherit;
    font-size: 0.85rem;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .finder-btn:hover:not(:disabled) {
    background: #f3f4f6;
    color: #0f0f0f;
  }

  .finder-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .empty {
    text-align: center;
    padding: 3rem 1rem;
    color: #6b7280;
  }

  .empty.error {
    color: #c0392b;
  }

  .hint {
    color: #9ca3af;
    font-size: 0.9rem;
    margin-top: 0.5rem;
  }

  .content {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    overflow: hidden;
  }

  .content-head {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    padding: 0.75rem 1rem;
    background: #fafaf9;
    border-bottom: 1px solid #e5e7eb;
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .version-badge {
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 0.8rem;
    background: #5b21b6;
    color: white;
    padding: 0.2rem 0.55rem;
    border-radius: 999px;
    font-weight: 600;
  }

  .file-count {
    font-size: 0.78rem;
    color: #6b7280;
    font-variant-numeric: tabular-nums;
  }

  .path {
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 0.75rem;
    color: #9ca3af;
    margin-left: auto;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .tree {
    flex: 1;
    overflow-y: auto;
    padding: 0.4rem 0;
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 0.82rem;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding-top: 0.22rem;
    padding-bottom: 0.22rem;
    padding-right: 1rem;
    line-height: 1.3;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    color: inherit;
    font: inherit;
  }

  .row:hover {
    background: #f5f3ff;
  }

  .caret {
    color: #9ca3af;
    font-size: 0.65rem;
    width: 10px;
    flex-shrink: 0;
  }

  .folder-name {
    color: #4b5563;
    font-weight: 600;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .folder-count {
    background: #e5e7eb;
    color: #6b7280;
    font-size: 0.65rem;
    padding: 0.05rem 0.4rem;
    border-radius: 999px;
    font-family: system-ui, -apple-system, sans-serif;
    flex-shrink: 0;
  }

  .file-name {
    color: #0f0f0f;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .size {
    font-size: 0.7rem;
    color: #9ca3af;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  @media (prefers-color-scheme: dark) {
    .head {
      border-bottom-color: #2a2a2c;
    }
    .back,
    .finder-btn {
      border-color: #3a3a3c;
      color: #d1d5db;
    }
    .back:hover,
    .finder-btn:hover:not(:disabled) {
      background: #2a2a2c;
      color: #f6f6f6;
    }
    .title-block h1 {
      color: #f6f6f6;
    }
    .subtitle {
      color: #9ca3af;
    }
    .subtitle code {
      background: #2a1f3d;
      color: #c4b5fd;
    }
    .version-select {
      background: #1c1c1e;
      border-color: #3a3a3c;
      color: #f6f6f6;
    }
    .empty {
      color: #9ca3af;
    }
    .empty.error {
      color: #fca5a5;
    }
    .hint {
      color: #6b7280;
    }
    .content {
      background: #1c1c1e;
      border-color: #2a2a2c;
    }
    .content-head {
      background: #2a2a2c;
      border-bottom-color: #3a3a3c;
    }
    .file-count,
    .path {
      color: #9ca3af;
    }
    .row:hover {
      background: #2a1f3d;
    }
    .caret,
    .size {
      color: #6b7280;
    }
    .folder-name {
      color: #d1d5db;
    }
    .folder-count {
      background: #2a2a2c;
      color: #9ca3af;
    }
    .file-name {
      color: #f6f6f6;
    }
  }
</style>
