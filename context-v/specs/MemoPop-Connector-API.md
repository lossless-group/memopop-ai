---
type: Specs
title: "MemoPop Connector API"
lede: "Eight tools, one step registry, and the docs and tests that ship with each. What we build so a partner's Claude can write a MemoPop memo."
publish: false
date_created: 2026-10-10
date_modified: 2026-10-10
date_authored_initial_draft: 2026-10-10
date_authored_current_draft: 2026-10-10
date_authored_final_draft:
authors:
  - Michael Staton
augmented_with:
  - Claude Code on Claude Opus 5.5 (1M context)
at_semantic_version: 0.0.0.2
status: Moved
category: Specification
site_uuid: f32ada52-1cdb-4348-9170-42eebdbc9bd3
hex_code: 2blght
summary: >-
  Spec of record for v1 of the MemoPop connector: a remote MCP server (and
  matching versioned REST API) at memopop.didi.sh that hands a client's Claude
  the memo method one step at a time and keeps every artifact. Defines the
  concepts, the step registry and how today's LangGraph nodes map onto it, the
  eight tools with inputs, outputs, and errors, the error envelope, versioning,
  auth, storage layout, the agent-facing documentation set (with a worked
  example entry and the generation targets), and the full test suite from
  registry lint to the live health check. Built on the eight decisions in
  context-v/decisions/. Ends with the plan split and the questions that are
  still open. Read before writing any plan or code for the connector.
tags:
  - MemoPop
  - Claude-Connector
  - Model-Context-Protocol
  - Hosted-API
  - API-Docs-For-Agents
  - Testing
  - Specification
---

# MemoPop Connector API (moved)

This spec now lives beside the code it governs, in the orchestrator:
`apps/memopop-orchestrator/context-v/specs/MemoPop-Connector-API.md`
([on GitHub](https://github.com/lossless-group/investment-memo-orchestrator/blob/development/context-v/specs/MemoPop-Connector-API.md)).

It moved on 2026-10-10 so the orchestrator's test ledger and CI, which clone
that repo alone, can read its test table. Its plans and loop are there too.
The decisions it rests on stay here, in `context-v/decisions/`.
