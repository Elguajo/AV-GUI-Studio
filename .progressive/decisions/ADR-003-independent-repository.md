# ADR-003 — GUI Studio as an independent product in this repository

Status: Accepted
Date: 2026-11-24
Supersedes: the "Fork as base" decision in [ADR-002](ADR-002-figcraft-base.md). All other ADR-002 decisions still apply: the Extron crate, metadata on nodes, mapping, Problems, branding, no upstream contributions and the retired eframe editor.

## Context

ADR-002 built GUI Studio as a downstream fork of FigCraft in a separate local checkout, `../gui-studio-fork`. That fork tracked the `upstream` remote and was not published. The owner wants one repository, [Elguajo/GUI-Studio](https://github.com/Elguajo/GUI-Studio), for the product and its specs. GUI Studio should be independent of FigCraft, and only chosen FigCraft improvements should be brought in. GUI-Studio is a public repository.

## Decision

- **One repository.** The FigCraft-based editor lives at the root of this repository. The Cargo workspace, `crates/`, `apps/`, `xtask/` and the FigCraft docs sit beside `.progressive/`. The import is a snapshot of fork commit `97bbf0e3`, which is FigCraft `b828e6c7` plus the GUI Studio rebrand and Extron work. FigCraft's commit history is not imported.
- **Independent product.** GUI Studio does not merge FigCraft `main`. Upstream improvements are ported on purpose: fetch FigCraft into a local `figcraft` remote, then `git cherry-pick` or re-implement the chosen change in its own commit. Each commit names the upstream commit or PR. FigCraft's paths are kept as they are, so cherry-picks still apply. Renaming crates or moving directories is a separate decision, made when it pays off.
- **Repository-level files.** This repository's `AGENTS.md`, `CLAUDE.md` and `.progressive/` stay authoritative. FigCraft's agent rules and roadmap are kept for reference in `docs/figcraft-agents.md` and `docs/figcraft-roadmap.md`. FigCraft's `.github` workflows and dependabot, CodeRabbit and Sonar settings target the storytold organisation and are not carried over. GUI Studio adds its own CI in a separate change.
- **Licences.** `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE` and `ATTRIBUTION.md` are kept, with FigCraft's copyright and GUI Studio's additions.
- **Prototype.** The eframe prototype (`src/`, `tests/`) leaves the tree. Tag `eframe-prototype` keeps it for reference. Read it with `git show eframe-prototype:src/project.rs`.

## Consequences

- Positive:
  - Specs, decisions and code change together in one public repository and one history.
  - GUI Studio can restructure FigCraft code freely, because no merge base has to be protected.
- Cost/risk:
  - Each adopted upstream change is manual work. Cherry-picks conflict where GUI Studio has changed the same lines.
  - Security fixes in FigCraft are not received automatically. Check FigCraft releases for security fixes when planning each phase.
  - The FigCraft code is now public under GUI Studio's name, so the trademark rules in ADR-002 (Branding) apply to everything published.
- Revisit when: upstream changes that GUI Studio needs pile up faster than they can be ported, or GUI Studio restructures the FigCraft crates enough that cherry-picks stop applying.
