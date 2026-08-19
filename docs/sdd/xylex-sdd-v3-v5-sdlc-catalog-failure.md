# Cause: Prompt 05 SDLC catalog never landed (xylex-sdd-v3 / v5)

| Field | Value |
| --- | --- |
| Date | 2026-08-19 |
| Objective | `docs/prompts/canonical-assurance-v1/05-sdlc-catalog.md` |
| Base SHA | `e2def07ee4c3ec265a6b5fee116931f0b2c9ce94` |
| Product result | **No catalog, fixtures, or `[[test]]` rows on the primary tree** |
| Verdict | Four runs, three distinct protocol stops. None reached `TRANSITION_APPLIED`. |

This is not a missing-SDLC-control failure. The desired catalog was specified, and in one run it was even implemented in an isolated worktree. The orchestrator never applied that work because Init or prestate/integrity gates fail-closed first.

Supersedes the I3 harness hole in [xylex-sdd-v2-i3-prestate-failure.md](xylex-sdd-v2-i3-prestate-failure.md) for **this prompt**. v3/v5 *did* register `sdd_sdlc_catalog_*` in isolated worktrees (I2a). That is not why these four runs died.

---

## 1. What ran

| Display name | Workflow | Isolation | Result | Time | Checkpoint |
| --- | --- | --- | --- | --- | --- |
| `xylex-sdd-v3` | v3 | worktree | **HARD FAIL dirty_tree** | ~40s | Init |
| `xylex-sdd-v3-2` | v3 | worktree | **HARD FAIL I6** | ~56m | `PRESTATE_PROVEN` then implement |
| `xylex-sdd-v5` | v5 | worktree | **HARD FAIL dirty_tree** | ~2m | Init |
| `xylex-sdd-v5-2` | v5 | worktree | **HARD FAIL I4a** | ~49m | `HARNESSES_READY` |

Slash invocations (`/xylex-sdd-v3 @docs\…`, `/xylex-sdd-v5 @docs\…`) do **not** pass `args.snapshot_include`. That is why the first run of each catalog id died in Init and a second run with admitted ADR paths was required.

---

## 2. Cause chain (one sentence per layer)

1. **Primary tree was dirty.** Two untracked ADR drafts under `docs/adr/` are classified `potential`, not tool-state. Init fail-closes unless they are in `snapshot_include`.
2. **v3 TargetAuthor wrote an unsatisfiable AC-2.** The frozen suite reads its own source and asserts it does not contain `#[ignore`. That substring is in the assertion, so AC-2 cannot go GREEN.
3. **v3 I5 forbids editing the target after freeze.** The solver implemented the catalog (15/16) and could not fix AC-2. I6 abort. No patch on primary.
4. **v5 added I4a to refuse that freeze.** Correct intent. The scanner is coarser than the intent: any file that looks like a self-read plus **any** negated `.contains("literal")` is rejected.
5. **v5 TargetAuthor avoided the `#[ignore]` trap and still tripped I4a** on `!id.contains('_')` (AC-3 hyphen-id rule). Freeze refused. Implement never started.

Layer 1 is operator/invocation. Layers 2–3 are a real unsatisfiable contract. Layers 4–5 are a false-positive integrity scan on a satisfiable product assertion.

---

## 3. Cause A — dirty tree at Init (v3, v5)

Probe: `git status --porcelain` → `written_paths`. Rhai classifies (`xylex-sdd-v3.rhai` / `v5.rhai` `classify_dirty_path`):

| Class | Rule | These runs |
| --- | --- | --- |
| `admitted` | path in `args.snapshot_include` | empty on slash runs |
| `tool_state` | ephemeral (`.xbp`, `docs/sdd`, …) unless the objective names the path | `docs/sdd/sdd-sdd-*`, `docs/sdd/xylex-sdd-v2-i3-prestate-failure.md` |
| `semantic` | `src/`, `crates/`, `tests/`, manifests | none |
| `potential` | everything else | **`docs/adr/0003-sdlc-canonical-assurance-catalog-draft.md`**, **`docs/adr/0003-vulnerability-canonical-assurance-catalog-draft.md`** |

`docs/sdd/` is ignored. **`docs/adr/` is not.** Any unadmitted `semantic` or `potential` path →

```text
HARD FAIL: unadmitted semantic or potentially-semantic dirty paths.
Admit paths via args.snapshot_include to fold them into RepositorySnapshot.
```

Those ADR files are leftovers from earlier catalog authoring. They were not product edits for Prompt 05. The protocol still treats them as undeclared snapshot material.

**Reproduction:** slash `/xylex-sdd-v5 @docs/prompts/canonical-assurance-v1/05-sdlc-catalog.md` with those two files untracked and no `snapshot_include`.

**Unblock that already worked:** relaunch with

```json
{
  "objective": "@docs/prompts/canonical-assurance-v1/05-sdlc-catalog.md",
  "snapshot_include": [
    "docs/adr/0003-sdlc-canonical-assurance-catalog-draft.md",
    "docs/adr/0003-vulnerability-canonical-assurance-catalog-draft.md"
  ]
}
```

`xylex-sdd-v3-2` and `xylex-sdd-v5-2` both passed this gate. `admitted_paths` matched. `potential_dirty_paths` emptied.

---

## 4. Cause B — unsatisfiable frozen AC-2 (v3-2, I6)

Run dir: `docs/sdd/sdd-sdd-59e9991c-2b5e88f63c/`  
Run id: `sdd-59e9991c-2b5e88f63c`

### Prestate (correct)

| Evidence | Command | Result |
| --- | --- | --- |
| EV-001 | `cargo test --test sdd_sdlc_catalog_baseline -- --nocapture` | 15 passed — I3 GREEN |
| EV-002 | `cargo test --test sdd_sdlc_catalog_target -- --nocapture` | 6 passed / 10 failed — I4 RED |

Harness registration existed in the isolated worktrees (I2a). I3 from the v2 report did not recur.

### Implement (almost)

Solver wrote, in the implement worktree only:

- `catalog/canonical/v1/{manifest.toml,controls/sdlc.toml,evidence/sdlc.toml,tests/sdlc.toml}`
- seven `fixtures/assurance/canonical/v1/sdlc/*/evidence.json`
- `crates/weeping-angel-control-test/src/{lib.rs,population.rs}`
- `[[test]]` names `sdd_sdlc_catalog_baseline` and `sdd_sdlc_catalog_target`

Target post (EV-003): **15 passed / 1 failed.** The fail is `ac2_workspace_gates_and_sibling_suites_remain`.

Sibling `[[test]]` names **were** in implement `Cargo.toml`. AC-2’s other check reads the **target file’s own text**:

```rust
fn this_suite_source() -> String {
    fs::read_to_string(manifest_dir().join("tests/sdd/sdlc_catalog.target.rs")).unwrap()
}

assert!(
    !src.contains("#[ignore"),
    "AC-2: target suite must not #[ignore] acceptance tests"
);
```

The source contains the characters `#[ignore` because the assertion quotes them. `this_suite_source().contains("#[ignore")` is always true. AC-2 is not a product contract. It is a self-matching negated contains.

I5: target suite immutable after `PRESTATE_PROVEN`. Solver cannot delete the assertion. I6: target not GREEN after implement → no `implementation.patch` on primary.

Primary `Cargo.toml` still has no `sdd_sdlc_catalog_*` rows. Catalog TOML/fixtures stayed in

```text
%USERPROFILE%\.xylex-sdd\worktrees\59e9991c93075928\sdd-59e9991c-2b5e88f63c\implement\
```

---

## 5. Cause C — I4a false positive (v5-2)

Run dir: `docs/sdd/sdd-sdd-088983da-389f66a4fd/`  
Run id: `sdd-088983da-389f66a4fd`

v5 exists to stop Cause B **before** freeze (`xylex-sdd-v5.rhai` I4a, ~2975–3003). TargetAuthor is told not to write `!source.contains("#[ignore")`.

This TargetAuthor complied. AC-2 only inspects `Cargo.toml` sibling paths. EV-001: 19/19 GREEN. EV-002: 3 passed / 13 failed — missing SDLC documents, a real I4 RED.

Then the static scan:

```text
HARD FAIL I4a: target suite is not a satisfiable product contract
(self_matching_negated_contains:_ in tests/sdd/sdlc_catalog.target.rs).
RED from an intrinsically unsatisfiable test is not I4 semantic RED.
Do not freeze.
```

### What the scanner actually does

`target_source_integrity_defect` (`xylex-sdd-v5.rhai` ~739–755):

1. `looks_like_self_read` if the file has `read_to_string` / `include_str` / `fs::read` **and** (`file!()` or `target_test_path` or the **basename** `sdlc_catalog.target.rs`).
2. If so, the **first** negated `.contains("…")` / `.contains('…')` anywhere in the file is a defect, **whether or not the receiver is the suite source**.

This suite is a “self-read” because AC-1 does:

```rust
fs::read_to_string(manifest_dir().join("Cargo.toml"))
// and
manifest_dir().join("tests/sdd/sdlc_catalog.target.rs").is_file()
```

`read_to_string` + basename substring ⇒ `looks_like_self_read == true`.

The first negated string-literal contains is AC-3, not AC-2:

```rust
assert!(
    !id.contains('_'),
    "AC-3: catalog ids use hyphen segments ({id})"
);
```

`id` is a catalog control id. `'_'` is a product character-class rule. Implementing 26 hyphenated `control.source.*` ids makes this GREEN without touching the target file. That **is** I4a’s intended “plausible product-state mutation.”

The scanner does not check whether `lit` occurs in the suite source as a self-match, and does not check whether the receiver is the self-read buffer. Literal `_` produces the defect token `self_matching_negated_contains:_`.

Checkpoint stayed `HARNESSES_READY`. `target_integrity` false. Implement not started.

---

## 6. Gate table

| Gate | v3 | v3-2 | v5 | v5-2 |
| --- | --- | --- | --- | --- |
| Dirty tree admitted | **fail** | pass | **fail** | pass |
| `spec_frozen` (I1) | — | true | — | true |
| `ac_coverage` (I2) | — | true | — | true |
| harness executable (I2a) | — | true | — | true |
| `baseline_pre_green` (I3) | — | true | — | true |
| `target_pre_red` (I4) | — | true | — | true |
| `target_integrity` (I4a) | n/a | n/a | — | **false** |
| target frozen (I5) | — | true (bad suite) | — | **not frozen** |
| `target_post_green` (I6) | — | **false** (15/16) | — | — |
| transition / final | — | — | — | — |

v3-2 is the only run that mutated product files, and only inside the implement worktree.

---

## 7. What did *not* cause these four fails

- Prompt 05 content (controls, evidence types, fixtures) being unspecified.
- I3 unregistered `tests/sdd/*.rs` (v2 cause). v3/v5 HarnessResolver registered the names in worktrees; baseline actually executed.
- ISO sliver / IAM / GitHub collector regressions (sibling pins held where tests ran).
- Semantic dirty `src/` or `Cargo.toml` on the primary tree.
- Agent crashes (`agents_fail: 0` on all four).
- Budget exhaustion (v3-2 spent 29/128; v5-2 spent 23/128).

---

## 8. How to unblock (without weakening I3/I4/I6)

Keep I3–I6. Fix the two protocol defects and the one invocation hole.

1. **Init:** slash ` /xylex-sdd-v*` must admit known leftover `docs/adr/*draft*` **or** those drafts must be committed/removed before launch. `snapshot_include` on a second run already works.
2. **TargetAuthor (v3-class suites):** never assert absence of a literal by scanning the target file that contains the assertion. Pin “no `#[ignore]`” with `#[cfg]` / a count of ignored tests / a dedicated helper whose needle is assembled at runtime (`format!("{}{}", "#", "[ignore")`) if the check is required at all.
3. **I4a scanner:** only flag negated `.contains(lit)` when:
   - the receiver is the self-read buffer (`src`, `this_suite_source()`, `include_str!(file!())`, …), **and**
   - `lit` actually occurs in that file (or would, because the assertion quotes it).
   Do **not** treat `!id.contains('_')` on catalog ids as unsatisfiable.
4. **Do not re-run v3** against the frozen `sdd-59e9991c` target suite. I5 will reproduce I6.
5. **Do not re-run v5 unchanged.** TargetAuthor will likely emit `!id.contains('_')` again and I4a will abort the same way.

After (3) or an authoring rule that forbids negated `.contains("…")` entirely, Prompt 05 is an additive dual-suite: baseline GREEN on current spine, target RED on missing `control.source|cicd|release|supply-chain.*`, implement fills `catalog/canonical/v1` + seven fixtures + harness rows, I6 GREEN, apply patch to primary.
