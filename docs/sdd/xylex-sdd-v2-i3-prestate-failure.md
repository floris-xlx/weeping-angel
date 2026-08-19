# Report: xylex-sdd-v2 HARD FAIL I3 (Prompt 05 + Prompt 06)

| Field | Value |
| --- | --- |
| Date | 2026-08-19 |
| Workflow | `xylex-sdd-v2` (`~/.grok/workflows/xylex-sdd-v2.rhai`) |
| Invariant | **I3** — baseline characterization MUST execute successfully on the pre-change tree |
| Verdict | Both runs stopped at `ProvePrestate`. Specs froze. No product code landed. |
| Base SHA | `e2def07ee4c3ec265a6b5fee116931f0b2c9ce94` |

This is not a product-logic failure. The orchestrator never reached Implement. The baseline *command* could not run because this repo does not auto-discover `tests/sdd/*.rs`.

---

## 1. What ran

| Display name | Prompt | Run id | Isolation | Requested mode | Recorded mode | Result | Time |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `xylex-sdd-v2` | `docs/prompts/canonical-assurance-v1/05-sdlc-catalog.md` | `sdd-de275e29-f5190941ef` | worktree | `strict` | `strict` | HARD FAIL I3 | ~38m |
| `xylex-sdd-v2-2` | `docs/prompts/canonical-assurance-v1/06-vulnerability-catalog.md` | `sdd-d16b8542-3e47532480` | worktree | `balanced` | `strict` | HARD FAIL I3 | ~31m |

Both used 12 of 128 agent budget and stopped at checkpoint `SPEC_FROZEN`.

`mode=balanced` on Prompt 06 never reached the workflow as `args.mode`. The slash line was treated as the objective string. The script defaults missing mode to `strict` (`xylex-sdd-v2.rhai` ~1150–1158). I3 is not relaxed in balanced mode, so this did not change the fail.

---

## 2. What I3 actually checks

From `xylex-sdd-v2.rhai`:

```text
I3  Baseline characterization MUST execute successfully on the pre-change tree.
```

`ProvePrestate` (after I1 spec freeze):

1. Create isolated worktrees from `base_revision` under `%USERPROFILE%\.xylex-sdd\worktrees\<objective_fp>\<run_id>\{baseline,target,implement}`.
2. **BaselineAuthor** writes only `run.suites.baseline.path` (the `.rs` file). It must not implement the feature and must not edit other paths.
3. **TargetAuthor** writes only the target `.rs` file.
4. An executor runs the frozen commands independently. Host classifies results (CI-001). Author `ok` is ignored.
5. `baseline_pre_green = green_valid_counts(...)` requires:
   - `exit_code == 0`
   - output is not “no tests found”
   - not (`tests_discovered > 0` and `tests_passed == 0`)
6. If that is false → `complete(HARD FAIL I3)`. Persist of worktree paths / evidence never happens.

---

## 3. Root cause (reproduced)

DiscoverSpec froze these commands:

| Run | Baseline command |
| --- | --- |
| Prompt 05 | `cargo test --test sdd_sdlc_catalog_baseline -- --nocapture` |
| Prompt 06 | `cargo test --test sdd_vulnerability_catalog_baseline -- --nocapture` |

Reproduced on this tree (same as the worktree `Cargo.toml` at `e2def07`):

```text
error: no test target named `sdd_sdlc_catalog_baseline` in default-run packages
error: no test target named `sdd_vulnerability_catalog_baseline` in default-run packages
```

Available SDD targets stop at Prompt 01–04 (`sdd_iam_catalog_*`, `sdd_canonical_assurance_catalog_*`, …). There is no Prompt 05/06 `[[test]]` stanza.

This repo registers integration tests explicitly:

```toml
[[test]]
name = "sdd_iam_catalog_baseline"
path = "tests/sdd/iam_catalog.baseline.rs"
```

`tests/sdd/*.rs` is **not** Cargo auto-discovery (`tests/*.rs` at the crate root is). A file under `tests/sdd/` is invisible to `cargo test --test <name>` until `Cargo.toml` lists it.

### Chicken-and-egg

| Constraint | Effect |
| --- | --- |
| BaselineAuthor allowed path | only `tests/sdd/{sdlc,vulnerability}_catalog.baseline.rs` |
| Executor | must not modify files; runs the command as written |
| Command | `--test sdd_*_catalog_baseline` |
| Tree at `e2def07` | no matching `[[test]]` |
| Result | cargo exits 1 **before any test function runs** |

I3 treats that as “baseline not GREEN.” I15 (infrastructure ≠ semantic RED) applies to *target* RED classification, not to baseline GREEN.

The authors did write the suites. They exist only in isolated worktrees:

```text
%USERPROFILE%\.xylex-sdd\worktrees\de275e291ea491b4\sdd-de275e29-f5190941ef\baseline\tests\sdd\sdlc_catalog.baseline.rs
%USERPROFILE%\.xylex-sdd\worktrees\d16b854296893b7b\sdd-d16b8542-3e47532480\baseline\tests\sdd\vulnerability_catalog.baseline.rs
```

Those worktrees’ `Cargo.toml` files still have the 15 pre-existing `[[test]]` entries. Neither author registered the new harness. The protocol forbade them from doing so.

---

## 4. What succeeded vs what did not

### Landed on primary (DiscoverSpec / I1)

| Artifact | Prompt 05 | Prompt 06 |
| --- | --- | --- |
| Spec | `docs/sdd/sdd-sdd-de275e29-f5190941ef/spec.md` | `docs/sdd/sdd-sdd-d16b8542-3e47532480/spec.md` |
| Acceptance | 16 ACs | 18 ACs |
| `spec_frozen` | true | true |
| Transition kind | **additive** | **replacement** |
| Catalog / crates / `Cargo.toml` | unchanged | unchanged |

Specs correctly characterize current `e2def07` behavior: `catalog/canonical/v1` lists only `fixture.example` + `identity`. They are usable as SSOT for a later slice.

### Did not land on primary

- `tests/sdd/sdlc_catalog.{baseline,target}.rs`
- `tests/sdd/vulnerability_catalog.{baseline,target}.rs`
- `[[test]]` registration
- `docs/sdd/<run>/evidence/*` (in-memory `evidence_push` never persisted; I3 aborts before `sdd-v2-persist-prestate`)
- `state.json` worktree paths stay `""` for the same reason
- Implementation, ISO remap, scanner-engine edits (never reached)

### Authored but trapped in worktrees

Prompt 05 baseline characterizes the current spine (fixture + IAM + ISO source sliver + frozen GitHub types). Absence of the new SDLC family is `#[ignore]` so the suite can stay GREEN after an additive landing. One ignored test even asserts that `Cargo.toml` does **not** list `sdd_sdlc_catalog_baseline` — which would fight registration if anyone added it.

Prompt 06 baseline characterizes absence of `control.vulnerability.*` / fixtures and leaves registration to implementer AC-1. Those tests are live `#[test]`s and would be GREEN *if* the harness were registered.

Target suites in the sibling worktrees assert the desired catalogs (AC coverage was accepted — I2 passed). They were never classified for I4 because I3 fails first.

---

## 5. Gate table (both runs)

| Gate | Expected at I3 | Observed |
| --- | --- | --- |
| `spec_frozen` (I1) | true | **true** |
| `ac_coverage` (I2) | true (else I2 fail) | reached I3 ⇒ I2 passed in-process |
| `baseline_pre_green` (I3) | true | **false** |
| `target_pre_red` (I4) | not evaluated | false |
| implement / post / transition / final | n/a | false |

Phase reached: **3/7 ProvePrestate**. Not a mid-implement regression.

---

## 6. Why this did not happen on Prompts 01–04

Earlier slices already have `[[test]]` names in root `Cargo.toml`. A new dual-suite on this repo is a **two-file** change (`.rs` + `Cargo.toml`). The V2 BaselineAuthor contract is **one-file**. That contract matches crates that auto-discover tests. It does not match Weeping Angel’s `tests/sdd/` harness.

Landed baselines (`iam_catalog.baseline.rs`, `canonical_assurance_catalog.baseline.rs`) are now superseded/`#[ignore]` on absence assertions. They were registered during those slices’ implement/transition, not invented out of thin air at I3 on a tree that lacked the target name.

---

## 7. What did *not* cause the fail

- Dirty primary tree (I14 worktrees were created; I14 abort would have been “could not create worktrees”).
- Empty authoring (`test_files` was non-empty; otherwise “authoring produced no test_files”).
- Incomplete AC map (that is I2, a different `complete()`).
- Target suite GREEN on current code (I4; never reached).
- Product compile errors inside the new `.rs` files (cargo never compiled them).
- Prompt 05 vs 06 content conflict (independent worktrees, identical harness hole).
- `mode=strict` vs `balanced` (I3 is identical).

---

## 8. How to unblock (without weakening I3)

I3 is correct: a characterization suite that cannot execute is not a proof.

Minimum change so the **same** workflow can pass I3:

1. **Register the baseline harness on the pre-change tree before / as part of authoring**, in the baseline worktree:
   ```toml
   [[test]]
   name = "sdd_sdlc_catalog_baseline"
   path = "tests/sdd/sdlc_catalog.baseline.rs"
   ```
   Same for `sdd_vulnerability_catalog_baseline`. Do **not** register the target suite yet if target tests would be collected by a workspace run you do not want GREEN.
2. **Widen BaselineAuthor allowed paths** in `xylex-sdd-v2.rhai` to include root `Cargo.toml` `[[test]]` addenda (or teach DiscoverSpec to emit a `rustc --test` / `cargo test --test` command that does not require a pre-existing name — not available here without registration).
3. **Do not characterize “suite is unregistered.”** That assertion is true on `e2def07` but makes the suite self-contradictory once registered. Registration is implementer AC-1 / target, not baseline.
4. Keep absence-of-catalog tests as live GREEN characterization for Prompt 06 (replacement). For Prompt 05 (additive), `#[ignore]` on absence is already the right CI-004 shape — but ignored-only suites can trip `tests_discovered > 0 && tests_passed == 0`. Keep at least one live passing test (the spine/IAM/ISO characterizations already do that).

Optional: persist `evidence/EV-001.json` and worktree paths even on I3 so the primary run dir is inspectable without hunting `%USERPROFILE%\.xylex-sdd\worktrees\`.

---

## 9. Resume vs new run

`state.json` checkpoint is `SPEC_FROZEN`. Worktrees still exist and already contain authored suites. A resume would re-enter `ProvePrestate`, recreate/reset worktrees (`git reset --hard` + `git clean -fd` if HEAD matches), and **wipe** the uncommitted `.rs` files unless they are committed in those worktrees.

Do not `resume` expecting the authored suites to survive a clean reset.

Next successful attempt should either:

- seed `Cargo.toml` `[[test]]` for the baseline name on a branch before launching V2, then point BaselineAuthor at the existing path; or
- patch the workflow to allow `Cargo.toml` harness edits in the baseline worktree, then launch a **new** run (new `run_id`).

Specs at `docs/sdd/sdd-sdd-de275e29-f5190941ef/spec.md` and `docs/sdd/sdd-sdd-d16b8542-3e47532480/spec.md` remain valid inputs.

---

## 10. Evidence index

| Claim | Where |
| --- | --- |
| I3 definition | `~/.grok/workflows/xylex-sdd-v2.rhai` lines 28, 2035–2045 |
| Author allowed path | same file ~1883, 1894 |
| Mode default `strict` | same file ~1150–1158 |
| Prompt 05 state | `docs/sdd/sdd-sdd-de275e29-f5190941ef/state.json` |
| Prompt 06 state | `docs/sdd/sdd-sdd-d16b8542-3e47532480/state.json` |
| No primary suites | `tests/sdd/` listing; `Cargo.toml` `[[test]]` through `sdd_canonical_assurance_catalog_*` |
| Authored baseline (05) | `%USERPROFILE%\.xylex-sdd\worktrees\de275e291ea491b4\sdd-de275e29-f5190941ef\baseline\tests\sdd\sdlc_catalog.baseline.rs` |
| Authored baseline (06) | `%USERPROFILE%\.xylex-sdd\worktrees\d16b854296893b7b\sdd-d16b8542-3e47532480\baseline\tests\sdd\vulnerability_catalog.baseline.rs` |
| Cargo error | `cargo test --test sdd_sdlc_catalog_baseline` / `sdd_vulnerability_catalog_baseline` → exit 1, “no test target named …” |
