---
title: "Premium Data Sources Integrations into MemoPop"
lede: >-
  PitchBook, CB Insights, Statista and a firm's own licensed data, brought into a memo with full provenance, without leaking what the license keeps private.
date_authored_initial_draft: 2026-11-26
date_authored_current_draft: 2026-11-26
date_authored_final_draft:
date_first_published:
date_last_updated: 2026-11-26
date_created: 2026-11-26
date_modified: 2026-11-26
at_semantic_version: 0.0.0.1
status: Draft
publish: false
category: Specification
type: Specs
tags:
  - MemoPop
  - Premium-Data
  - Perplexity
  - Source-Curation
  - Provenance
  - Licensing
  - Model-Context-Protocol
authors:
  - Michael Staton
augmented_with:
  - Claude Code on Claude Opus 5.5
site_uuid: 25eb8d67-7a29-4156-a2ff-90e31c87c39f
hex_code: n82e7d
summary: >-
  How MemoPop uses the licensed data Perplexity now exposes (included sources
  such as PitchBook Essentials, CB Insights, Statista, Wiley and Midpage, and
  licensed connectors such as Crunchbase, FactSet, Guidepoint and Quartr).
  Records what exists and how code can reach it as of 2026-11-26 (the Perplexity
  app and the Computer MCP server can; the Agent API cannot), maps each source
  onto the orchestrator's LangGraph nodes, and specifies capture as SourceFile
  records with origin "premium", sensitivity internal_only, per-fact-type
  precedence, a per-run query budget, per-firm entitlements in brand config,
  and export rules for licensed content. Phased from analyst paste to the
  connector route. Read before writing any plan or code that calls a premium
  source.
---

# Premium Data Sources Integrations into MemoPop

## Why Care?

A memo is only as good as what its research can reach. Today MemoPop reaches two places: the company's own dataroom, and the open web through Perplexity Sonar and SearXNG. For a seed-stage private company the open web is thin. Funding history, who invested, and how big the market is are exactly the facts a partner checks first, and they're the facts most often missing or wrong on the open web.

The data partners actually rely on (PitchBook, Crunchbase, CB Insights, Statista, expert-call libraries) sits behind licenses. Perplexity now puts some of it inside its product. Some comes with every plan, and the rest connects to a subscription the firm already pays for. That makes the data reachable from an agent for the first time, but with strings attached: quotas, licenses, and an account that belongs to a person.

This spec covers how MemoPop uses that data without breaking its two standing rules: **every claim traces to a source an analyst can open**, and **a firm's data stays the firm's**.

## What exists (as of 2026-11-26)

From Perplexity's help center article [Premium data sources](https://www.perplexity.ai/help-center/en/articles/12870803-premium-data-sources) (last modified 2026-11-17) and its [API docs](https://docs.perplexity.ai/llms.txt). These are vendor descriptions and should be checked against real answers before we rely on them.

### Included sources

These come with every Perplexity plan, with no partner account. Each plan gets a limited number of premium queries a month.

| Source | What it holds | Memo sections it serves |
|---|---|---|
| **PitchBook Essentials** | Company profiles: firmographics, funding history, key investors, leadership | Company overview, funding and cap table context, team, investor syndicate, comparables |
| **CB Insights** | Market trends, market maps, company activity | Market, competitive landscape |
| **Statista** | Statistics, market data, trend analysis, forecasts | Market size, growth, adoption |
| **Wiley** | Business, STEM, psychology and medical books and journals | Technical and scientific diligence, health and life sciences |
| **Midpage** | 13.8M U.S. court opinions, statutes, regulations and constitutions, with a citator | Regulatory risk, legal and IP considerations |

"Essentials" is a subset: basic firmographics, funding history, key investors and leadership, not the full PitchBook dataset.

### Licensed connectors

These link a subscription the firm already holds. They need a Perplexity Pro, Max or Enterprise plan plus the partner's license, and they run "under your existing agreement with that partner". Several need a separate entitlement on the partner's side. For example, FactSet MCP access isn't included with FactSet Workstation, Guidepoint has to enable MCP access, and Morningstar needs an MCP server license or Morningstar Direct.

| Group | Connectors | What MemoPop would use them for |
|---|---|---|
| Private markets | Crunchbase, Grata, D&B Commercial Graph, Chronograph, AngelList, Octus, Papermark | Funding, investors and people (Crunchbase), private-company firmographics (Grata, D&B), fund and portfolio metrics for portfolio memos (Chronograph, AngelList), data rooms (Papermark) |
| Expert knowledge | Guidepoint | Expert interview transcripts for "what operators and customers say" |
| Public markets | FactSet, Morningstar, Daloopa, Quartr, Zacks, Wisesheets, AlphaCreek, Canary Data, MT Newswires, IBISWorld | Public comparables, earnings calls, filings, and industry research (IBISWorld) |
| Crypto | Coinbase, Base, OpenSea, Allium | Onchain diligence for crypto deals |

### How code can reach them

| Route | Reaches premium data? | Auth and billing | Notes |
|---|---|---|---|
| Perplexity app (`@PitchBook …`) | Yes | A person's plan and quota | Manual. An analyst asks, then pastes the answer. |
| **Perplexity Computer MCP server**, `https://www.perplexity.ai/rest/computer/mcp` | **Yes.** Its connector list includes CB Insights, PitchBook, Statista and SEC EDGAR among 400+ services | OAuth 2.1 as a Perplexity user. API keys are rejected. Usage draws on that account's credits | Tasks take minutes and support MCP Tasks. Connector sign-ins surface as `auth_required` events. `incognito: true` keeps a conversation out of the user's history and memory. |
| Perplexity Agent API (`/v1/agent`) | **No.** Its managed connectors are GitHub, Slack, Google Drive, Datadog, Linear and Notion | API key | The nearest tools are `finance_search` (public markets, $5 per 1,000 calls) and `people_search`. |
| Agent API **custom connector** to a partner's own MCP server | Only where the partner publishes one, as FactSet and Morningstar do for licensed users | The partner's license, and our API key | The only route that can run unattended without a person's Perplexity seat. |

**What this means for the design:** for now, premium data enters MemoPop through a person (the app), an agent signed in as a person (Computer MCP), or a partner's own MCP server. The orchestrator's direct Perplexity calls can't reach it, and a run on a schedule can't reach it without someone's seat. The design keeps the capture format independent of the route, so a future Agent API connector slots in without changing anything downstream.

## Where it fits in the pipeline

The orchestrator's nodes in `src/workflow.py`, and what premium data changes in each:

| Node | Today | With premium sources |
|---|---|---|
| `dataroom` | Reads the company's documents, ranked first | Unchanged. A Papermark data room could feed it through the firm's Papermark connector. |
| `research`, `section_research` | Perplexity Sonar Pro per section, plus SearXNG candidates | Before reaching for the web, ask the firm's premium sources the questions they answer best (precedence below) and file each answer as a source. |
| `competitive_researcher`, `competitive_evaluator` | Discover and classify competitors through Sonar Pro | CB Insights market maps and PitchBook or Crunchbase comparables supply the candidate list, and the web fills gaps. |
| `team_roster_enrichment` | Web and social lookups | PitchBook leadership and Crunchbase people as the first pass. |
| `aggregate_sources` (Sources.md) | Dataroom at rank 0, web at rank 1, gated publishers marked `internal_only` | Premium answers appear in their own block, marked `internal_only`, for the analyst to approve like any other source. |
| `fact_verify` | Re-checks claims through Sonar Pro | Funding, valuation and investor claims are checked against PitchBook or Crunchbase first. Where premium and web sources disagree, flag the conflict rather than silently picking one. |
| `scorecard` (12Ps) | Scores from the memo | Market-size and traction scores can cite Statista or IBISWorld, with the estimate's publisher named. |
| Portfolio memos (`portfolio_listing_agent`, `run_portfolio.py`) | Web and firm files | Chronograph or AngelList portfolio metrics, when the firm connects them. |

## Design

### 1. Capture every answer as a source file

A premium answer is a source like any other. It goes through curation, extraction and citation, and it's never pasted straight into prose. It's written as a `SourceFile` (`src/curation/source_file.py`) using fields that already exist:

| Field | Value |
|---|---|
| `origin` | `premium` (a new value beside `searxng`, `perplexity`, `analyst-paste`, `pack` and `inbox`) |
| `origin_detail` | `provider` (e.g. `pitchbook-essentials`), `route` (`app`, `computer-mcp` or `partner-mcp`), `question`, `asked_at`, `thread_id` or answer permalink, `account` (whose seat), `entitlement` (`included` or `licensed`) |
| `url` | The record's own page at the provider when the answer names one (a PitchBook profile URL, a Statista statistic page). Otherwise the answer's permalink |
| `publisher` | The data provider, not Perplexity ("PitchBook", "Statista") |
| `sensitivity` | **Always `internal_only`.** This is the default and can't be overridden by the pipeline. Only an analyst can change it, and only with the license in hand |
| `body` | The answer text verbatim, then an `# Extracts` section, following [Extract Sources Where Possible, with Fallback to Ground Content](Extract-Sources-Where-Possible-with-Fallback-to-Ground-Content.md). The premium answer *is* the ground content, since there's usually no page to fetch |
| `published_at` | The data's as-of date when the answer states one ("as of Q3 2026"), never the date we asked |
| `tags` | `premium`, plus the provider |

Each answer cites its own sources inside the provider. Those citations go into `extra_metadata.provider_citations`, so an analyst with access can open them.

### 2. Precedence by fact type

The local-first rule in `src/source_priority.py` extends one tier: establish what the documents say, then what the licensed data says, then send the web after what's still missing.

| Fact type | Order |
|---|---|
| Terms, cap table, financials | Dataroom only. Premium and web data are context, never the answer |
| Funding rounds, amounts, investors | Dataroom, then PitchBook or Crunchbase, then the web |
| Leadership and team | Dataroom, then PitchBook or Crunchbase people, then LinkedIn and the web |
| Market size and growth | Analyst-provided, then Statista or IBISWorld, then the web. Always attribute the estimate ("Statista estimates …") |
| Competitors | CB Insights, PitchBook and Crunchbase together, then the web |
| Public comparables | FactSet, Quartr or Daloopa if connected, otherwise `finance_search` |
| Regulation and case law | Midpage, then primary government sources on the web |

### 3. Per-firm entitlements

A firm declares which premium sources it holds and allows in `io/{firm}/configs/brand-{firm}-config.yaml`. Nothing is on by default.

```yaml
# io/{firm}/configs/brand-{firm}-config.yaml (sketch)
premium_sources:
  route: computer-mcp            # app | computer-mcp | partner-mcp
  account: analyst@firm.example  # whose seat and credits a run uses
  max_queries_per_run: 12
  sources:
    - pitchbook-essentials       # included
    - cb-insights                # included
    - statista                   # included
    - crunchbase                 # licensed: the firm's own subscription
    - guidepoint                 # licensed, MCP access enabled by Guidepoint
```

A run reads this, asks only the listed sources, and records the account in every source file it writes. Firm-private configs stay in the firm's private submodule, per the orchestrator's AGENTS.md.

### 4. Budget and reuse

- **A per-run cap** (`max_queries_per_run`, also settable as `MEMOPOP_PREMIUM_MAX_QUERIES`). Included sources have a monthly premium-query limit per plan, and licensed connectors spend the partner account's quota. A portfolio run across 20 companies must not use up a month's allowance.
- **Questions are planned before they're asked.** One question per fact type per company, batched into as few Computer tasks as the route allows, not one per section.
- **Answers are reused across versions.** `src/reuse_gate.py` already reuses prior artifacts. A premium answer is reused for the same company and question while its as-of date is within a freshness window (default 30 days; funding facts go stale faster than market sizes). A fresh run (`fresh_requested`) re-asks.

### 5. Export rules for licensed content

`internal_only` already decides whether a source is citable outside the firm. For premium sources it also decides what leaves the building:

- **Inside the firm** (the partner's memo, IC deck, the desktop app): premium facts and citations render normally.
- **Outside the firm** (LP updates, a shared Google Doc, `export_cdn`, anything public): premium-sourced sentences are flagged at export. The analyst either removes them, rewrites them without the licensed specifics, or confirms the firm's license allows it. The export logs the decision.
- **Never in MemoPop's own marketing or example memos.** Anything shown on `memopop-site` uses open-web sources only.

### 6. Routes, in the order we build them

1. **Analyst paste.** An analyst asks in the Perplexity app (`@PitchBook …`) and pastes the answer into the deal's inbox. This reuses the `analyst-paste` path with `origin: premium`. No new integration, so it's the right first test of capture, precedence and export.
2. **Local Claude Code with Computer MCP.** When `MEMOPOP_RESEARCH_PROVIDER` is `claude-code` and the operator's Claude Code session has the Computer MCP server connected, the premium questions become one Computer task per company. Use `incognito: true` so deal questions stay out of the user's Perplexity history and memory, and handle `auth_required` by surfacing the sign-in link.
3. **The connector route.** In the hosted MemoPop Connector (`apps/memopop-orchestrator/context-v/specs/MemoPop-Connector-API.md`), the partner's own Claude drives MemoPop's steps. If the partner's Claude also has the Computer MCP server, the premium questions run in the partner's session, on the partner's seat and entitlements. MemoPop never holds the partner's Perplexity credentials. This is the cleanest answer to "whose seat", so it's the default for hosted use.
4. **Partner MCP through the Agent API.** For a firm with a partner's MCP license (FactSet, Morningstar), register the partner server as an Agent API custom connector in the firm's Perplexity project. This is the only route that can run on a schedule without a person.
5. **Native Agent API connectors**, if Perplexity ever exposes premium sources there. Capture is unchanged.

## Quality bar (ranked)

1. **Every premium fact traces to a source file** with its provider, question, date and account. Nothing from a premium answer reaches prose any other way.
2. **Licensed content never leaves the firm unflagged.** An export containing an `internal_only` premium sentence can't complete without an analyst decision.
3. **The dataroom still wins** on terms, cap table and financials.
4. **Conflicts are shown, not resolved silently.** When PitchBook and the web disagree on a round, the memo or the review surface says so.
5. **No surprise spend.** A run never exceeds its query cap, and the run log shows how many premium queries it used and on whose account.
6. **Off by default.** A firm with no `premium_sources` block runs exactly as today.

## Related risk: the Sonar endpoint

The orchestrator calls Perplexity's `chat/completions` with `sonar-pro` in seven agents (`perplexity_section_researcher`, `research_enhanced`, `competitive_landscape_researcher`, `competitive_landscape_evaluator`, `citation_enrichment`, `fact_verifier` and `visualization_enrichment`). Perplexity ended support for Sonar Chat Completions on 2026-09-27 and now rewrites those requests into Agent API requests, rolling out model by model. Output may already differ from what these agents were tuned on, and the Agent API's `low`, `medium` and `high` effort levels cite as `[web:1]` rather than `[1]`. That migration is its own piece of work, but it touches the same agents this spec does, so the two should be planned together. See the Perplexed exploration [What Perplexity's 2026 API Could Add to Perplexed](https://github.com/lossless-group/perplexed-plugin/blob/development/context-v/explorations/What-Perplexitys-2026-API-Could-Add-to-Perplexed.md).

## Open questions

- **Whose seat in a local run?** Route 2 uses the operator's Perplexity account. Is that acceptable for client work, or does each firm need its own Perplexity seat before route 2 is allowed?
- **What do licenses actually allow?** PitchBook, Statista and CB Insights terms for quoting in a memo shared with an investment committee, co-investors or LPs. This needs the actual agreements, not a guess. Until then, `internal_only` and the export flag are the safe default.
- **How deep is "Essentials"?** Run one company we know well through PitchBook Essentials and compare it with `crawl-fetch-ingest` and the dataroom before relying on it.
- **Freshness windows per fact type.** Is 30 days right for market sizes and too long for funding?
- **Can a Computer task be made deterministic enough** to diff across memo versions, or is reuse the only way to keep versions stable?
- **Should the Sources.md review surface show premium sources as a separate block** so analysts see the licensed tier at a glance?

## Phases

1. **Prove capture.** Analyst paste with `origin: premium` on one live deal. Verify Sources.md, extraction, citation assembly and the export flag end to end.
2. **Precedence and budget.** The per-fact-type order in `source_priority`, the per-run cap, and reuse through `reuse_gate`.
3. **Computer MCP, local.** Route 2 behind `premium_sources.route: computer-mcp`.
4. **Connector route.** Premium questions as part of the step registry, run in the partner's session.
5. **Unattended.** Partner MCP through the Agent API, for firms that hold those licenses.

## Related

- [Extract Sources Where Possible, with Fallback to Ground Content](Extract-Sources-Where-Possible-with-Fallback-to-Ground-Content.md)
- MemoPop Connector API (`apps/memopop-orchestrator/context-v/specs/MemoPop-Connector-API.md`)
- [Self-Generating Deal Scout for VC Opportunity Scans](https://github.com/lossless-group/lossless-content/blob/development/specs/Self-Generating-Deal-Scout-for-VC-Opportunity-Scans.md), which uses the same premium sources upstream of the memo
- [What Perplexity's 2026 API Could Add to Perplexed](https://github.com/lossless-group/perplexed-plugin/blob/development/context-v/explorations/What-Perplexitys-2026-API-Could-Add-to-Perplexed.md), including its premium data sources addendum
- `crawl-fetch-ingest` and `vc-firm-profile-ingest` skills, the open-web enrichment this complements
