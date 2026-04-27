<script lang="ts">
  import { settings } from '$lib/stores/settings.svelte';
  import { flow } from '$lib/stores/flow.svelte';
  import AnchorOrchestrator from '$lib/components/AnchorOrchestrator.svelte';
  import OutlineGallery from '$lib/components/OutlineGallery.svelte';
  import OutlineDetail from '$lib/components/OutlineDetail.svelte';
  import FirmCreationModal from '$lib/components/FirmCreationModal.svelte';
  import DealCreationModal from '$lib/components/DealCreationModal.svelte';
</script>

{#if !settings.loaded}
  <div class="loading-shell">Loading…</div>
{:else if !settings.repoPath}
  <AnchorOrchestrator />
{:else}
  <OutlineGallery />

  {#if flow.stage.kind === 'outline_detail'}
    <OutlineDetail outline={flow.stage.outline} />
  {:else if flow.stage.kind === 'create_firm'}
    <FirmCreationModal outline={flow.stage.outline} />
  {:else if flow.stage.kind === 'create_deal'}
    <DealCreationModal outline={flow.stage.outline} />
  {/if}
{/if}

<style>
  .loading-shell {
    padding: 4rem;
    text-align: center;
    color: #6b7280;
  }

  @media (prefers-color-scheme: dark) {
    .loading-shell {
      color: #9ca3af;
    }
  }
</style>
