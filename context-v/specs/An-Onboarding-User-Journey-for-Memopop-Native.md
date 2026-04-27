# An Onboarding User Journey for memopop-native

> Draft — captures the working understanding from session 2026-04-27. Not yet aligned. Edit freely.

## The Question This Spec Answers

What does the first 30 minutes with memopop-native feel like for a VC who hasn't used a terminal in years and has no investment in this tool yet?

## Who We're Designing For

Not Mike. Not other technical operators who already know what the orchestrator is. **Other people** — VCs, partners, principals at firms who:

- Hear about MemoPop from a friend / a webinar / a tweet.
- Want to see what it does.
- Will not read a README cover-to-cover.
- Will not fight a Python venv.
- Will give the tool somewhere between 5 and 15 minutes of patience before they decide whether it's worth more time.

The implicit assumption: they have an AI assistant available (Claude, ChatGPT, Cursor, whatever) and are willing to use it for the parts they can't do themselves. So "have your AI install the orchestrator" is a viable instruction; "follow this 12-step terminal walkthrough" is not.

## The Failure Mode We're Avoiding

The configure-first wall. Most internal tools onboard like this:

1. "Welcome! First, set up your account."
2. "Now configure your data sources."
3. "Now choose your preferences."
4. "Now invite your team."
5. *(user already gone)*

By the time the user sees what the tool actually does, they've made twelve decisions about something they haven't yet seen the value of. For a busy VC: drop-off.

We're inverting this. **Configuration is a tax we charge the user only at the moment they want to do something — never before.**

## The North Star

Five minutes after a VC double-clicks the app for the first time, they are looking at one of our sample outlines, scrolling through ten dense sections of guiding questions and preferred sources, thinking *"oh shit — this is more comprehensive than what we pay for."* That sentence is the entire onboarding goal.

Everything before that sentence is overhead to minimize. Everything after that sentence is them choosing to lean in — which is when we can ask them for things.

## The Journey

### Breadcrumbs and Progress Bar

The onboarding flow should have a clear breadcrumb trail and progress bar to show the user where they are in the process. This helps to reduce anxiety and provides a sense of accomplishment as they progress through the onboarding flow. It also helps to know they're "almost done" and can see value quickly.

### Pre-flight (happens outside the app)

The user has to clone the orchestrator from GitHub before the app can do anything. We can't avoid this — the app *is* a thin shell over the Python orchestrator, and the orchestrator is real software with dependencies.

What we can do:

- **Make the GitHub link prominent in onboarding.** Already done.
- **Tell them to use their AI assistant if they get stuck.** Already done.
- **Trust the orchestrator's README to do the rest.** Out of scope for the GUI spec, but worth noting: the orchestrator README should be optimized for "I am pasting this into Claude" — readable, complete, no hidden steps.

If the user can't get this far, no UI work in our app saves them. This is a hard floor.

### Step 1 — Anchor the orchestrator (one-time)

First launch. The app shows a single panel:

- "Welcome to MemoPop."
- "You'll need the orchestrator installed locally. If you haven't yet: [GitHub link]. Either follow the README or hand it to your AI assistant — should be straightforward with a little patience."
- A "Browse" button.

User clicks Browse, picks the directory where they cloned the orchestrator. The app verifies it has the expected shape (e.g., a `templates/outlines/` directory exists) and remembers the path.

This is the only required configuration step before the user sees anything valuable. It is a tax we cannot avoid charging — the app physically cannot read outlines without knowing where they are.

### Step 2 — Land in the outline gallery

After anchoring the orchestrator, the user lands on the **outline gallery**. This is the home view from this moment forward.

The gallery is a visually rich grid of cards, one per outline file in `templates/outlines/`. Each card shows enough detail that the user can size up the outline at a glance:

- **Title** of the outline (e.g., "Standard Direct Investment", "Hypernova Fund Commitment").
- **Type badge**: Direct Investment / Fund Commitment.
- **One-line description** of who it's for.
- **Section count** ("10 sections, 47 guiding questions").
- **A taste of preferred sources** ("Includes Statista, CB Insights, PitchBook, ...").
- **A "Try this on a company →" CTA.**

> [!NOTE]
> The gallery may require we add new properties to the outline, such as a hero image or a more brief or more detailed description, Tags, etc. to make it feel impressive. The point is that the user thinks *"these are not toy templates."*

Clicking a card should optionally let the user **expand into a detail view** — full section breakdown, guiding questions per section, validation criteria, vocabulary notes. This is the "scroll through ten dense sections" moment in the North Star.

The user can browse without committing. No firm, no deal, no friction. They are shopping.

### Step 3 — Convert browsing into commitment

The user picks an outline they want to try. They click **"Try this on a company →"**.

This is the moment we charge them the next tax — but not before.

### Step 4 — Lightweight firm creation (only if needed)

When they click the CTA, we check: does the user have any firm under `io/`?

- **If yes** (e.g., they used the app before) — proceed straight to Step 5 with their active firm.
- **If no** — show a small inline prompt:

  > **Set up your firm**
  >
  > We'll create a private space for the work you do in MemoPop. Everything you generate stays in `io/{your-firm-name}/` and is automatically marked private — no risk of committing client data to a public repo.
  >
  > **What's your firm called?** [_________________]
  >
  > *Tip: The conventional name, often in short form. Often not the legal name or even the longer name. e.g., "Hypernova" instead of "Hypernova Capital".*
  >
  > [ Cancel ] [ Continue → ]

  The user types something like `Hypernova`, `Acme Capital`, or `Black Rock Ventures`. We:

  1. **Preserve their original input** as the firm's *conventional name* (with whatever casing and spacing they typed).
  2. **Normalize the input to snake_case** for the directory name — lowercase, spaces → underscores, strip punctuation. Show the resulting `io/{snake_case_name}/` path back to the user as a small confirmation line so there are no surprises.
  3. **Create the firm filesystem skeleton** (see below).
  4. **Add the firm folder to the orchestrator repo's `.gitignore`** so deals never land in a public commit.
  5. **Set this firm as active.**
  6. **Continue to Step 5.**

This is a small modal, dismissible, not a page-turn. The user is still mentally in the gallery — they're not "doing onboarding," they're "trying an outline."

### Step 4a — Firm filesystem skeleton (what gets created)

The app creates the **minimum viable** structure under the orchestrator's `io/` directory. Everything else is created lazily — when the user actually runs something that needs it.

```
io/
└── {snake_case_firm}/                                # e.g., io/hypernova/
    ├── configs/
    │   └── brand-{snake_case_firm}-config.yaml       # e.g., brand-hypernova-config.yaml
    └── deals/                                         # empty until first memo
```

Filename convention: `brand-{snake_case_firm}-config.yaml`. This matches the existing pattern across firms already in the orchestrator (e.g., `brand-alpha-partners-config.yaml`, `brand-hypernova-config.yaml`).

The brand config file is a **stub** — the only field populated is the firm name as the user typed it:

```yaml
# io/{snake_case_firm}/configs/brand-{snake_case_firm}-config.yaml
# Firm identity stub. Real branding (colors, fonts, logos, tagline) is filled
# in by the "Brand setup" flow — not part of onboarding.

company:
  conventional_name: "Hypernova"   # exactly what the user typed
                                   # — short, common-usage name (e.g., "Sequoia", not "Sequoia Capital")
                                   # — preserves casing and spacing the user used
```

The full schema (filled in later by Brand setup, not now) follows the existing convention, with one addition — the firm-naming model splits into three layers:

```yaml
company:
  # Three names, three purposes:
  conventional_name: "..."     # the short, common-usage name — the one humans actually say
                               # e.g., "Sequoia", "Fifty Years", "Hypernova"
                               # USED EVERYWHERE in the UI by default
                               # collected at onboarding (the only one that is)

  name: "..."                  # the official name — the firm's full public identity
                               # e.g., "Sequoia Capital", "Fifty Years Ventures"
                               # collected later, only if needed for memo headers / footers

  legal_entity_name: "..."     # the legal entity — for disclosures and legal language
                               # e.g., "Sequoia Capital Operations LLC"
                               # collected later, only if the firm uses memos as legal documents

  tagline: "..."
  confidential_footer: "This document is confidential and proprietary to {company_name}."

colors:                # light mode
  primary: "..."
  secondary: "..."
  accent: "..."
  text_dark: "..."
  text_light: "..."
  background: "..."
  background_alt: "..."

colors_dark:           # dark mode (mirrors `colors`)
  ...

fonts:
  family: "..."
  fallback: "..."
  google_fonts_url: "..."
  weight: 400
  header_family: "..."
  header_fallback: "..."
  header_fonts_dir: "io/{snake_case_firm}/assets/fonts"
  header_weight: 700

logo:
  light_mode: "..."    # URL or local path
  dark_mode: "..."
  width: "180px"
  height: "60px"
  alt: "..."
```

Onboarding only writes `company.conventional_name`. **Everything else is deferred until after the user has run their first memo.** That includes:

- Official `company.name` and `company.legal_entity_name`
- Tagline and confidential-footer text
- Colors (light + dark)
- Fonts (body + headers)
- Logo URLs/files

This is not laziness — it's strategy. The path to a hooked user is: **see the outlines → run a memo → look at the output → care about how it looks → tell us how to make it look right.** Asking for a logo SVG or a hex color in onboarding loses 90% of the audience. Get them attached to the content first; iterate on identity later.

When the user is ready, **Brand setup** is its own guided flow (see "What's NOT in This Spec"). One promising version of that flow is an agent that crawls the firm's website and extracts the brand info automatically — paste a URL, get a populated config — so the user never has to upload an SVG by hand.

**Lazy creation of other firm directories.** Existing firms in the orchestrator also have `assets/`, `templates/`, `versions.json`, per-firm `README.md`, and per-firm `.gitignore`. These are **not** created during onboarding:

- `assets/` — created by Brand setup when the user uploads fonts or logos.
- `templates/` — created if/when the firm customizes outlines (Phase 2 work).
- `versions.json` — created automatically by the orchestrator on first memo run.
- `README.md`, per-firm `.gitignore` — skipped entirely. Not load-bearing for the GUI's flow.

Keeping the initial skeleton this small is deliberate. We're not pre-staging structure the user hasn't earned the right to know about yet.

### Step 4b — `.gitignore` handling

The app appends a line to the orchestrator repo's `.gitignore` so the firm's directory and everything inside it stays uncommitted:

```
# Added by MemoPop — firm data is private
/io/*/
```

We use the wildcard form (`/io/*/`) rather than per-firm entries because:

- It's a one-time write — adding multiple firms doesn't accumulate `.gitignore` cruft.
- It's a clear declaration: *"any firm folder is private by default."*
- A future user adding a firm by hand (outside the app) is also covered.

If the line is already present (e.g., the user has used the app before), this is a no-op.

### Step 5 — Hand off to the action

Once the firm exists and the outline is chosen, we collect just enough to run a memo:

- **What company are you analyzing?** (text field) OR EVEN BETTER, just a url. Sometimes we need to collect both to disambiguate. 
- Optional, but recommended:
  - **Pitch deck (optional).** Drag-and-drop a PDF, or skip.
  - **Stage / type / mode** — pre-filled from the chosen outline's defaults; user can override.
  - **Mode** - ENUM list of "Justify" or "Evalute" (I forget what the exact syntax is for the enum) -- some VCs want to write a memo to justify why they invested, others want to write a memo to evaluate whether they should invest. Very few want to do both.

Submit kicks off the orchestrator. The user lands on a job-status view with streaming logs. This part is out of scope for *this* spec — the action runner is its own thing — but the handoff from onboarding to running needs to feel seamless. THIS PART IS NEXT, BECAUSE THE USER NEEDS TO SEE THE OUTPUT, THE SPEED, THE THOROUGHNESS, ETC.

### After the first run

From this point forward:

- **The home view stays the outline gallery.** "Run another memo" is always one click away.
- **The firm picker becomes a switch-firm affordance** in the header or a sidebar — not a gate. Most users will only ever have one firm.
- **Recent memos** could appear on the gallery as a secondary section ("Last week you ran X, Y, Z"). Worth considering.
- **Settings** is reachable from a menu — repo path, current firm, future tunables. Not the home.

## What This Means for the App's Architecture

The current build (after sessions 2026-04-27_01 and _02) treats Settings as the home. The orchestrator path and the firm picker are foreground. **That's the wrong frame for new users.**

Concrete changes implied:

- **The home route stops being `<Settings />`** — it becomes `<OutlineGallery />` (with a guard: if no orchestrator path, render the path-anchoring panel; once anchored, render the gallery).
- **Settings moves to a secondary route** like `/settings`, reachable from a sidebar or menu.
- **The firm dropdown** comes out of the home view. It either lives in the header (compact "active firm" indicator with a dropdown to switch) or in Settings as a list.
- **The empty-firm-list state**, currently a dead end in our UI, becomes irrelevant — new users never see it. Their first firm is created mid-flow at Step 4.
- **A new query is needed**: `GET /outlines` — returns the list of outlines from `templates/outlines/` with parsed metadata (title, type, section count, source hints).
- **A new action is needed**: `POST /actions/create-firm` — takes a firm name, slugifies it, creates the directory, edits `.gitignore`, returns the canonical slug.

## What We've Already Built (and How It Changes)

We have:

- A Settings page at the home route with orchestrator-path picker + firm dropdown.
- Onboarding copy explaining the orchestrator + private-firm-repo concept.
- A working Transport layer + the `GET /firms` query.
- Persistent settings via the `store` plugin.

What stays:

- All of the above mechanics. Transport, persistence, the path picker, the existing onboarding copy.

What changes:

- The Settings page moves out of the home route. Home becomes the gallery.
- The onboarding copy may shorten — once the gallery is the landing, we don't need to explain the firm-repo concept *up front*. We can introduce it at Step 4 when it's actually relevant.
- The current onboarding still applies for **Step 1** (anchoring the orchestrator). That panel still appears as a gate when no path is set. Nothing wasted.

## Open Questions

1. ~~**`.gitignore` strategy.**~~ **Resolved:** `/io/*/` with a header comment. See Step 4b.

2. ~~**Firm name slugification visibility.**~~ **Resolved:** Show the user the resulting `io/{snake_case_name}/` path as a confirmation line before they hit Continue. See Step 4.

3. **Outline detail view: modal, side-panel, or full-screen?** Modal feels lightweight and lets the user back out without losing their place in the gallery. Full-screen feels more substantial and matches the "this is comprehensive" goal. Side-panel is the compromise.

4. **Outline gallery: what counts as "compelling"?** What metadata, when rendered as a card, makes a VC say *"oh shit, comprehensive"*? Need to actually look at one or two outline YAMLs and decide what to surface.

5. **What happens if the orchestrator's `templates/outlines/` is empty or missing?** Edge case. Probably an error state that says "Hmm — this folder doesn't look like the orchestrator. Did you pick the right directory?" Should not be fatal.

6. **Recent memos on the gallery.** Worth showing for return users? Or keep the gallery focused on outline-shopping and put recents in Settings or a separate view?

7. **Multi-firm switching for power users.** When does a user have more than one firm? Probably never for individual VCs, sometimes for consultants who advise multiple firms. Not a v1 problem, but the architecture (active firm in app settings, switch via header dropdown) should accommodate it without rework.

8. **Onboarding state machine vs implicit gates.** I've described the journey as a sequence (Step 1 → 2 → 3 → 4 → 5), but the UI doesn't necessarily need a literal state machine. Each gate is just a conditional render. Worth deciding whether we ever need to track "where in onboarding is this user" explicitly, or whether the app's *current state* (orchestrator path set? firm exists? deal in flight?) tells us everything we need.

## What's NOT in This Spec

- The actual memo-generation flow (Step 5 and beyond). That's a separate spec — the action runner, the log streaming, the artifact viewer.
- Outline editing / customization. The "copy this outline so my firm can edit it" feature. Phase 2 work.
- **Brand setup flow.** Filling in everything beyond `company.conventional_name`:

> Any extra work before the moment of value propisition and experiencing the value would discourage users from finishing the Onboarding walkthrough.  What could be considered more basic data in the initial Onboarding, such as  official `company.name`, `company.legal_entity_name` (for memo disclosures), tagline -- is still too much in the beginning. They will fill it out later when they have incremental goals in trying to generate GREAT exported memos.  Things like confidential-footer text, light + dark color palettes, body and header fonts, light/dark logo URLs or paths, can all be captured as they seek a shareable memo.  

> Onboarding stubs the file with `company.conventional_name` only — that's all we need to render the firm's identity in the UI for first-run memos. Brand setup happens *after* the user has run their first memo and wants their output to look polished. It is its own spec; the most promising version of that flow leans on the auto-fetch capability described below rather than asking the user to upload an SVG.

- **Brand auto-fetch (and the broader "agent-driven information extraction" pattern).** A future agent that takes a firm's website URL and returns a populated brand config — light + dark colors sampled from the site, fonts identified from CSS, logo files extracted, tagline scraped from the homepage. This same kind of agent will be needed for **deal analysis** (target company website, public filings, news) — so the auto-fetch capability is shared infrastructure across firm onboarding and deal generation, not a brand-only feature. Out of scope here, but worth flagging that Brand setup's UX explicitly assumes this exists. *"Paste your firm's URL, we'll do the rest"* beats *"Now upload your light-mode logo SVG."* Every time.
- Pitch deck handling. The drop-and-parse of PDF decks.
- Any concept of multi-user, auth, or sharing.
- Settings beyond orchestrator path + active firm. (API keys, model preferences, default outline per firm, etc. — later.)
- Branded export / scorecards / one-pagers. All separate features that come after the user has run their first memo.

The boundary of this spec is: **the user has just clicked "Try this on a company →" and hand-off to the runner is about to happen.** Everything after is someone else's spec.
