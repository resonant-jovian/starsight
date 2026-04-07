---
name: starsight-docs
description: "Maintain, merge, refine, and generate the starsight documentation suite: STARSIGHT_NEW.md (development reference with checkboxes and ASCII diagrams) and LEARN.md (TTS-compatible teaching document). Use this skill whenever working on starsight documentation files, merging Part 1 into LEARN.md, creating ASCII diagrams, deduplicating chapters, adding visual improvements, or performing quality checks on either file. Also triggers for any resonant-jovian ecosystem documentation work including chromata, prismatica, caustic, or phasma docs."
---

# Starsight Documentation Agent

This document contains everything needed to maintain the starsight documentation suite. It encodes decisions, procedures, constraints, and quality standards accumulated across 8+ sessions.

## The Documentation Suite

Three output files:

| File | Words | Purpose | Constraints |
|---|---|---|---|
| `STARSIGHT_NEW.md` | ~28,000 | Development reference: ordered checkboxes, code blocks, ASCII diagrams, tables | Rich markdown, 338 checkboxes, balanced fences |
| `LEARN.md` | ~34,000 | Teaching document: every concept in plain prose | **TTS-clean: 0 backticks, 0 code fences, 0 pipes, 0 tables** |
| `LEARN_CHEATSHEET.md` | ~750 | One-page quick reference | Dense prose, checkable facts |

## Critical Context

The user uploads the SAME original unmerged files every session:
- `STARSIGHT_NEW.md` (~51,548w) — has Part 1 Listen, Part 2 Build, Part 3 Look up, Part 4 Navigate
- `LEARN.md` (~58,614w) — original with 154 sections and massive internal duplication (~30,902w)

The merge MUST be redone from scratch each time. Never assume the files are pre-merged.

## The Merge Procedure

### Step 1: Extract Part 1 from STARSIGHT

Part 1 ("Listen") is pure prose that belongs in LEARN.md, not STARSIGHT. Extract it, remove it, renumber remaining parts.

```python
p1s = ss.index('\n# Part 1 — Listen\n')
p2s = ss.index('\n# Part 2 — Build\n')
part1 = ss[p1s:p2s]
rest = ss[p2s:]
# Renumber: Part 2→1, Part 3→2, Part 4→3
# Replace all "Part 1 Listen" references with "LEARN.md"
```

### Step 2: Deduplicate LEARN.md

The original LEARN.md has ~154 sections with ~30,902 words of duplication. Keep these line ranges:

| Keep | Lines | Content |
|---|---|---|
| Intro + Ch 1-12 | 0–477 | Core Rust + graphics |
| Ch 13-18 | 751–1055 | Visualization theory |
| Ch 19-36 + appendix | 2002–2860 | Architecture + tooling |
| Glossary | 2861–3035 | Term definitions |

Drop lines 478–750, 1056–1522, 1523–2001, 3036–end (all duplicates).

### Step 3: Integrate Part 1 Unique Content into LEARN

Extract subsections from Part 1 (using `### ` and `## 1.` headers) and insert at these locations:

| Content | Insert after chapter about... | Position |
|---|---|---|
| Scene graph, Builder patterns, Thread safety | Design patterns | after |
| API design, Code standards, Versioning | Workspace structure | after |
| First coding session, Managing complexity, 0.1.0 MVP, Debugging | Building starsight | before |
| Edition 2024, GPL-3.0, No async, Ecosystem positioning, Accessibility | resonant-jovian ecosystem | after |
| After 1.0, DPI handling, Xtask pattern, Default themes | Long-term maintenance | after |

**String escaping hazard**: The `Don'ts` section key contains an apostrophe. Use `donts = g("Don'ts")` then reference the variable.

### Step 4: Add Learning Roadmap

Insert before the first `## Chapter` heading:

> Five arcs. Chapters 1–17: Rust. Chapters 18–21: graphics. Chapters 22–28: visualization theory. Chapters 29–45: architecture. Chapters 46+: supplementary. Glossary at end.

Include skip-ahead instructions for experienced readers.

### Step 5: Remove Duplicate Appendix Chapters

These titles appear twice after integration — remove the duplicates:
- "How to handle the edge cases that break charts"
- "How the resonant-jovian ecosystem creates a unified"
- "How DPI and resolution affect chart output"

### Step 6: Renumber All Chapters Sequentially

Strip existing numbering (both word-form "Twenty-Two" and numeric), renumber as `## Chapter 1: Title` through `## Chapter 66: Title`.

### Step 7: Restore Glossary

The glossary sometimes gets lost during deduplication. Explicitly extract from original lines 2861–3035 and append with `## Glossary` header.

### Step 8: Verify TTS Compliance

```python
assert learn.count('`') == 0      # zero backticks
assert learn.count('|') == 0      # zero pipes (no tables)
assert '```' not in learn          # zero code fences
```

## Visual Improvements for STARSIGHT

These sections are added to the merged STARSIGHT_NEW.md. Each must be created fresh.

### Part 1 — Build (insert before Pre-0.1.0)

1. **Quick start**: 3-line `plot!` macro example
2. **Call trace**: ASCII tree showing `plot!` expansion through all layers
3. **Migration table**: 8-row "Coming from another language?" (matplotlib/seaborn/ggplot2/plotly → starsight)
4. **Milestone progress table**: Done/Todo/Total per milestone

### Part 2 — Look up (insert before tiny-skia section)

5. **Import quick reference**: "by task" table (11 rows) + "by layer" table (internal crate imports)
6. **Data-to-pixel pipeline**: ASCII flowchart showing DATA → Figure → Marks → Coord → Backend → OUTPUT
7. **Coordinate cheat sheet**: ASCII diagram with Y-inversion formula
8. **Common recipes**: 6 copy-paste code patterns (line, two series, scatter, SVG, theme, colormap)
9. **Feature flags table**: 12 features with dependency and default columns
10. **Troubleshooting table**: 8 common errors with fixes

### Part 3 — Navigate (insert before crate dependency graph)

11. **Layer architecture**: Nested box diagram showing all 7 layers + backend grid
12. **Type flow**: User input types → Layer 5 → internal L1/L2 types
13. **"Which crate do I edit?" table**: 23-row task-to-file mapping
14. **Examples directory layout**: ASCII tree with 14 examples

## ASCII Diagram Style Guide

Inspired by diagrams from Chromium, Linux, LLVM, TensorFlow codebases (see asciidiagrams.github.io). Use Unicode box-drawing characters:

**Characters**: `┌ ┐ └ ┘ │ ├ ┤ ┬ ┴ ┼ ─ ═ ▼ ▲ ► ◄ ●`

**Patterns used**:

- **Tree**: `├──` for intermediate items, `└──` for last item, `│` for vertical continuation
- **Boxes**: `┌───┐` / `│   │` / `└───┘` for containers
- **Double-line**: `═` for major boundaries (facade crate border)
- **Arrows**: `▼` for downward flow, `──▶` for horizontal flow
- **Nesting**: Inner boxes indented 3 spaces inside outer boxes
- **Annotations**: Right-aligned layer labels, inline descriptions after `──`

**The architecture diagram** is the most complex: outer facade box contains L7/L6 side-by-side, L5–L1 stacked vertically, L1 has a 6-cell backend grid, separated `├───┤` footer for xtask.

**Width**: No constraint — use whatever width makes the diagram clear and information-dense. The user explicitly requested wide diagrams.

## Existing ASCII Block Modifications

The crate dependency graph and file tree in the original STARSIGHT also need cleanup:

- **Crate dependency graph**: Already present, just needs consistent tree formatting
- **File tree**: Trim long right-aligned CAPS descriptions (e.g., "RENDERING + PRIMITIVES + ERROR"), condense `[exists]`/`[target]` annotations

## Assembly Order

```python
content = TITLE_BLOCK + base_without_part1.lstrip()
# Insert quick start before "## Pre-0.1.0"
# Insert P2 additions before "## tiny-skia"
# Remove old "## Feature flag reference", insert new before "## Links"
# Insert P3 additions before "## Crate dependency graph"
# Remove old "## Quick reference: which crate do I edit?" if new exists
# Clean up: collapse 4+ blank lines to 2
```

## Verification Checklist

Run after every assembly:

```python
# STARSIGHT
assert cb == 338                          # checkbox count preserved
assert fc % 2 == 0                        # code fences balanced
assert len(secs) == len(set(secs))        # all section titles unique
assert stale_refs == 0                    # no "Part 1.*Listen" references
# All 14 visual improvements present
for c in ['Quick start', 'Coming from', 'Milestone', 'Import quick',
          'Data-to-pixel', 'Coordinate', 'Common recipes', 'Feature flags',
          'Troubleshooting', 'Layer architecture', 'Type flow',
          'Which crate do I edit', 'Examples directory']:
    assert c in content

# LEARN
assert learn.count('`') == 0             # TTS clean
assert '## Glossary' in learn            # glossary present
assert len(chapters) == 66               # chapter count
assert 'How to use this document' in learn  # roadmap present
```

## Target Sizes

| File | Words | Key Metric |
|---|---|---|
| STARSIGHT_NEW.md | ~28,000 | 338 checkboxes, ~41 sections, ~790 balanced fences |
| LEARN.md | ~34,000 | 66 chapters + glossary, 0 backticks/pipes/fences |
| Combined | ~62,000 | Down from 110,162 original = 44% reduction |

## Key Technical Facts

These verified facts must be preserved in the documentation:

- tiny-skia 0.12 renamed `ClipMask` → `Mask`
- `Color::from_rgba()` returns `Option`; `from_rgba8()` does not
- cosmic-text R↔B channel swap applies only to softbuffer display, not PNG/SVG
- Wilkinson Extended tick algorithm has no existing Rust implementation
- `Point + Point` does not compile (no `Add<Point> for Point`)
- `Point - Point = Vec2` (displacement between positions)
- Edition 2024, resolver 3, GPL-3.0-only
- MSRV 1.85
- Wilkinson tick weights: simplicity 0.2, coverage 0.25, density 0.5, legibility 0.05
- premultiplied alpha: `(r×a, g×a, b×a, a)` — tiny-skia uses this internally
- WCAG contrast ratio: `(L1 + 0.05) / (L2 + 0.05)`, minimum 4.5:1

## User Preferences

- **Naming**: Latin/Greek-rooted scientific words (chromata, prismatica, caustic, starsight)
- **Communication**: Terse, directive, short imperative instructions
- **Docs**: Single consolidated files preferred over fragments
- **ASCII**: Wide and detailed, Unicode box-drawing, no width constraints
- **Output**: Prefers options and artifacts over lengthy explanations
- **Learning format**: TTS-safe prose (no bullets, bold, or tables) for audio playback
- **OS**: Arch Linux, GitButler for version control, Inkscape for SVG

## The resonant-jovian Ecosystem

| Crate | Status | What it is |
|---|---|---|
| starsight | Scaffolded, 338 TODOs | Unified Rust visualization library |
| chromata | Working, unpublished | 1,104 editor/terminal color themes as compile-time constants |
| prismatica | Working, unpublished | 260+ perceptually uniform colormaps as compile-time LUTs |
| caustic | Active | 6D Vlasov–Poisson solver |
| phasma | Active | Terminal UI for caustic |

## File Paths

- User uploads: `/mnt/user-data/uploads/STARSIGHT_NEW.md`, `/mnt/user-data/uploads/LEARN.md`
- Working: `/home/claude/SS.md`, `/home/claude/LN.md`, `/home/claude/SS_BASE.md`
- Parts: `/home/claude/parts/` (title.md, p1_qs.md, p2_add.md, p2_feat.md, p3_add.md)
- Output: `/mnt/user-data/outputs/STARSIGHT_NEW.md`, `/mnt/user-data/outputs/LEARN.md`

## Common Pitfalls

1. **Glossary disappears**: The deduplication step can lose the glossary. Always explicitly extract and append it.
2. **Apostrophe in "Don'ts"**: Breaks Python f-strings. Use variable assignment.
3. **Stale Part 1 references**: After removing Part 1, grep for `Part 1.*Listen` and replace with `LEARN.md`.
4. **LEARN backtick leak**: Part 1 content contains backticks. All backticks must be stripped when inserting into LEARN.
5. **Chapter renumbering**: Both word-form ("Twenty-Two") and numeric ("22") numbering exist in the original. The regex must catch both.
6. **Duplicate appendix chapters**: Three appendix topics appear in both the original LEARN and the inserted Part 1 content. Remove the duplicates.
7. **File tree width**: The original file tree has lines up to 104 chars. Trim annotations to keep reasonable width.
