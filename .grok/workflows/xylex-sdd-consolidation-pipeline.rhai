// xylex-sdd-consolidation-pipeline
// Controlled consolidation: one architect, measured inventory, few executors,
// adversarial validator, integration. Not a parallel refactoring swarm.

let meta = #{
    name: "xylex-sdd-consolidation-pipeline",
    description: "Architect-owned migration contracts executed in dependency-aware lanes. Max 3–6 implementers. Lane B serial. Validators do not fix code. Never close debt by editing architecture to match implementation.",
    when_to_use: "Use for Weeping Angel consolidation waves. Prefer mode=contracts to emit/refresh docs/debt/migrations. mode=execute runs one wave or one debt_id in worktrees then validates. Do not use for new features.",
    phases: [
        #{ title: "Inventory", detail: "measure live metrics; do not edit product code" },
        #{ title: "Architect", detail: "lock queue from docs/debt/migrations; no independent architecture from executors" },
        #{ title: "Execute", detail: "one invariant per agent, worktree, allowlisted paths only" },
        #{ title: "Validate", detail: "adversarial disprove; no fixes" },
        #{ title: "Integrate", detail: "cherry-pick order + cargo xtask guard + inventory" },
    ],
};

let inventory_schema = #{
    "type": "object",
    "required": ["summary"],
    "properties": #{
        "summary": #{ "type": "string" },
        "root_test_binaries": #{ "type": "integer" },
        "public_symbols": #{ "type": "integer" },
        "duplicate_type_names": #{ "type": "integer" },
        "duplicate_helper_definitions": #{ "type": "integer" },
        "require_needles_fns": #{ "type": "integer" },
        "unwrap_plus_expect": #{ "type": "integer" },
        "notes": #{ "type": "string" },
    },
};

let architect_schema = #{
    "type": "object",
    "required": ["summary", "ready_count"],
    "properties": #{
        "summary": #{ "type": "string" },
        "ready_count": #{ "type": "integer" },
        "blocked": #{ "type": "array", "maxItems": 16, "items": #{ "type": "string" } },
        "queue": #{
            "type": "array",
            "maxItems": 16,
            "items": #{
                "type": "object",
                "properties": #{
                    "id": #{ "type": "string" },
                    "lane": #{ "type": "string" },
                    "wave": #{ "type": "integer" },
                    "contract_path": #{ "type": "string" },
                    "canonical_owner": #{ "type": "string" },
                    "invariant": #{ "type": "string" },
                    "ready": #{ "type": "boolean" },
                    "blocked_reason": #{ "type": "string" },
                },
            },
        },
        "notes": #{ "type": "string" },
    },
};

let exec_schema = #{
    "type": "object",
    "required": ["debt_id", "summary", "blocked"],
    "properties": #{
        "debt_id": #{ "type": "string" },
        "summary": #{ "type": "string" },
        "blocked": #{ "type": "boolean" },
        "block_reason": #{ "type": "string" },
        "files_changed": #{ "type": "array", "maxItems": 40, "items": #{ "type": "string" } },
        "commit_message": #{ "type": "string" },
        "worktree_path": #{ "type": "string" },
        "metric_after": #{ "type": "integer" },
        "notes": #{ "type": "string" },
    },
};

let verdict_schema = #{
    "type": "object",
    "required": ["debt_id", "complete", "reason", "evidence"],
    "properties": #{
        "debt_id": #{ "type": "string" },
        "complete": #{ "type": "boolean" },
        "reason": #{ "type": "string" },
        "evidence": #{ "type": "string" },
        "old_path_gone": #{ "type": "boolean" },
        "second_ssot": #{ "type": "boolean" },
        "tests_weakened": #{ "type": "boolean" },
        "public_symbols_increased": #{ "type": "boolean" },
        "new_alias": #{ "type": "boolean" },
        "new_root_test": #{ "type": "boolean" },
        "architecture_edited_to_fit": #{ "type": "boolean" },
        "notes": #{ "type": "string" },
    },
};

let integrate_schema = #{
    "type": "object",
    "required": ["summary", "guard_ok", "inventory_ok"],
    "properties": #{
        "summary": #{ "type": "string" },
        "guard_ok": #{ "type": "boolean" },
        "inventory_ok": #{ "type": "boolean" },
        "integrated": #{ "type": "array", "maxItems": 8, "items": #{ "type": "string" } },
        "rejected": #{ "type": "array", "maxItems": 8, "items": #{ "type": "string" } },
        "notes": #{ "type": "string" },
    },
};

fn as_str(v) {
    if v == () { return ""; }
    if type_of(v) == "string" { return v; }
    v.to_string()
}

fn as_bool(v, fallback) {
    if v == () { return fallback; }
    if type_of(v) == "bool" { return v; }
    let s = as_str(v);
    s.trim();
    if s == "true" || s == "1" { return true; }
    if s == "false" || s == "0" { return false; }
    fallback
}

fn as_int(v, fallback, min_v, max_v) {
    let n = fallback;
    if v != () {
        if type_of(v) == "i64" || type_of(v) == "int" || type_of(v) == "f64" { n = v; }
        else if type_of(v) == "string" {
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

fn agent_ok(r) {
    r != () && r.success && r.output != ()
}

fn j(v) {
    json_encode(v)
}

fn extract_kv(blob, key) {
    let s = as_str(blob);
    let n = as_str(key) + "=";
    if n == "=" { return ""; }
    let i = 0;
    let hit = -1;
    while i + n.len() <= s.len() {
        if s.sub_string(i, n.len()) == n {
            hit = i;
            i = s.len();
        } else {
            i += 1;
        }
    }
    if hit < 0 { return ""; }
    let k = hit + n.len();
    let out = "";
    while k < s.len() {
        let ch = s.sub_string(k, 1);
        if ch == " " || ch == "\n" || ch == "\r" || ch == "\t" || ch == "," {
            k = s.len();
        } else {
            out += ch;
            k += 1;
        }
    }
    out
}

let a = if args == () {
    #{}
} else if type_of(args) == "string" {
    #{ objective: args }
} else {
    args
};
let blob = as_str(a.objective) + " " + as_str(a.goal) + " " + as_str(a.paste) + " " + as_str(a.query);
let mode = as_str(a.mode);
if mode == "" { mode = extract_kv(blob, "mode"); }
if mode == "" { mode = "contracts"; }
if mode != "contracts" && mode != "execute" && mode != "full" {
    pause("verification", "args.mode must be contracts | execute | full");
}

let debt_id = as_str(a.debt_id);
if debt_id == "" { debt_id = extract_kv(blob, "debt_id"); }
let wave = as_int(a.wave, 1, 1, 4);
if a.wave == () {
    let wtok = extract_kv(blob, "wave");
    if wtok != "" { wave = as_int(wtok, wave, 1, 4); }
}
let max_exec = as_int(a.max_executors, 3, 1, 6);
if a.max_executors == () {
    let mtok = extract_kv(blob, "max_executors");
    if mtok != "" { max_exec = as_int(mtok, max_exec, 1, 6); }
}
let skip_in_flight = as_str(a.skip_ids);
if skip_in_flight == "" { skip_in_flight = extract_kv(blob, "skip_ids"); }

log("pipeline mode=" + mode + " wave=" + wave.to_string() + " max_executors=" + max_exec.to_string() + " debt_id=" + debt_id);

// ============================================================================
phase("Inventory");
let inv_p = "You are the INVENTORY agent. READ ONLY. Do not edit product code.\n";
inv_p += "Read docs/debt/current.md, docs/debt/consolidation-baseline.md, docs/debt/structural-duplication.toml, docs/debt/migrations/QUEUE.toml.\n";
inv_p += "If cargo xtask inventory --json exists, run it. Report live counts: root_test_binaries, public_symbols, duplicate_type_names, duplicate_helper_definitions, require_needles_fns, unwrap_plus_expect.\n";
inv_p += "Do not close debt. Do not change architecture.toml. summary must cite file paths you actually read.\n";
let inv_r = agent(inv_p, #{
    label: "inventory",
    capability_mode: "read-only",
    output_schema: inventory_schema,
});
if !agent_ok(inv_r) {
    complete(#{ summary: "Inventory agent failed.", success: false });
}
let inventory = inv_r.output;
log("inventory: " + as_str(inventory.summary));

// ============================================================================
phase("Architect");
let arch_p = "You are the ARCHITECT. You own canonical decisions. Executors must not invent architecture.\n";
arch_p += "Read and do not rewrite as a second program spec:\n";
arch_p += "- architecture/architecture.toml\n- architecture/invariants.toml\n";
arch_p += "- docs/debt/structural-duplication.toml\n- docs/debt/register.toml\n";
arch_p += "- docs/debt/migrations/QUEUE.toml and docs/debt/migrations/DUP-*.toml\n";
arch_p += "Requested wave=" + wave.to_string() + " debt_id=" + j(debt_id) + " skip_ids=" + j(skip_in_flight) + "\n";
arch_p += "In-flight means another LIVE workflow still running that package. Uncommitted COMPLETED work is not in-flight.\n";
arch_p += "DUP-002 status=verified and tests/support existing means C01 is done — do not block DUP-003 for that.\n";
arch_p += "Phase 1 is done if architecture/domain-ownership.toml exists and ADR 0050 is Accepted — do not block DUP-017 for dirty-tree alone.\n";
arch_p += "If args.debt_id is set, mark that id ready=true when its contract exists and depends_on rows are verified/canonicalized/removed or already implemented.\n";
arch_p += "Return queue entries from EXISTING contract files only. Do not mint new public types or new SSOTs in this phase.\n";
arch_p += "ready=true only if: contract file exists, depends_on not open, not in skip_ids, lane B not stacked, and (if debt_id set) id matches.\n";
arch_p += "If mode is contracts, you MAY add missing DUP-*.toml under docs/debt/migrations/ for QUEUE first_six only, copying fields from structural-duplication.toml. Do not edit architecture.toml to fit code.\n";
arch_p += "ready_count = number of ready=true items. summary must list ids.\n";
arch_p += "mode=" + mode + "\n";
arch_p += "inventory: " + j(inventory) + "\n";
let arch_cap = "read-only";
if mode == "contracts" || mode == "full" { arch_cap = "all"; }
let arch_r = agent(arch_p, #{
    label: "architect",
    capability_mode: arch_cap,
    output_schema: architect_schema,
});
if !agent_ok(arch_r) {
    complete(#{ summary: "Architect agent failed.", success: false, inventory: inventory });
}
let architect = arch_r.output;
let ready_n = 0;
if architect.ready_count != () { ready_n = architect.ready_count; }
log("architect ready_count=" + ready_n.to_string() + " " + as_str(architect.summary));

if mode == "contracts" {
    complete(#{
        summary: "Contracts-only: architect refreshed the queue. No executors launched.",
        success: true,
        mode: mode,
        inventory: inventory,
        architect: architect,
        ready_count: ready_n,
        max_executors: max_exec,
        note: "Launch mode=execute with one debt_id after in-flight Phase 1 / C01 settle. Lane B stays serial.",
    });
}

if ready_n < 1 {
    complete(#{
        summary: "No ready work packages. Inventory/architect only.",
        success: true,
        mode: mode,
        inventory: inventory,
        architect: architect,
    });
}

let queue = [];
if architect.queue != () && type_of(architect.queue) == "array" {
    queue = architect.queue;
}

let selected = [];
let qi = 0;
while qi < queue.len() {
    let item = queue[qi];
    if item != () {
        let ready = as_bool(item.ready, false);
        let id = as_str(item.id);
        let take = ready;
        if debt_id != "" && id != debt_id { take = false; }
        if take { selected.push(item); }
    }
    qi += 1;
}

if selected.len() == 0 {
    complete(#{
        summary: "Architect returned a queue but nothing matched debt_id/ready filter.",
        success: true,
        inventory: inventory,
        architect: architect,
    });
}

if selected.len() > max_exec {
    log("capping selected " + selected.len().to_string() + " → " + max_exec.to_string());
    let capped = [];
    let ci = 0;
    while ci < max_exec {
        capped.push(selected[ci]);
        ci += 1;
    }
    selected = capped;
}

let lane_b_count = 0;
let lj = 0;
while lj < selected.len() {
    if as_str(selected[lj].lane) == "B" { lane_b_count += 1; }
    lj += 1;
}
if lane_b_count > 1 {
    complete(#{
        summary: "HARD FAIL: Lane B (assurance semantics) must be serial. Architect selected multiple B items.",
        success: false,
        selected: selected,
    });
}

// ============================================================================
phase("Execute");
let jobs = [];
let ei = 0;
while ei < selected.len() {
    let pkg = selected[ei];
    let pid = as_str(pkg.id);
    let ep = "You are a NARROW EXECUTOR. You do not make architectural decisions.\n";
    ep += "ONE INVARIANT ONLY. ONE DEBT ID. ONE COMMIT in your isolated worktree if git is available.\n";
    ep += "Do not merge other agents. Do not edit architecture.toml / invariants.toml / domain-ownership.toml unless the contract allowlist names that exact file for metadata correction.\n";
    ep += "Never close debt by weakening a guard or rewriting architecture to describe your diff.\n";
    ep += "Contract:\n" + j(pkg) + "\n";
    ep += "Read docs/debt/migrations/" + pid + ".toml and obey allowed_modifications / forbidden / required_deletion / acceptance / do_not.\n";
    ep += "Monotonic: duplicate types, public symbols, root tests, duplicate helpers, production unwrap/expect must not increase.\n";
    ep += "Goal is net reduction. EXTEND/MIGRATE/CONSOLIDATE. Do not CREATE.\n";
    ep += "Return exec_schema. blocked=true if the contract cannot be met without violating forbidden.\n";
    jobs.push(#{
        prompt: ep,
        label: "exec-" + pid,
        capability_mode: "all",
        isolation_worktree: true,
        output_schema: exec_schema,
    });
    ei += 1;
}

let exec_results = [];
if selected.len() == 1 {
    let one = agent(jobs[0].prompt, #{
        label: jobs[0].label,
        capability_mode: jobs[0].capability_mode,
        isolation_worktree: true,
        output_schema: exec_schema,
    });
    exec_results.push(one);
} else {
    exec_results = parallel(jobs);
}

let executions = [];
let xi = 0;
while xi < exec_results.len() {
    let er = exec_results[xi];
    if agent_ok(er) {
        executions.push(er.output);
    } else {
        executions.push(#{
            debt_id: as_str(selected[xi].id),
            summary: "executor failed or empty",
            blocked: true,
            block_reason: "agent error",
        });
    }
    xi += 1;
}

// ============================================================================
phase("Validate");
let vjobs = [];
let vi = 0;
while vi < executions.len() {
    let ex = executions[vi];
    let did = as_str(ex.debt_id);
    if did == "" { did = as_str(selected[vi].id); }
    let vp = "You are the ADVERSARIAL VALIDATOR. You do not fix code.\n";
    vp += "Disprove completion. complete=true ONLY with concrete evidence you inspected.\n";
    vp += "Check: old path gone? second SSOT under another name? tests weakened? public symbol count up? unwrap/expect production up? compatibility alias? new root test binary? new parser/projection/helper? architecture.toml edited to fit the implementation?\n";
    vp += "Contract: " + j(selected[vi]) + "\n";
    vp += "Executor claim: " + j(ex) + "\n";
    vp += "Prestate inventory: " + j(inventory) + "\n";
    vp += "If executor blocked=true, complete=false.\n";
    vp += "Default complete=false when evidence is missing.\n";
    vjobs.push(#{
        prompt: vp,
        label: "validate-" + did,
        capability_mode: "read-only",
        output_schema: verdict_schema,
    });
    vi += 1;
}

let vres = [];
if vjobs.len() == 1 {
    vres.push(agent(vjobs[0].prompt, #{
        label: vjobs[0].label,
        capability_mode: "read-only",
        output_schema: verdict_schema,
    }));
} else {
    vres = parallel(vjobs);
}

let verdicts = [];
let confirmed = [];
let vj = 0;
while vj < vres.len() {
    let vr = vres[vj];
    if agent_ok(vr) && vr.output.complete == true && as_str(vr.output.evidence) != "" {
        if vr.output.architecture_edited_to_fit == true || vr.output.tests_weakened == true || vr.output.second_ssot == true {
            verdicts.push(vr.output);
        } else {
            verdicts.push(vr.output);
            confirmed.push(vr.output);
        }
    } else if agent_ok(vr) {
        verdicts.push(vr.output);
    } else {
        verdicts.push(#{
            debt_id: as_str(selected[vj].id),
            complete: false,
            reason: "validator failed closed",
            evidence: "",
        });
    }
    vj += 1;
}

log("validator confirmed " + confirmed.len().to_string() + "/" + verdicts.len().to_string());

if confirmed.len() == 0 {
    complete(#{
        summary: "No package survived adversarial validation. Nothing integrated.",
        success: false,
        inventory: inventory,
        architect: architect,
        executions: executions,
        verdicts: verdicts,
    });
}

// ============================================================================
phase("Integrate");
let ip = "You are the INTEGRATION agent. Cherry-pick only validator-complete worktrees in dependency order.\n";
ip += "Run cargo xtask guard and cargo xtask inventory after each semantic (Lane B) change. After mechanical Lane A, guard + inventory once is enough.\n";
ip += "Do not let executors merge each other. Do not edit architecture.toml to make a failing guard pass.\n";
ip += "Reject any package whose validator complete=false or architecture_edited_to_fit.\n";
ip += "Confirmed: " + j(confirmed) + "\n";
ip += "Executions: " + j(executions) + "\n";
ip += "Prestate inventory: " + j(inventory) + "\n";
ip += "Update docs/debt/structural-duplication.toml status from evidence only. verified only if close law holds.\n";
ip += "Return integrate_schema. guard_ok and inventory_ok must reflect commands you ran.\n";
let int_r = agent(ip, #{
    label: "integrate",
    capability_mode: "all",
    output_schema: integrate_schema,
});
if !agent_ok(int_r) {
    complete(#{
        summary: "Integration agent failed.",
        success: false,
        verdicts: verdicts,
        executions: executions,
    });
}
let integration = int_r.output;
if integration.guard_ok != true || integration.inventory_ok != true {
    complete(#{
        summary: "HARD FAIL: integration did not keep repository law green.",
        success: false,
        integration: integration,
        verdicts: verdicts,
    });
}

complete(#{
    summary: as_str(integration.summary),
    success: true,
    mode: mode,
    inventory: inventory,
    architect: architect,
    executions: executions,
    verdicts: verdicts,
    confirmed: confirmed,
    integration: integration,
});
