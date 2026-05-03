import type { Outline } from '$lib/types';
import type { JobEvent, MilestoneStage, MilestoneLevel } from '$lib/transport';

export interface Milestone {
  id: string;
  ts: string;
  stage: MilestoneStage;
  level: MilestoneLevel;
  label: string;
  detail?: string;
}

export interface DealPayload {
  companyUrl: string;
  companyName: string;
  deckPath: string | null;
  mode: 'consider' | 'justify';
}

export type JobRunStatus = 'queued' | 'running' | 'completed' | 'failed';

// In-memory event cap for the running_job stage. The Svelte 5 deep-reactivity
// system creates a Proxy per nested object — at ~5–10k events on a long run,
// the per-event proxy overhead plus #each's full re-realization causes layout
// thrash and GC pauses ("screen going black every 20–30s"). 2000 is plenty
// for tailing live; the persisted `.logs/runs/{job}.jsonl` has the full record.
const MAX_EVENTS = 2000;

export type FlowStage =
  | { kind: 'idle' }
  | { kind: 'outline_detail'; outline: Outline }
  | { kind: 'create_firm'; outline: Outline }
  | { kind: 'create_deal'; outline: Outline }
  | { kind: 'ready_to_run'; outline: Outline; payload: DealPayload }
  | {
      kind: 'running_job';
      outline: Outline;
      payload: DealPayload;
      jobId: string;
      status: JobRunStatus;
      events: JobEvent[];
      milestones: Milestone[];
      outputDir: string | null;
      version: string | null;
      errorMessage: string | null;
    }
  | { kind: 'brand_setup'; firm: string };

class FlowState {
  stage = $state<FlowStage>({ kind: 'idle' });

  showDetail(outline: Outline) {
    this.stage = { kind: 'outline_detail', outline };
  }

  startTrying(outline: Outline, hasActiveFirm: boolean) {
    this.stage = hasActiveFirm
      ? { kind: 'create_deal', outline }
      : { kind: 'create_firm', outline };
  }

  proceedToDeal(outline: Outline) {
    this.stage = { kind: 'create_deal', outline };
  }

  markReady(outline: Outline, payload: DealPayload) {
    this.stage = { kind: 'ready_to_run', outline, payload };
  }

  editDeal() {
    if (this.stage.kind === 'ready_to_run' || this.stage.kind === 'running_job') {
      this.stage = { kind: 'create_deal', outline: this.stage.outline };
    }
  }

  markCancelled(reason: string) {
    if (this.stage.kind !== 'running_job') return;
    this.stage.status = 'failed';
    this.stage.errorMessage = reason;
    this.stage.milestones.push({
      id: `cancelled-${Date.now()}`,
      ts: new Date().toISOString(),
      stage: 'complete',
      level: 'error',
      label: 'Stopped by user',
    });
  }

  markRunning(outline: Outline, payload: DealPayload, jobId: string) {
    this.stage = {
      kind: 'running_job',
      outline,
      payload,
      jobId,
      status: 'queued',
      events: [],
      milestones: [],
      outputDir: null,
      version: null,
      errorMessage: null,
    };
  }

  appendEvent(event: JobEvent) {
    if (this.stage.kind !== 'running_job') return;
    this.stage.events.push(event);
    // Cap the event log to a rolling tail to keep webview memory and reactive
    // proxy churn bounded. The full event stream is on disk via the sidecar's
    // server-log.jsonl persistence, so dropping older entries here is a UI
    // optimization only. Milestones are kept uncapped — they're small and the
    // checklist needs the full pipeline view.
    if (this.stage.events.length > MAX_EVENTS) {
      this.stage.events = this.stage.events.slice(-MAX_EVENTS);
    }

    if (event.type === 'status' && typeof event.status === 'string') {
      this.stage.status = event.status as JobRunStatus;
    } else if (event.type === 'complete') {
      this.stage.status = 'completed';
      if (typeof event.output_dir === 'string') this.stage.outputDir = event.output_dir;
      if (typeof event.version === 'string') this.stage.version = event.version;
      // Synthesize a "complete" milestone for the timeline.
      this.stage.milestones.push({
        id: `complete-${Date.now()}`,
        ts: event.ts,
        stage: 'complete',
        level: 'success',
        label: 'Memo complete',
        detail:
          typeof event.version === 'string' ? `Version ${event.version}` : undefined,
      });
    } else if (event.type === 'error') {
      this.stage.status = 'failed';
      if (typeof event.message === 'string') this.stage.errorMessage = event.message;
      this.stage.milestones.push({
        id: `error-${Date.now()}`,
        ts: event.ts,
        stage: 'complete',
        level: 'error',
        label: 'Run failed',
        detail: typeof event.message === 'string' ? event.message : undefined,
      });
    } else if (event.type === 'milestone') {
      this.stage.milestones.push({
        id: typeof event.id === 'string' ? event.id : `m-${Date.now()}`,
        ts: event.ts,
        stage: (typeof event.stage === 'string' ? event.stage : 'start') as MilestoneStage,
        level: (typeof event.level === 'string' ? event.level : 'info') as MilestoneLevel,
        label: typeof event.label === 'string' ? event.label : 'Update',
        detail: typeof event.detail === 'string' ? event.detail : undefined,
      });
    }
  }

  startBrandSetup(firm: string) {
    this.stage = { kind: 'brand_setup', firm };
  }

  close() {
    this.stage = { kind: 'idle' };
  }
}

export const flow = new FlowState();
