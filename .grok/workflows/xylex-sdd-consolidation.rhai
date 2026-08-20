// xylex-sdd-consolidation
// Catalog id: xylex-sdd-consolidation. Host names must match `[a-z0-9]+(-[a-z0-9]+)*`.
//
// Generation-2 dual-suite: FORKED FROM workflows/spec-driven-development.rhai (V1).
// Not a bump of xylex-sdd-v2…v6. Those remain the expansion-optimized lineage (frozen).
// Keeps ~70–80% of V1: spec-before-code, baseline GREEN, target RED, no test-weakening,
// iterate to target GREEN, ADR/docs, supersede. Adds Classify + ArchPostcheck.
//
// Catalog trio:
//   xylex-sdd-route          = classify only, then invoke the matching specialist
//   xylex-sdd-feature        = CREATE (exceptional)
//   xylex-sdd-change         = EXTEND / REPLACE / REUSE
//   xylex-sdd-consolidation  = CONSOLIDATE / MIGRATE / RETIRE
//

let meta = #{
    name: "xylex-sdd-consolidation",
    description: "Generation-2 consolidation SDD forked from spec-driven-development V1. Net reduction: one debt, canonical owner, delete-or-migrate close law. Invoke as /workflow xylex-sdd-consolidation.",
    when_to_use: "Use when Classify disposition is CONSOLIDATE, MIGRATE, or RETIRE. Not for new features. One debt_id per run.",
    phases: [
        #{ title: "Scope", detail: "objective, package manager, test commands, report dir" },
        #{ title: "Classify", detail: "existence/change classification; CREATE is exceptional" },
        #{ title: "Spec", detail: "write/refine specification with canonical ownership; no product code" },
        #{ title: "BaselineGreen", detail: "characterization — prefer existing tests, dedicated binary last resort" },
        #{ title: "TargetRed", detail: "desired-behavior tests that FAIL on current implementation" },
        #{ title: "Gate", detail: "optional dry_run stop or await_user" },
        #{ title: "Implement", detail: "reuse/extend/migrate first; CREATE only if classified" },
        #{ title: "DocsAdr", detail: "amend or supersede governing ADR; new ADR last" },
        #{ title: "Iterate", detail: "repair until target green without weakening criteria" },
        #{ title: "ArchPostcheck", detail: "behavioral green + repo integrity + ownership" },
        #{ title: "Supersede", detail: "retire characterization scaffolding; additive is constrained" },
        #{ title: "Finalize", detail: "proof report; complete only if all three axes pass" },
    ],
};

let WORKFLOW_KIND = "consolidation";
let WORKFLOW_NAME = "xylex-sdd-consolidation";

let classify_schema = #{
    "type": "object",
    "required": [
        "capability_exists", "disposition", "canonical_owner", "bounded_context",
        "existing_owners", "competing_representations", "create_justification",
        "next_workflow", "summary"
    ],
    "properties": #{
        "capability_exists": #{ "type": "boolean" },
        "disposition": #{ "type": "string" },
        "canonical_owner": #{ "type": "string" },
        "bounded_context": #{ "type": "string" },
        "existing_owners": #{ "type": "array", "maxItems": 24, "items": #{ "type": "string" } },
        "reuse_apis": #{ "type": "array", "maxItems": 24, "items": #{ "type": "string" } },
        "competing_representations": #{ "type": "array", "maxItems": 24, "items": #{ "type": "string" } },
        "governing_adr": #{ "type": "string" },
        "adr_action": #{ "type": "string" },
        "new_public_surface": #{ "type": "boolean" },
        "new_persistence": #{ "type": "boolean" },
        "new_adr_required": #{ "type": "boolean" },
        "create_justification": #{ "type": "string" },
        "next_workflow": #{ "type": "string" },
        "debt_id": #{ "type": "string" },
        "close_law": #{ "type": "string" },
        "summary": #{ "type": "string" },
        "notes": #{ "type": "string" },
    },
};

let postcheck_schema = #{
    "type": "object",
    "required": [
        "added_public_type", "added_parser_projection_dto", "second_way",
        "new_cross_crate_dep", "new_adr_instead_of_amend", "inventory_increased",
        "architecture_toml_matches", "ownership_ok", "repo_integrity_ok", "summary"
    ],
    "properties": #{
        "added_public_type": #{ "type": "boolean" },
        "added_parser_projection_dto": #{ "type": "boolean" },
        "second_way": #{ "type": "boolean" },
        "new_cross_crate_dep": #{ "type": "boolean" },
        "new_adr_instead_of_amend": #{ "type": "boolean" },
        "inventory_increased": #{ "type": "boolean" },
        "architecture_toml_matches": #{ "type": "boolean" },
        "ownership_ok": #{ "type": "boolean" },
        "repo_integrity_ok": #{ "type": "boolean" },
        "suspicious": #{ "type": "boolean" },
        "summary": #{ "type": "string" },
        "notes": #{ "type": "string" },
    },
};



// --- schemas ---

let scope_schema = #{
    "type": "object",
    "required": [
        "root", "package_manager", "objective", "test_command",
        "report_dir", "notes"
    ],
    "properties": #{
        "root": #{ "type": "string" },
        "package_manager": #{ "type": "string" },
        "objective": #{ "type": "string" },
        "test_command": #{ "type": "string" },
        "baseline_test_path": #{ "type": "string" },
        "target_test_path": #{ "type": "string" },
        "report_dir": #{ "type": "string" },
        "live_seams": #{
            "type": "array", "maxItems": 20,
            "items": #{ "type": "string" },
        },
        "notes": #{ "type": "string" },
    },
};

let spec_schema = #{
    "type": "object",
    "required": [
        "title", "problem", "current_behavior", "desired_behavior",
        "acceptance_criteria", "out_of_scope", "risks", "adr_needed",
        "spec_path", "summary"
    ],
    "properties": #{
        "title": #{ "type": "string" },
        "problem": #{ "type": "string" },
        "current_behavior": #{ "type": "string" },
        "desired_behavior": #{ "type": "string" },
        "acceptance_criteria": #{
            "type": "array", "maxItems": 20,
            "items": #{ "type": "string" },
        },
        "out_of_scope": #{
            "type": "array", "maxItems": 16,
            "items": #{ "type": "string" },
        },
        "risks": #{
            "type": "array", "maxItems": 12,
            "items": #{ "type": "string" },
        },
        "adr_needed": #{ "type": "boolean" },
        "adr_draft_path": #{ "type": "string" },
        "spec_path": #{ "type": "string" },
        "summary": #{ "type": "string" },
    },
};

let suite_proof_schema = #{
    "type": "object",
    "required": [
        "suite_kind", "test_files", "command", "expected_outcome",
        "actual_outcome", "ok", "proof_excerpt", "notes"
    ],
    "properties": #{
        "suite_kind": #{ "type": "string" },
        "test_files": #{
            "type": "array", "maxItems": 24,
            "items": #{ "type": "string" },
        },
        "command": #{ "type": "string" },
        "expected_outcome": #{ "type": "string" },
        "actual_outcome": #{ "type": "string" },
        "ok": #{ "type": "boolean" },
        "exit_code": #{ "type": "integer" },
        "proof_excerpt": #{ "type": "string" },
        "notes": #{ "type": "string" },
    },
};

let implement_schema = #{
    "type": "object",
    "required": [
        "files_changed", "summary", "target_status", "baseline_status",
        "command_evidence"
    ],
    "properties": #{
        "files_changed": #{
            "type": "array", "maxItems": 40,
            "items": #{ "type": "string" },
        },
        "summary": #{ "type": "string" },
        "target_status": #{ "type": "string" },
        "baseline_status": #{ "type": "string" },
        "command_evidence": #{ "type": "string" },
        "blocked": #{ "type": "boolean" },
        "block_reason": #{ "type": "string" },
    },
};

let docs_schema = #{
    "type": "object",
    "required": ["docs_updated", "adr_paths", "summary"],
    "properties": #{
        "docs_updated": #{
            "type": "array", "maxItems": 30,
            "items": #{ "type": "string" },
        },
        "adr_paths": #{
            "type": "array", "maxItems": 12,
            "items": #{ "type": "string" },
        },
        "summary": #{ "type": "string" },
        "skipped_reason": #{ "type": "string" },
    },
};

let supersede_schema = #{
    "type": "object",
    "required": [
        "baseline_files", "action", "supersede_kind", "target_still_green",
        "baseline_not_green", "baseline_retired", "additive_baseline",
        "summary", "command_evidence"
    ],
    "properties": #{
        "baseline_files": #{
            "type": "array", "maxItems": 24,
            "items": #{ "type": "string" },
        },
        "action": #{ "type": "string" },
        // delete | move | skip | additive — structured gate, not free-text parsing
        "supersede_kind": #{ "type": "string" },
        "target_still_green": #{ "type": "boolean" },
        "baseline_not_green": #{ "type": "boolean" },
        "baseline_retired": #{ "type": "boolean" },
        "additive_baseline": #{ "type": "boolean" },
        "summary": #{ "type": "string" },
        "command_evidence": #{ "type": "string" },
    },
};

let finalize_schema = #{
    "type": "object",
    "required": ["report_path", "summary", "verify_ok", "protocol_table"],
    "properties": #{
        "report_path": #{ "type": "string" },
        "summary": #{ "type": "string" },
        "verify_ok": #{ "type": "boolean" },
        "protocol_table": #{ "type": "string" },
        "telemetry_path": #{ "type": "string" },
        "remaining_backlog": #{
            "type": "array", "maxItems": 16,
            "items": #{ "type": "string" },
        },
    },
};

// --- args ---

fn as_str(v) {
    if v == () { return ""; }
    if type_of(v) == "string" { return v; }
    v.to_string()
}

fn as_bool(v, default_val) {
    if v == () { return default_val; }
    if type_of(v) == "bool" { return v; }
    let s = as_str(v);
    s.trim();
    if s == "true" || s == "1" || s == "yes" { return true; }
    if s == "false" || s == "0" || s == "no" { return false; }
    default_val
}

fn as_int(v, default_val, min_v, max_v) {
    let n = default_val;
    if v != () {
        if type_of(v) == "i64" || type_of(v) == "int" || type_of(v) == "f64" {
            n = v;
        } else if type_of(v) == "string" {
            // JSON args often arrive as strings; parse common small ints.
            let s = v;
            s.trim();
            if s == "1" { n = 1; }
            else if s == "2" { n = 2; }
            else if s == "3" { n = 3; }
            else if s == "4" { n = 4; }
            else if s == "5" { n = 5; }
            else if s == "6" { n = 6; }
            else if s == "0" { n = 0; }
        }
    }
    if n < min_v { n = min_v; }
    if n > max_v { n = max_v; }
    n
}

let a = if args == () { #{} } else { args };
let objective = as_str(a.objective);
if objective == "" {
    objective = as_str(a.goal);
}
if objective == "" {
    objective = as_str(a.paste);
}
let root = as_str(a.root);
if root == "" { root = as_str(a.scope); }
let dry_run = as_bool(a.dry_run, false);
let want_await = as_bool(a.await_user, false);
let max_iters = as_int(a.max_iters, 3, 1, 6);
let report_dir = as_str(a.report_dir);
if report_dir == "" { report_dir = "docs/sdd"; }
let report_slug = as_str(a.report_slug);
let test_command_arg = as_str(a.test_command);
let baseline_glob = as_str(a.baseline_glob);
let target_glob = as_str(a.target_glob);
let telemetry_on = as_bool(a.telemetry, true);
let run_tag = as_str(a.run_id);
if run_tag == "" {
    run_tag = as_str(a.telemetry_run_id);
}
if run_tag == "" {
    // Deterministic tag from objective fingerprint (no wall clock in workflows).
    run_tag = "sdd-" + fingerprint(objective).sub_string(0, 12);
}

if objective == "" {
    pause("verification", "Pass args.objective (or goal/paste) plus args.debt_id for exactly one consolidation increment (e.g. C01 / DUP-002). This workflow deletes/migrates; it does not create features.");
}

let debt_id_arg = as_str(a.debt_id);
if debt_id_arg == "" {
    pause("verification", "Pass args.debt_id for exactly one increment (C01–C16 or one DUP-*). Parallel feature work in the same run is abort.");
}

// --- helpers for prompts + telemetry ---
// Rhai fn bodies cannot close over outer `let` bindings — pass a mutable map.

fn j(v) {
    json_encode(v)
}

fn telem_budget_snapshot() {
    let b = budget();
    if b == () {
        return #{ total: 0, spent: 0, remaining: 0 };
    }
    #{
        total: b.total,
        spent: b.spent,
        remaining: b.remaining,
    }
}

// Clone gate map so historical events do not share a live ref (all would show final state).
fn telem_gates_copy(g) {
    #{
        baseline_green: g.baseline_green,
        target_red: g.target_red,
        target_green: g.target_green,
        baseline_superseded: g.baseline_superseded,
        arch_postcheck: g.arch_postcheck,
        dry_run: g.dry_run,
        no_delta: g.no_delta,
    }
}

// Rhai maps are pass-by-value: helpers MUST return t; callers reassign `telem = …`.
// Also logs TELEMETRY|{json} for /workflows scrapers.
fn telem_emit(t, kind, phase_name, detail_map) {
    if t.on != true { return t; }
    t.seq += 1;
    let bud = telem_budget_snapshot();
    let gates_snap = telem_gates_copy(t.gates);
    let ev = #{
        seq: t.seq,
        run_id: t.run_id,
        workflow: as_str(t.workflow),
        kind: kind,
        phase: phase_name,
        budget: bud,
        agents_total: t.agents,
        agents_ok: t.agent_ok,
        agents_fail: t.agent_fail,
        tokens_used_sum: t.tokens,
        duration_ms_sum: t.duration_ms,
        gates: gates_snap,
        detail: detail_map,
    };
    t.events.push(ev);
    log("TELEMETRY|" + j(ev));
    // Durable last-event for monitors.
    write_scratch_file("sdd-telemetry-last.json", j(ev));
    t
}

fn telem_write_snapshot(t, reason) {
    let snap = telem_snapshot(t, reason);
    write_scratch_file("sdd-telemetry.json", j(snap));
    let jsonl = "";
    let ei = 0;
    while ei < t.events.len() {
        jsonl += j(t.events[ei]) + "\n";
        ei += 1;
    }
    write_scratch_file("sdd-telemetry.jsonl", jsonl);
    snap
}

// Host agent results may expose tokens under several field names.
fn telem_num(v) {
    if v == () { return 0; }
    if type_of(v) == "i64" || type_of(v) == "int" || type_of(v) == "f64" {
        return v;
    }
    0
}

fn telem_extract_tokens(agent_result) {
    if agent_result == () { return 0; }
    let toks = telem_num(agent_result.tokens_used);
    if toks > 0 { return toks; }
    toks = telem_num(agent_result.token_count);
    if toks > 0 { return toks; }
    toks = telem_num(agent_result.total_tokens);
    if toks > 0 { return toks; }
    if agent_result.usage != () {
        toks = telem_num(agent_result.usage.total_tokens);
        if toks > 0 { return toks; }
        toks = telem_num(agent_result.usage.tokens);
        if toks > 0 { return toks; }
        let pt = telem_num(agent_result.usage.prompt_tokens);
        let ct = telem_num(agent_result.usage.completion_tokens);
        if pt + ct > 0 { return pt + ct; }
    }
    if agent_result.metrics != () {
        toks = telem_num(agent_result.metrics.tokens_used);
        if toks > 0 { return toks; }
        toks = telem_num(agent_result.metrics.total_tokens);
        if toks > 0 { return toks; }
    }
    0
}

fn telem_extract_duration_ms(agent_result) {
    if agent_result == () { return 0; }
    let d = telem_num(agent_result.duration_ms);
    if d > 0 { return d; }
    d = telem_num(agent_result.elapsed_ms);
    if d > 0 { return d; }
    d = telem_num(agent_result.latency_ms);
    if d > 0 { return d; }
    if agent_result.metrics != () {
        d = telem_num(agent_result.metrics.duration_ms);
        if d > 0 { return d; }
    }
    0
}

fn telem_record_agent(t, phase_name, label, agent_result, extra) {
    if t.on != true { return t; }
    t.agents += 1;
    let ok = false;
    let d_ms = 0;
    let toks = 0;
    let aid = "";
    if agent_result != () {
        if agent_result.success == true { ok = true; }
        d_ms = telem_extract_duration_ms(agent_result);
        toks = telem_extract_tokens(agent_result);
        if agent_result.agent_id != () { aid = as_str(agent_result.agent_id); }
    }
    if ok {
        t.agent_ok += 1;
    } else {
        t.agent_fail += 1;
    }
    t.tokens += toks;
    t.duration_ms += d_ms;
    let detail = #{
        label: label,
        success: ok,
        duration_ms: d_ms,
        tokens_used: toks,
        agent_id: aid,
    };
    if extra != () {
        detail.extra = extra;
    }
    telem_emit(t, "agent", phase_name, detail)
}

fn telem_gate(t, name, passed, evidence) {
    if name == "baseline_green" { t.gates.baseline_green = passed; }
    if name == "target_red" { t.gates.target_red = passed; }
    if name == "target_green" { t.gates.target_green = passed; }
    if name == "baseline_superseded" { t.gates.baseline_superseded = passed; }
    if name == "arch_postcheck" { t.gates.arch_postcheck = passed; }
    telem_emit(t, "gate", name, #{
        passed: passed,
        evidence: as_str(evidence),
    })
}

fn telem_snapshot(t, reason) {
    #{
        run_id: t.run_id,
        workflow: as_str(t.workflow),
        reason: reason,
        objective_fp: t.objective_fp,
        dry_run: t.dry_run,
        no_delta: t.no_delta,
        max_iters: t.max_iters,
        agents_total: t.agents,
        agents_ok: t.agent_ok,
        agents_fail: t.agent_fail,
        tokens_used_sum: t.tokens,
        duration_ms_sum: t.duration_ms,
        budget: telem_budget_snapshot(),
        gates: telem_gates_copy(t.gates),
        event_count: t.events.len(),
        events: t.events,
    }
}

// --- args continued (no_delta) ---
// (telemetry_on / run_tag already parsed above)
let no_delta = as_bool(a.no_delta, false);
if !no_delta {
    no_delta = as_bool(a.allow_dual_green, false);
}

// In-memory telemetry state (passed into helpers — no outer-scope captures).
let telem = #{
    on: telemetry_on,
    run_id: run_tag,
    workflow: WORKFLOW_NAME,
    objective_fp: fingerprint(objective).sub_string(0, 16),
    dry_run: dry_run,
    no_delta: no_delta,
    max_iters: max_iters,
    events: [],
    seq: 0,
    agents: 0,
    agent_ok: 0,
    agent_fail: 0,
    tokens: 0,
    duration_ms: 0,
    gates: #{
        baseline_green: false,
        target_red: false,
        target_green: false,
        baseline_superseded: false,
        arch_postcheck: false,
        dry_run: dry_run,
        no_delta: no_delta,
    },
};

telem = telem_emit(telem, "run_start", "Scope", #{
    objective_fp: telem.objective_fp,
    root: root,
    dry_run: dry_run,
    await_user: want_await,
    max_iters: max_iters,
});

// ============================================================================
// Scope
// ============================================================================

phase("Scope");
log("SDD: scoping for dual-suite protocol");

let scope_prompt = "";
scope_prompt += "You are scoping a SPEC-DRIVEN DEVELOPMENT run.\n";
scope_prompt += "HARD PROTOCOL (do not invent a different order):\n";
scope_prompt += "1) Spec before product code\n";
scope_prompt += "2) Baseline characterization tests that PASS on CURRENT implementation\n";
scope_prompt += "3) Target tests that FAIL on CURRENT implementation\n";
scope_prompt += "4) Implement until targets PASS\n";
scope_prompt += "5) Baseline suite must FAIL (behavior changed) then be superseded/retired\n";
scope_prompt += "6) ADRs if architecture decision; update docs\n\n";
scope_prompt += "Objective (JSON): " + j(objective) + "\n";
scope_prompt += "Suggested root (JSON): " + j(root) + "\n";
scope_prompt += "Suggested test_command (JSON): " + j(test_command_arg) + "\n";
scope_prompt += "Suggested report_dir (JSON): " + j(report_dir) + "\n";
scope_prompt += "baseline_glob (JSON): " + j(baseline_glob) + "\n";
scope_prompt += "target_glob (JSON): " + j(target_glob) + "\n\n";
scope_prompt += "Use shell/list_dir/read_file/grep to detect package manager and real test commands.\n";
scope_prompt += "Propose baseline_test_path and target_test_path (files or dirs) under the package if not provided.\n";
scope_prompt += "Weeping Angel: dual-suite under xtask/tests/sdd_consolidation_<debt_id>_{baseline,target}.rs via cargo test -p xtask. FORBIDDEN: tests/sdd/, test/sdd/*.ts, new root [[test]], Guard 16.\n";
scope_prompt += "debt_id (exactly one): " + j(debt_id_arg) + "\n";
scope_prompt += "Return the structured scope schema only.\n";

let scope_r = agent(scope_prompt, #{
    label: "sdd-scope",
    capability_mode: "execute",
    output_schema: scope_schema,
});
telem = telem_record_agent(telem, "Scope", "sdd-scope", scope_r, ());

if !(scope_r != () && scope_r.success && scope_r.output != ()) {
    let snap = telem_write_snapshot(telem, "scope_failed");
    complete(#{
        summary: "Scope failed — cannot start SDD.",
        success: false,
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

let scope = scope_r.output;
let work_root = as_str(scope.root);
if work_root == "" { work_root = root; }
let test_cmd = as_str(scope.test_command);
if test_cmd == "" { test_cmd = test_command_arg; }
let out_report_dir = as_str(scope.report_dir);
if out_report_dir == "" { out_report_dir = report_dir; }
let baseline_path = as_str(scope.baseline_test_path);
let target_path = as_str(scope.target_test_path);

log("SDD scope root=" + work_root + " test=" + test_cmd);
telem = telem_emit(telem, "phase", "Scope", #{
    root: work_root,
    test_command: test_cmd,
    report_dir: out_report_dir,
    baseline_path: baseline_path,
    target_path: target_path,
});


// ============================================================================
// Classify — Existence / Change Classification (before Spec)
// ============================================================================

phase("Classify");
log("SDD G2: classify existing capability before writing a spec");
telem = telem_emit(telem, "phase_start", "Classify", #{ catalog: WORKFLOW_KIND });

let clf_prompt = "";
clf_prompt += "EXISTENCE / CHANGE CLASSIFICATION. Do not write a spec yet. Do not change product code.\n";
clf_prompt += "Objective: " + j(objective) + "\n";
clf_prompt += "Root hint: " + j(work_root) + "\n";
clf_prompt += "Search the repository for types, APIs, ADRs, specs, SSOTs, invariants that already own this behavior.\n";
clf_prompt += "DEFAULT ACTION ORDER (CREATE is exceptional):\n";
clf_prompt += "1. REUSE  2. EXTEND  3. MIGRATE  4. CONSOLIDATE  5. REPLACE  6. CREATE\n";
clf_prompt += "Disposition must be one of: CREATE, EXTEND, MIGRATE, CONSOLIDATE, REPLACE, RETIRE, REUSE.\n";
clf_prompt += "If capability does NOT exist: disposition=CREATE, next_workflow=xylex-sdd-feature.\n";
clf_prompt += "If YES same abstraction: EXTEND → xylex-sdd-change.\n";
clf_prompt += "If competing abstraction: CONSOLIDATE → xylex-sdd-consolidation.\n";
clf_prompt += "If obsolete implementation: REPLACE → xylex-sdd-change.\n";
clf_prompt += "If compatibility removal: RETIRE → xylex-sdd-consolidation.\n";
clf_prompt += "If migrate consumers to an existing owner: MIGRATE → xylex-sdd-consolidation.\n";
clf_prompt += "If already fully owned and only call-site reuse: REUSE → xylex-sdd-change (no new types).\n";
clf_prompt += "CREATE requires create_justification proving no existing owner can correctly own the behavior.\n";
clf_prompt += "adr_action: amend | supersede | new | none. Prefer amend; new ADR only if no governing ADR exists.\n";
clf_prompt += "Name canonical_owner and bounded_context. List competing_representations and reuse_apis.\n";
clf_prompt += "Return classify_schema only.\n";

let clf_r = agent(clf_prompt, #{
    label: "sdd-classify",
    capability_mode: "all",
    output_schema: classify_schema,
});
telem = telem_record_agent(telem, "Classify", "sdd-classify", clf_r, ());

if !(clf_r != () && clf_r.success && clf_r.output != ()) {
    let snap = telem_write_snapshot(telem, "classify_failed");
    complete(#{
        summary: "Classify phase failed — cannot start Spec.",
        success: false,
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

let classify = clf_r.output;
let disp = as_str(classify.disposition);
disp.trim();
let next_wf = as_str(classify.next_workflow);
next_wf.trim();
let owner = as_str(classify.canonical_owner);
owner.trim();

if disp == "CREATE" && as_str(classify.create_justification) == "" {
    let snap = telem_write_snapshot(telem, "create_unjustified");
    complete(#{
        summary: "HARD FAIL: CREATE is exceptional. Provide evidence that no existing owner can own this behavior.",
        success: false,
        classify: classify,
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

if WORKFLOW_KIND == "feature" {
    if disp != "CREATE" {
        let snap = telem_write_snapshot(telem, "wrong_catalog");
        complete(#{
            summary: "Capability already exists (disposition=" + disp + "). Use /workflow " + next_wf + " instead of xylex-sdd-feature.",
            success: false,
            next_workflow: next_wf,
            classify: classify,
            telemetry: snap,
            telemetry_run_id: run_tag,
        });
    }
}
if WORKFLOW_KIND == "change" {
    if disp != "EXTEND" && disp != "REPLACE" && disp != "REUSE" {
        let snap = telem_write_snapshot(telem, "wrong_catalog");
        complete(#{
            summary: "Disposition " + disp + " is not a change run. Use /workflow " + next_wf + ".",
            success: false,
            next_workflow: next_wf,
            classify: classify,
            telemetry: snap,
            telemetry_run_id: run_tag,
        });
    }
    if owner == "" {
        let snap = telem_write_snapshot(telem, "owner_missing");
        complete(#{
            summary: "HARD FAIL: change SDD requires canonical_owner before Spec.",
            success: false,
            classify: classify,
            telemetry: snap,
            telemetry_run_id: run_tag,
        });
    }
}
if WORKFLOW_KIND == "consolidation" {
    if disp != "CONSOLIDATE" && disp != "MIGRATE" && disp != "RETIRE" {
        let snap = telem_write_snapshot(telem, "wrong_catalog");
        complete(#{
            summary: "Disposition " + disp + " is not consolidation. Use /workflow " + next_wf + ".",
            success: false,
            next_workflow: next_wf,
            classify: classify,
            telemetry: snap,
            telemetry_run_id: run_tag,
        });
    }
    if owner == "" {
        let snap = telem_write_snapshot(telem, "owner_missing");
        complete(#{
            summary: "HARD FAIL: consolidation requires canonical_owner before Spec.",
            success: false,
            classify: classify,
            telemetry: snap,
            telemetry_run_id: run_tag,
        });
    }
}

if no_delta {
    let snap = telem_write_snapshot(telem, "no_delta_forbidden");
    complete(#{
        summary: "HARD FAIL: no_delta/allow_dual_green is forbidden in xylex-sdd-consolidation. Close law requires implement.",
        success: false,
        classify: classify,
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

telem = telem_emit(telem, "phase_end", "Classify", #{
    disposition: disp,
    next_workflow: next_wf,
    canonical_owner: owner,
    capability_exists: classify.capability_exists,
});


// ============================================================================
// Spec (no product code)
// ============================================================================

phase("Spec");
log("SDD: writing specification (no product implementation yet)");
telem = telem_emit(telem, "phase_start", "Spec", #{});

let spec_prompt = "";
spec_prompt += "Write a durable SPEC for this change BEFORE any product implementation.\n";
spec_prompt += "DO NOT change production source that implements the feature yet.\n";
spec_prompt += "Write a SMALL increment spec under " + out_report_dir + "/. Do NOT author a second giant architectural-cleanup program spec. Master laws/order stay in docs/specs/architectural-consolidation-program.md.\n\n";
spec_prompt += "Objective: " + j(objective) + "\n";
spec_prompt += "Locked debt_id (exactly one): " + j(debt_id_arg) + "\n";
spec_prompt += "Root: " + j(work_root) + "\n";
spec_prompt += "Scope notes: " + j(as_str(scope.notes)) + "\n\n";
spec_prompt += "Use read_file/grep. Discovery is incomplete until these are answered:\n";
spec_prompt += "Existing capability search; Canonical owner; Affected bounded context;\n";
spec_prompt += "Existing APIs/types to reuse; New public surface required?; New persistence required?; New ADR required?\n";
spec_prompt += "Classify result (binding): " + j(classify) + "\n";
spec_prompt += "One debt_id. Close law required. Enumerate consumers of every competing representation. No feature creation.\n";
spec_prompt += "- problem / user-visible goal\n";
spec_prompt += "- current_behavior (what code does today)\n";
spec_prompt += "- desired_behavior (what it should do after change)\n";
spec_prompt += "- acceptance_criteria as testable bullets\n";
spec_prompt += "- out_of_scope\n";
spec_prompt += "- risks\n";
spec_prompt += "adr_needed follows classify.adr_action: amend|supersede|new|none — not 'architecture involved → new ADR'.\n";
spec_prompt += "If adr_action=amend, edit the governing ADR path. If supersede, draft with explicit supersedes. If new, draft only because none exists.\n";
spec_prompt += "Return structured fields including spec_path to the written markdown.\n";

let spec_r = agent(spec_prompt, #{
    label: "sdd-spec",
    capability_mode: "all",
    output_schema: spec_schema,
});
telem = telem_record_agent(telem, "Spec", "sdd-spec", spec_r, ());

if !(spec_r != () && spec_r.success && spec_r.output != ()) {
    let snap = telem_write_snapshot(telem, "spec_failed");
    complete(#{
        summary: "Spec phase failed.",
        success: false,
        scope: scope,
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

let spec = spec_r.output;
log("SDD spec: " + as_str(spec.title) + " @ " + as_str(spec.spec_path));
telem = telem_emit(telem, "phase_end", "Spec", #{
    title: as_str(spec.title),
    spec_path: as_str(spec.spec_path),
    adr_needed: spec.adr_needed,
});

// ============================================================================
// Baseline GREEN
// ============================================================================

phase("BaselineGreen");
log("SDD: characterization tests must PASS on current implementation");
telem = telem_emit(telem, "phase_start", "BaselineGreen", #{});

let base_prompt = "";
base_prompt += "BASELINE suite — CHARACTERIZATION of CURRENT behavior.\n";
base_prompt += "HARD RULES:\n";
base_prompt += "- Write tests that encode what the code does TODAY (the found/current case).\n";
base_prompt += "- Run them against the CURRENT tree (no feature implementation yet).\n";
base_prompt += "- They MUST PASS (GREEN). If they fail, fix the tests or fix accidental breakage — do NOT implement the new feature yet.\n";
base_prompt += "Characterize CURRENT duplication (competing representations still exist). Prefer existing tests that already prove duplication; dedicated baseline only if needed as scaffolding.\n";
base_prompt += "- Characterization is scaffolding, not a permanent architectural pattern.\n\n";
base_prompt += "objective: " + j(objective) + "\n";
base_prompt += "root: " + j(work_root) + "\n";
base_prompt += "spec_path: " + j(as_str(spec.spec_path)) + "\n";
base_prompt += "current_behavior (from spec): " + j(as_str(spec.current_behavior)) + "\n";
base_prompt += "baseline_test_path: " + j(baseline_path) + "\n";
base_prompt += "test_command: " + j(test_cmd) + "\n";
base_prompt += "Use shell to run tests. Capture command + excerpt proving PASS.\n";
base_prompt += "suite_kind must be \"baseline\". expected_outcome must be \"pass\". actual_outcome must match. ok=true only if tests passed.\n";

let base_r = agent(base_prompt, #{
    label: "sdd-baseline-green",
    capability_mode: "all",
    output_schema: suite_proof_schema,
});
telem = telem_record_agent(telem, "BaselineGreen", "sdd-baseline-green", base_r, ());

if !(base_r != () && base_r.success && base_r.output != ()) {
    let snap = telem_write_snapshot(telem, "baseline_agent_failed");
    complete(#{
        summary: "Baseline suite phase failed (agent error).",
        success: false,
        spec: spec,
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

let baseline_proof = base_r.output;
if baseline_proof.ok != true {
    // One repair attempt: fix baseline tests only
    log("SDD: baseline not green — one repair on characterization tests only");
    let base_fix = "";
    base_fix += "Baseline suite did NOT pass. Repair BASELINE tests only so they PASS on CURRENT code.\n";
    base_fix += "Do NOT implement the new product feature. Do NOT write target suite yet.\n";
    base_fix += "Prior proof: " + j(baseline_proof) + "\n";
    base_fix += "spec current_behavior: " + j(as_str(spec.current_behavior)) + "\n";
    base_fix += "test_command: " + j(test_cmd) + "\n";
    base_fix += "Return suite_proof again with ok=true only if green.\n";
    let base_r2 = agent(base_fix, #{
        label: "sdd-baseline-repair",
        capability_mode: "all",
        output_schema: suite_proof_schema,
    });
    telem = telem_record_agent(telem, "BaselineGreen", "sdd-baseline-repair", base_r2, ());
    if base_r2 != () && base_r2.success && base_r2.output != () {
        baseline_proof = base_r2.output;
    }
}

if baseline_proof.ok != true {
    telem = telem_gate(telem, "baseline_green", false, as_str(baseline_proof.proof_excerpt));
    let snap = telem_write_snapshot(telem, "baseline_not_green");
    complete(#{
        summary: "HARD FAIL: baseline characterization suite is not green on current code. Aborting before target-red or implement.",
        success: false,
        baseline_proof: baseline_proof,
        spec_path: as_str(spec.spec_path),
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

log("SDD: baseline GREEN");
telem = telem_gate(telem, "baseline_green", true, as_str(baseline_proof.command));
telem = telem_emit(telem, "phase_end", "BaselineGreen", #{
    ok: true,
    command: as_str(baseline_proof.command),
    test_files: baseline_proof.test_files,
});

// ============================================================================
// Target RED
// ============================================================================

phase("TargetRed");
log("SDD: target suite must FAIL on current implementation");
telem = telem_emit(telem, "phase_start", "TargetRed", #{});

let red_prompt = "";
red_prompt += "TARGET suite — DESIRED behavior from the spec.\n";
red_prompt += "HARD RULES:\n";
red_prompt += "Target encodes the close law: old path gone or compatibility-only. Prefer structural enforcement (compiler, visibility, xtask) over new source-string tests.\n";
red_prompt += "- Encode acceptance criteria. Do not copy baseline files as a second permanent suite by default.\n";
red_prompt += "- Run them on CURRENT implementation WITHOUT implementing the feature.\n";
red_prompt += "- They MUST FAIL (RED). If they pass, they are not targeting new behavior — fix the tests.\n";
red_prompt += "- Do NOT change product code to make them pass yet.\n";
red_prompt += "- Prefer target_test_path separate from baseline.\n\n";
red_prompt += "objective: " + j(objective) + "\n";
red_prompt += "root: " + j(work_root) + "\n";
red_prompt += "desired_behavior: " + j(as_str(spec.desired_behavior)) + "\n";
red_prompt += "acceptance_criteria: " + j(spec.acceptance_criteria) + "\n";
red_prompt += "target_test_path: " + j(target_path) + "\n";
red_prompt += "baseline_test_path (do not modify to force red): " + j(baseline_path) + "\n";
red_prompt += "test_command: " + j(test_cmd) + "\n";
red_prompt += "suite_kind must be \"target\". expected_outcome must be \"fail\". ok=true only if tests FAILED as required.\n";

let red_r = agent(red_prompt, #{
    label: "sdd-target-red",
    capability_mode: "all",
    output_schema: suite_proof_schema,
});
telem = telem_record_agent(telem, "TargetRed", "sdd-target-red", red_r, ());

if !(red_r != () && red_r.success && red_r.output != ()) {
    let snap = telem_write_snapshot(telem, "target_red_agent_failed");
    complete(#{
        summary: "Target-red phase failed (agent error).",
        success: false,
        baseline_proof: baseline_proof,
        spec: spec,
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

let target_red_proof = red_r.output;
if target_red_proof.ok != true {
    log("SDD: target suite not red — one repair of target tests only");
    let red_fix = "";
    red_fix += "Target suite did not FAIL on current code. Adjust TARGET tests only so they fail for the right reason.\n";
    red_fix += "Do NOT implement the product feature. Do NOT weaken acceptance criteria to pass.\n";
    red_fix += "Prior proof: " + j(target_red_proof) + "\n";
    red_fix += "desired_behavior: " + j(as_str(spec.desired_behavior)) + "\n";
    red_fix += "suite_kind=target expected_outcome=fail ok=true only if fail.\n";
    let red_r2 = agent(red_fix, #{
        label: "sdd-target-red-repair",
        capability_mode: "all",
        output_schema: suite_proof_schema,
    });
    telem = telem_record_agent(telem, "TargetRed", "sdd-target-red-repair", red_r2, ());
    if red_r2 != () && red_r2.success && red_r2.output != () {
        target_red_proof = red_r2.output;
    }
}

// Target RED is required unless no_delta / allow_dual_green (characterization lock-in).
// Dual-green: agent reports actual_outcome pass/green while ok=false (expected fail).
let target_is_red = target_red_proof.ok == true;
let target_is_green_on_current = false;
if !target_is_red {
    let act = as_str(target_red_proof.actual_outcome);
    act.trim();
    if act == "pass" || act == "green" || act == "ok" {
        target_is_green_on_current = true;
    }
    // exit_code 0 with expected fail also means suite is green on current.
    if !target_is_green_on_current && target_red_proof.exit_code == 0 {
        target_is_green_on_current = true;
    }
}

if !target_is_red {
    if no_delta && target_is_green_on_current {
        log("SDD: no_delta — target GREEN on current is intentional (dual-green lock-in)");
        telem = telem_gate(telem, "target_red", false, "no_delta dual-green: target not required red");
        telem = telem_gate(telem, "target_green", true, as_str(target_red_proof.command));
        telem = telem_emit(telem, "phase_end", "TargetRed", #{
            ok: false,
            no_delta: true,
            dual_green: true,
            command: as_str(target_red_proof.command),
            test_files: target_red_proof.test_files,
        });
        // Skip implement: dual-green means desired == current.
        phase("Gate");
        telem = telem_emit(telem, "gate", "no_delta_stop", #{
            baseline_green: true,
            target_red: false,
            target_green: true,
            dual_green: true,
        });
        let snap = telem_write_snapshot(telem, "no_delta_dual_green_complete");
        let dry_path = write_scratch_file(
            "sdd-no-delta.md",
            "# SDD no_delta / dual-green\n\nBaseline GREEN + target GREEN on current code (characterization only). No implement.\n\n" + as_str(spec.summary) + "\n"
        );
        complete(#{
            summary: "no_delta: dual-green on current code — characterization only, no implement.",
            success: true,
            dry_run: dry_run,
            no_delta: true,
            dual_green: true,
            spec_path: as_str(spec.spec_path),
            baseline_proof: baseline_proof,
            target_red_proof: target_red_proof,
            scratch: dry_path,
            telemetry: snap,
            telemetry_run_id: run_tag,
        });
    }
    telem = telem_gate(telem, "target_red", false, as_str(target_red_proof.proof_excerpt));
    let snap = telem_write_snapshot(telem, "target_not_red");
    complete(#{
        summary: "HARD FAIL: target suite is not RED on current implementation. Aborting before implement. (Set no_delta/allow_dual_green for characterization-only dual-green.)",
        success: false,
        baseline_proof: baseline_proof,
        target_red_proof: target_red_proof,
        spec_path: as_str(spec.spec_path),
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

log("SDD: target RED (good)");
telem = telem_gate(telem, "target_red", true, as_str(target_red_proof.command));
telem = telem_emit(telem, "phase_end", "TargetRed", #{
    ok: true,
    command: as_str(target_red_proof.command),
    test_files: target_red_proof.test_files,
});

// ============================================================================
// Gate
// ============================================================================

phase("Gate");

if dry_run {
    telem = telem_emit(telem, "gate", "dry_run_stop", #{
        baseline_green: true,
        target_red: true,
    });
    let snap = telem_write_snapshot(telem, "dry_run_complete");
    let dry_path = write_scratch_file("sdd-dry-run.md", "# SDD dry_run\n\nSpec and dual-suite proof only.\n\n" + as_str(spec.summary) + "\n");
    complete(#{
        summary: "dry_run: stopped after baseline GREEN + target RED. No implement.",
        success: true,
        dry_run: true,
        spec_path: as_str(spec.spec_path),
        baseline_proof: baseline_proof,
        target_red_proof: target_red_proof,
        scratch: dry_path,
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

if want_await {
    telem = telem_emit(telem, "gate", "await_user", #{ message: "paused after dual-suite proof" });
    await_user("user", "Baseline is green and target is red. Resume to implement, document, and supersede.");
    telem = telem_emit(telem, "gate", "await_user_resumed", #{});
}

// ============================================================================
// Implement
// ============================================================================

phase("Implement");
log("SDD: implementing product change for target suite");
telem = telem_emit(telem, "phase_start", "Implement", #{});

let impl_prompt = "";
impl_prompt += "Do not satisfy this specification by adding a parallel implementation.\n";
impl_prompt += "The objective is net architectural reduction.\n";
impl_prompt += "Before changing code:\n";
impl_prompt += "- locate every existing representation of the concept;\n";
impl_prompt += "- identify the canonical owner;\n";
impl_prompt += "- enumerate consumers of every competing representation;\n";
impl_prompt += "- identify serialization/public API consequences.\n";
impl_prompt += "During implementation:\n";
impl_prompt += "- migrate consumers toward the canonical owner;\n";
impl_prompt += "- reuse existing types/functions wherever possible;\n";
impl_prompt += "- delete superseded implementations;\n";
impl_prompt += "- do not introduce compatibility aliases unless explicitly required;\n";
impl_prompt += "- do not introduce new public domain types unless the specification requires it.\n";
impl_prompt += "The change MUST reduce or preserve: public symbol count; duplicate type count; duplicated helper count; root integration-test binary count; persistence representations; projection paths.\n";
impl_prompt += "A green test suite is insufficient if competing semantic paths remain.\n";
impl_prompt += "debt_id: " + j(debt_id_arg) + "\n";
impl_prompt += "Ask: what already exists that should be extended or replaced? EXTEND / MIGRATE / CONSOLIDATE. Do not CREATE.\n";
impl_prompt += "TARGET tests must PASS, but that is not success by itself.\n";
impl_prompt += "Rules:\n";
impl_prompt += "- Do NOT delete target tests.\n";
impl_prompt += "- Do NOT change target tests to match a weaker implementation.\n";
impl_prompt += "- Do NOT introduce another representation, public API, parser, helper, or ADR sibling without classify permission.\n";
impl_prompt += "- Prefer small allowlisted diffs; no git add -A / node_modules.\n";
impl_prompt += "canonical_owner: " + j(as_str(classify.canonical_owner)) + "\n";
impl_prompt += "disposition: " + j(disp) + "\n\n";
impl_prompt += "objective: " + j(objective) + "\n";
impl_prompt += "spec_path: " + j(as_str(spec.spec_path)) + "\n";
impl_prompt += "desired_behavior: " + j(as_str(spec.desired_behavior)) + "\n";
impl_prompt += "acceptance_criteria: " + j(spec.acceptance_criteria) + "\n";
impl_prompt += "target suite files: " + j(target_red_proof.test_files) + "\n";
impl_prompt += "baseline suite files: " + j(baseline_proof.test_files) + "\n";
impl_prompt += "test_command: " + j(test_cmd) + "\n";
impl_prompt += "Run target suite; set target_status to pass|fail. Run baseline suite; set baseline_status to pass|fail.\n";
impl_prompt += "command_evidence must include real command output excerpts.\n";

let impl_r = agent(impl_prompt, #{
    label: "sdd-implement",
    capability_mode: "all",
    output_schema: implement_schema,
});
telem = telem_record_agent(telem, "Implement", "sdd-implement", impl_r, ());

let implement = #{};
if impl_r != () && impl_r.success && impl_r.output != () {
    implement = impl_r.output;
} else {
    implement = #{
        files_changed: [],
        summary: "implement agent failed",
        target_status: "unknown",
        baseline_status: "unknown",
        command_evidence: "",
        blocked: true,
        block_reason: "implement agent error",
    };
}
telem = telem_emit(telem, "phase_end", "Implement", #{
    target_status: as_str(implement.target_status),
    baseline_status: as_str(implement.baseline_status),
    files_changed: implement.files_changed,
});

// ============================================================================
// Docs + ADR
// ============================================================================

phase("DocsAdr");
log("SDD: ADRs and documentation");
telem = telem_emit(telem, "phase_start", "DocsAdr", #{ adr_needed: spec.adr_needed });

let docs_prompt = "";
docs_prompt += "Update documentation for the implemented change. Write ADRs if needed.\n";
docs_prompt += "adr_needed from spec: " + j(spec.adr_needed) + "\n";
docs_prompt += "Amend or supersede governing ADRs; do not add an ADR that restates a competing owner. Update debt inventory from evidence.\n";
docs_prompt += "classify.adr_action=" + j(as_str(classify.adr_action)) + " governing_adr=" + j(as_str(classify.governing_adr)) + "\n";
docs_prompt += "If adr_action=amend, update governing_adr. If supersede, new ADR must declare supersedes. If none, skip ADR.\n";
docs_prompt += "Update README/docs/contracts as appropriate — not fluff.\n";
docs_prompt += "objective: " + j(objective) + "\n";
docs_prompt += "implement summary: " + j(as_str(implement.summary)) + "\n";
docs_prompt += "files_changed: " + j(implement.files_changed) + "\n";
docs_prompt += "spec_path: " + j(as_str(spec.spec_path)) + "\n";
docs_prompt += "Return docs_updated and adr_paths arrays.\n";

let docs_r = agent(docs_prompt, #{
    label: "sdd-docs-adr",
    capability_mode: "all",
    output_schema: docs_schema,
});
telem = telem_record_agent(telem, "DocsAdr", "sdd-docs-adr", docs_r, ());

let docs = #{ docs_updated: [], adr_paths: [], summary: "docs skipped" };
if docs_r != () && docs_r.success && docs_r.output != () {
    docs = docs_r.output;
}
telem = telem_emit(telem, "phase_end", "DocsAdr", #{
    docs_updated: docs.docs_updated,
    adr_paths: docs.adr_paths,
});

// ============================================================================
// Iterate until target green
// ============================================================================

phase("Iterate");
log("SDD: iterate until target suite is green");
telem = telem_emit(telem, "phase_start", "Iterate", #{ max_iters: max_iters });

let target_status = as_str(implement.target_status);
let baseline_status = as_str(implement.baseline_status);
let iter = 0;
while iter < max_iters && target_status != "pass" {
    iter += 1;
    log("SDD repair iteration " + iter.to_string());
    telem = telem_emit(telem, "iterate", "Iterate", #{ iter: iter, target_status: target_status });
    let fix_prompt = "";
    fix_prompt += "Target suite is NOT green (status=" + target_status + ").\n";
    fix_prompt += "Repair product code so TARGET tests PASS without adding a parallel abstraction.\n";
    fix_prompt += "Do NOT gut acceptance criteria. Flaky target tests may be repaired only if they were not asserting the spec.\n";
    fix_prompt += "Keep baseline suite as historical characterization (may fail).\n";
    fix_prompt += "prior implement: " + j(implement) + "\n";
    fix_prompt += "target files: " + j(target_red_proof.test_files) + "\n";
    fix_prompt += "test_command: " + j(test_cmd) + "\n";
    fix_prompt += "Return implement schema with fresh evidence.\n";
    let fix_r = agent(fix_prompt, #{
        label: "sdd-iterate-" + iter.to_string(),
        capability_mode: "all",
        output_schema: implement_schema,
    });
    telem = telem_record_agent(telem, "Iterate", "sdd-iterate-" + iter.to_string(), fix_r, #{ iter: iter });
    if fix_r != () && fix_r.success && fix_r.output != () {
        implement = fix_r.output;
        target_status = as_str(implement.target_status);
        baseline_status = as_str(implement.baseline_status);
    } else {
        break;
    }
}

if target_status != "pass" {
    telem = telem_gate(telem, "target_green", false, "still " + target_status + " after " + iter.to_string() + " iters");
    let snap = telem_write_snapshot(telem, "target_not_green");
    complete(#{
        summary: "HARD FAIL: target suite still not green after " + max_iters.to_string() + " implement iterations.",
        success: false,
        implement: implement,
        baseline_proof: baseline_proof,
        target_red_proof: target_red_proof,
        docs: docs,
        spec_path: as_str(spec.spec_path),
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

telem = telem_gate(telem, "target_green", true, as_str(implement.command_evidence));
telem = telem_emit(telem, "phase_end", "Iterate", #{
    iters_used: iter,
    target_status: target_status,
    baseline_status: baseline_status,
});

// Prove baseline fails (supersession signal) when still present
if baseline_status == "pass" {
    log("SDD: baseline still passes after change — prove behavior shift or document no-break");
    let prove_prompt = "";
    prove_prompt += "After implementation, baseline characterization suite still PASSES.\n";
    prove_prompt += "Either (a) the change was purely additive and baseline still holds (document in notes),\n";
    prove_prompt += "or (b) re-run baseline suite and confirm pass is real.\n";
    prove_prompt += "Then set baseline_status accurately. Target must remain pass.\n";
    prove_prompt += "implement: " + j(implement) + "\n";
    prove_prompt += "test_command: " + j(test_cmd) + "\n";
    let prove_r = agent(prove_prompt, #{
        label: "sdd-baseline-post-check",
        capability_mode: "all",
        output_schema: implement_schema,
    });
    telem = telem_record_agent(telem, "Iterate", "sdd-baseline-post-check", prove_r, ());
    if prove_r != () && prove_r.success && prove_r.output != () {
        implement = prove_r.output;
        target_status = as_str(implement.target_status);
        baseline_status = as_str(implement.baseline_status);
    }
}

if target_status != "pass" {
    telem = telem_gate(telem, "target_green", false, "regressed after baseline post-check");
    let snap = telem_write_snapshot(telem, "target_regressed");
    complete(#{
        summary: "HARD FAIL: target suite regressed after baseline post-check.",
        success: false,
        implement: implement,
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}


// ============================================================================
// ArchPostcheck — behavioral green is not complete
// ============================================================================

phase("ArchPostcheck");
log("SDD G2: architectural postcheck after target GREEN");
telem = telem_emit(telem, "phase_start", "ArchPostcheck", #{});

let pc_prompt = "";
pc_prompt += "ARCHITECTURAL POSTCHECK. Target suite is GREEN. That is necessary, not sufficient.\n";
pc_prompt += "Complete = behavioral correctness + repository integrity + architectural ownership.\n";
pc_prompt += "Answer from the actual diff (files_changed) and repo search:\n";
pc_prompt += "- Did this change add a public type?\n";
pc_prompt += "- Did this change add a parser/projection/DTO?\n";
pc_prompt += "- Did this change create a second way of doing something?\n";
pc_prompt += "- Did it introduce a new cross-crate dependency?\n";
pc_prompt += "- Did it create a new ADR/spec instead of updating an owner?\n";
pc_prompt += "- Did duplicate-name/helper/API inventories increase?\n";
pc_prompt += "- Does architecture.toml (or equivalent) still describe reality?\n";
pc_prompt += "classify: " + j(classify) + "\n";
pc_prompt += "implement: " + j(implement) + "\n";
pc_prompt += "docs: " + j(docs) + "\n";
pc_prompt += "WORKFLOW_KIND=" + WORKFLOW_KIND + "\n";
if WORKFLOW_KIND == "consolidation" {
    pc_prompt += "HARD: second_way must be false. inventory_increased must be false. Close law: competing paths gone or compatibility-only.\n";
    pc_prompt += "Run `cargo xtask guard` and `cargo xtask inventory` (or inventory --check). Guards/specs must not be weakened. Update docs/debt from evidence.\n";
    pc_prompt += "debt_id: " + j(debt_id_arg) + "\n";
}
if WORKFLOW_KIND == "feature" {
    pc_prompt += "CREATE may add surface only if classify.create_justification covers each new public type/parser/persistence.\n";
    pc_prompt += "If a second_way appeared, this run must fail and be rerouted to consolidation.\n";
}
if WORKFLOW_KIND == "change" {
    pc_prompt += "Must keep canonical_owner. New public types without justification fail. Prefer reuse/extend.\n";
}
pc_prompt += "ownership_ok=true only if canonical owner still uniquely owns the behavior (or documented compatibility-only leftovers).\n";
pc_prompt += "repo_integrity_ok=true only if inventories did not silently grow and architecture metadata matches.\n";
pc_prompt += "suspicious=true if any growth looks unjustified — still set the booleans honestly.\n";
pc_prompt += "Return postcheck_schema. Do not weaken tests.\n";

let pc_r = agent(pc_prompt, #{
    label: "sdd-arch-postcheck",
    capability_mode: "all",
    output_schema: postcheck_schema,
});
telem = telem_record_agent(telem, "ArchPostcheck", "sdd-arch-postcheck", pc_r, ());

let postcheck = #{
    added_public_type: true,
    added_parser_projection_dto: true,
    second_way: true,
    new_cross_crate_dep: true,
    new_adr_instead_of_amend: true,
    inventory_increased: true,
    architecture_toml_matches: false,
    ownership_ok: false,
    repo_integrity_ok: false,
    suspicious: true,
    summary: "postcheck agent failed",
    notes: "",
};
if pc_r != () && pc_r.success && pc_r.output != () {
    postcheck = pc_r.output;
}

let arch_ok = postcheck.ownership_ok == true && postcheck.repo_integrity_ok == true && postcheck.architecture_toml_matches == true;
if WORKFLOW_KIND == "consolidation" {
    if postcheck.second_way == true || postcheck.inventory_increased == true {
        arch_ok = false;
    }
}
if postcheck.second_way == true && WORKFLOW_KIND != "consolidation" {
    arch_ok = false;
}
if classify.new_public_surface != true && postcheck.added_public_type == true {
    arch_ok = false;
}

if arch_ok != true {
    telem = telem_gate(telem, "arch_postcheck", false, as_str(postcheck.summary));
    let snap = telem_write_snapshot(telem, "arch_postcheck_failed");
    complete(#{
        summary: "HARD FAIL: target GREEN but architectural postcheck failed. " + as_str(postcheck.summary),
        success: false,
        postcheck: postcheck,
        classify: classify,
        implement: implement,
        telemetry: snap,
        telemetry_run_id: run_tag,
    });
}

telem = telem_gate(telem, "arch_postcheck", true, as_str(postcheck.summary));
telem = telem_emit(telem, "phase_end", "ArchPostcheck", #{
    ownership_ok: postcheck.ownership_ok,
    repo_integrity_ok: postcheck.repo_integrity_ok,
    suspicious: postcheck.suspicious,
});


// ============================================================================
// Supersede baseline
// ============================================================================

phase("Supersede");
log("SDD: supersede baseline suite (retire as historical)");
telem = telem_emit(telem, "phase_start", "Supersede", #{});

let super_prompt = "";
super_prompt += "SUPERSEDE the baseline characterization suite.\n";
super_prompt += "The target suite is the new source of truth and is GREEN.\n\n";
super_prompt += "Required outcomes:\n";
super_prompt += "1) Target suite still PASS (target_still_green=true).\n";
super_prompt += "2) Choose exactly one supersede_kind:\n";
super_prompt += "   - delete: remove baseline files (preferred when behavior changed)\n";
super_prompt += "   - move: move baseline to superseded/ or *.baseline.superseded.*\n";
super_prompt += "   - skip: FORBIDDEN in Weeping Angel (INV-NO-SUPERSEDED-BASELINES — do not #[ignore] baselines)\n";
super_prompt += "   - additive: FORBIDDEN unless leftover paths are listed as compatibility-only with sunset. Prefer delete of characterization scaffolding.\n";

super_prompt += "3) Set booleans (not free-text):\n";
super_prompt += "   - baseline_retired=true for delete|move|skip\n";
super_prompt += "   - additive_baseline=true ONLY for supersede_kind=additive\n";
super_prompt += "   - baseline_not_green=true if baseline was run and failed OR was deleted/moved/skipped\n";
super_prompt += "   - baseline_not_green=false only when additive_baseline=true and baseline still passes\n";
super_prompt += "4) command_evidence must include real test commands/excerpts proving target green and baseline status.\n\n";
super_prompt += "baseline files: " + j(baseline_proof.test_files) + "\n";
super_prompt += "target files: " + j(target_red_proof.test_files) + "\n";
super_prompt += "test_command: " + j(test_cmd) + "\n";
super_prompt += "action: short human description. supersede_kind: delete|move|skip|additive.\n";

let super_r = agent(super_prompt, #{
    label: "sdd-supersede",
    capability_mode: "all",
    output_schema: supersede_schema,
});
telem = telem_record_agent(telem, "Supersede", "sdd-supersede", super_r, ());

let supersede = #{
    baseline_files: [],
    action: "none",
    supersede_kind: "none",
    target_still_green: false,
    baseline_not_green: false,
    baseline_retired: false,
    additive_baseline: false,
    summary: "supersede agent failed",
    command_evidence: "",
};
if super_r != () && super_r.success && super_r.output != () {
    supersede = super_r.output;
}

if supersede.target_still_green != true {
    // one recovery
    log("SDD: supersede broke target green — recover target suite");
    let rec_prompt = "";
    rec_prompt += "Supersede step left target suite not green. Restore target PASS without reviving baseline as the gate.\n";
    rec_prompt += "supersede: " + j(supersede) + "\n";
    rec_prompt += "target files: " + j(target_red_proof.test_files) + "\n";
    rec_prompt += "test_command: " + j(test_cmd) + "\n";
    let rec_r = agent(rec_prompt, #{
        label: "sdd-supersede-recover",
        capability_mode: "all",
        output_schema: supersede_schema,
    });
    telem = telem_record_agent(telem, "Supersede", "sdd-supersede-recover", rec_r, ());
    if rec_r != () && rec_r.success && rec_r.output != () {
        supersede = rec_r.output;
    }
}

// Structured supersede success — do not parse free-text action strings.
let kind = as_str(supersede.supersede_kind);
kind.trim();
if kind == "" {
    kind = "none";
}
// Back-compat if older agent filled only action.
if kind == "none" {
    let act_fallback = as_str(supersede.action);
    act_fallback.trim();
    if act_fallback == "delete" || act_fallback == "move" || act_fallback == "skip" || act_fallback == "additive" {
        kind = act_fallback;
    }
}
let retired = supersede.baseline_retired == true;
let additive = supersede.additive_baseline == true;
if kind == "delete" || kind == "move" {
    retired = true;
}
if kind == "skip" || kind == "additive" {
    if WORKFLOW_KIND == "consolidation" {
        kind = "none";
        retired = false;
        additive = false;
    } else if kind == "skip" {
        retired = true;
    } else {
        additive = true;
    }
}
// Normalize booleans onto supersede for report consumers.
supersede.supersede_kind = kind;
supersede.baseline_retired = retired;
supersede.additive_baseline = additive;

let super_ok = false;
if supersede.target_still_green == true {
    if retired && supersede.baseline_not_green == true {
        super_ok = true;
    }
    if additive && supersede.baseline_not_green != true {
        // Additive: baseline still green is allowed; not retired as the gate.
        super_ok = true;
    }
}
telem = telem_gate(telem, "baseline_superseded", super_ok, kind + ": " + as_str(supersede.action));
telem = telem_emit(telem, "phase_end", "Supersede", #{
    action: as_str(supersede.action),
    supersede_kind: kind,
    target_still_green: supersede.target_still_green,
    baseline_not_green: supersede.baseline_not_green,
    baseline_retired: retired,
    additive_baseline: additive,
    super_ok: super_ok,
});

// ============================================================================
// Finalize report
// ============================================================================

phase("Finalize");
log("SDD: writing final report + telemetry");
telem = telem_emit(telem, "phase_start", "Finalize", #{});

let snap_pre = telem_snapshot(telem, "pre_finalize");
write_scratch_file("sdd-telemetry.json", j(snap_pre));

let fin_prompt = "";
fin_prompt += "Write a durable markdown report under " + out_report_dir + " for this SDD run.\n";
if report_slug != "" {
    fin_prompt += "Prefer filename stem: " + report_slug + "\n";
} else {
    fin_prompt += "Filename like sdd-<short-slug>.md\n";
}
fin_prompt += "Also write telemetry JSON next to the report as <stem>-telemetry.json (same dir).\n";
fin_prompt += "telemetry_run_id: " + run_tag + "\n";
fin_prompt += "Include protocol proof table:\n";
fin_prompt += "| Step | Expected | Actual |\n";
fin_prompt += "| Spec | written | path |\n";
fin_prompt += "| Baseline | PASS on old | proof |\n";
fin_prompt += "| Target pre | FAIL on old | proof |\n";
fin_prompt += "| Implement | target PASS | proof |\n";
fin_prompt += "| Baseline post | FAIL or retired | proof |\n";
fin_prompt += "| Supersede | target still PASS | proof |\n";
fin_prompt += "| Docs/ADR | updated | paths |\n\n";
fin_prompt += "Include a Telemetry section: agents_ok/fail, tokens_used_sum, duration_ms_sum, budget, gates, event_count.\n";
fin_prompt += "Inputs:\n";
fin_prompt += "spec: " + j(spec) + "\n";
fin_prompt += "baseline_proof: " + j(baseline_proof) + "\n";
fin_prompt += "target_red_proof: " + j(target_red_proof) + "\n";
fin_prompt += "implement: " + j(implement) + "\n";
fin_prompt += "docs: " + j(docs) + "\n";
fin_prompt += "supersede: " + j(supersede) + "\n";
fin_prompt += "telemetry_snapshot: " + j(snap_pre) + "\n";
fin_prompt += "verify_ok true only if target_still_green AND arch_postcheck passed AND (baseline_retired with baseline_not_green OR constrained additive).\n";
fin_prompt += "Complete = behavioral correctness + repository integrity + architectural ownership.\n";
fin_prompt += "classify: " + j(classify) + "\n";
fin_prompt += "postcheck: " + j(postcheck) + "\n";
fin_prompt += "supersede_kind from supersede phase: " + j(as_str(supersede.supersede_kind)) + "\n";
fin_prompt += "Return report_path, telemetry_path, protocol_table markdown, summary, verify_ok.\n";

let fin_r = agent(fin_prompt, #{
    label: "sdd-finalize",
    capability_mode: "all",
    output_schema: finalize_schema,
});
telem = telem_record_agent(telem, "Finalize", "sdd-finalize", fin_r, ());

let final = #{
    report_path: "",
    summary: "finalize failed",
    verify_ok: false,
    protocol_table: "",
    telemetry_path: "",
    remaining_backlog: [],
};
if fin_r != () && fin_r.success && fin_r.output != () {
    final = fin_r.output;
}

// Never trust agent verify_ok alone — require structured super_ok.
let verify_ok = false;
if final.verify_ok == true && super_ok == true && telem.gates.arch_postcheck == true {
    verify_ok = true;
}
telem = telem_emit(telem, "run_end", "Finalize", #{
    verify_ok: verify_ok,
    super_ok: super_ok,
    supersede_kind: as_str(supersede.supersede_kind),
    baseline_retired: supersede.baseline_retired,
    additive_baseline: supersede.additive_baseline,
    report_path: as_str(final.report_path),
    telemetry_path: as_str(final.telemetry_path),
});

let snap = telem_write_snapshot(telem, "complete");
let telem_path = "scratch/sdd-telemetry.json";

let scratch = write_scratch_file(
    "sdd-report.md",
    "# Spec-driven development result\n\n" + as_str(final.summary) + "\n\n" + as_str(final.protocol_table) + "\n\n## Telemetry\n\n```json\n" + j(snap) + "\n```\n"
);

complete(#{
    summary: as_str(final.summary),
    success: verify_ok,
    verify_ok: verify_ok,
    report_path: as_str(final.report_path),
    telemetry_path: as_str(final.telemetry_path),
    telemetry_scratch: telem_path,
    telemetry_run_id: run_tag,
    telemetry: snap,
    scratch: scratch,
    spec_path: as_str(spec.spec_path),
    classify: classify,
    postcheck: postcheck,
    baseline_proof: baseline_proof,
    target_red_proof: target_red_proof,
    implement: implement,
    docs: docs,
    supersede: supersede,
    protocol_table: as_str(final.protocol_table),
    remaining_backlog: final.remaining_backlog,
});
