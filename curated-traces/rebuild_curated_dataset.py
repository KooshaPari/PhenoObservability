#!/usr/bin/env python3
"""
Recreates the curated conversation trace dataset (Full + Lite CSVs)
from the actual source databases:
  - ForgeCode  : ~/forge/.forge.db           (conversations table)
  - Claude Code: ~/.claude/projects/**/*.jsonl
  - Codex      : ~/.codex/sessions/**/*.jsonl
  - Cursor     : ~/.cursor/sessions/**/*.jsonl  (if present)
  - Factory    : ~/.factory/sessions/**/*.jsonl (if present)

Outputs:
  curated-traces/Full.csv   - 100 traces per source, 50% human / 50% agent
  curated-traces/Lite.csv   -  2 traces per source, 50% human / 50% agent
  curated-traces/manifest.json - provenance metadata for each trace

Schema (per the original curate.py / Lite CSV):
  trace_id, origin, source, started_at, duration_s, turns,
  tools_used, file_edits, file_reads, subprocess_invocations, subagents_spawned,
  model, total_tokens_input, total_tokens_output, est_cost_usd,
  is_human_driven, parent_session_id, workspace_id, intent_len_chars,
  first_user_msg_len_chars, has_assistant_reflection, has_recovery,
  path_to_full

Each ROW corresponds to ONE trace (= one session / one conversation).
Files are NOT embedded; rather, pointer + summary statistics are kept
inline so the CSVs stay small enough to share.
"""
from __future__ import annotations
import csv
import glob
import json
import os
import random
import sqlite3
import sys
from collections import Counter
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Any, Iterable

ROOT = Path("/Users/kooshapari/CodeProjects/Phenotype/repos")
OUT_DIR = ROOT / "curated-traces"
OUT_DIR.mkdir(parents=True, exist_ok=True)
SEED = 20260628  # the date of the original task spec

random.seed(SEED)


# ----------------------------------------------------------------------
# 1.  Source extractors (one per origin)
# ----------------------------------------------------------------------


@dataclass
class TraceSummary:
    trace_id: str
    origin: str                         # forge | codex | claude | cursor | factory
    source: str                         # coarse sub-class (e.g. "Web", "CLI")
    started_at: str                     # ISO-8601
    duration_s: float                   # wall-clock span between first and last entry
    turns: int                          # user/assistant alternations
    tools_used: list[str]               # unique tool names used
    file_edits: int                     # write/edit/patch tool calls
    file_reads: int                     # read/grep tool calls
    subprocess_invocations: int         # bash/shell/exec tool calls
    subagents_spawned: int              # task/sub-agent delegations
    model: str                          # primary model (when discernible)
    total_tokens_in: int                # sum of input_tokens across the trace
    total_tokens_out: int               # sum of output_tokens across the trace
    est_cost_usd: float                 # tiny_llm-based estimate (see below)
    is_human_driven: bool               # True if origin = direct human (not sub-agent)
    parent_session_id: str | None       # set when origin = sub-agent
    workspace_id: str | None
    intent_len_chars: int               # length of first user message
    first_user_msg_len_chars: int       # alias for downstream tooling
    has_assistant_reflection: bool      # did the assistant write down a plan/summary?
    has_recovery: bool                  # did any turn exhibit error-recovery?
    path_to_full: str                   # where the RAW trace JSON lives on disk


# ----- Pricing estimate (tiny heuristic; rev later) ---------------------

_PRICE_PER_1K_IN = {
    # USD per 1k tokens; rough averages.  Will be replaced by `pricing_catalog.ts`.
    "gpt-4o": 0.005, "gpt-4o-mini": 0.00015,
    "claude-3-5-sonnet": 0.003, "claude-sonnet-4": 0.003,
    "claude-opus-4": 0.015,
    "deepseek-v3": 0.00027, "deepseek-v4-flash": 0.00014,
    "glm-4.6": 0.0006, "glm-5.2": 0.001, "MiniMax-M3": 0.0004,
    "ling-2.6-flash": 0.00007, "ling-2.6-pro": 0.00014,
}
_PRICE_PER_1K_OUT = {k: v * 4 for k, v in _PRICE_PER_1K_IN.items()}


def _estimate_cost(model: str, in_tok: int, out_tok: int) -> float:
    pi = _PRICE_PER_1K_IN.get(model, 0.001)
    po = _PRICE_PER_1K_OUT.get(model, 0.004)
    return round((in_tok / 1000.0) * pi + (out_tok / 1000.0) * po, 4)


# ----------------------------------------------------------------------
# ForgeCode adapter
# ----------------------------------------------------------------------


def extract_forge(limit: int | None = None) -> Iterable[TraceSummary]:
    db_path = Path("/Users/kooshapari/forge/.forge.db")
    if not db_path.exists():
        return
    # Read-only, fail-fast on lock contention so a hung forge subprocess doesn't
    # block us for minutes.
    con = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True, timeout=2)
    con.row_factory = sqlite3.Row
    cur = con.cursor()
    # Filter in SQL so multi-MB contexts never reach Python.
    q = """
        SELECT conversation_id, title, parent_id, workspace_id,
               created_at, updated_at, LENGTH(context) AS ctx_len
        FROM conversations
        WHERE context IS NOT NULL
          AND LENGTH(context) BETWEEN 500 AND 500000
        ORDER BY created_at DESC
    """
    if limit:
        q += f" LIMIT {int(limit)}"
    cur.execute(q)
    for meta in cur.fetchall():
        # Now fetch ONLY the context for this single row, only if we survived the size filter.
        cur.execute("SELECT context FROM conversations WHERE conversation_id = ?", (meta["conversation_id"],))
        ctx_row = cur.fetchone()
        if not ctx_row:
            continue
        try:
            ctx = json.loads(ctx_row["context"]) if ctx_row["context"] else {}
        except Exception:
            continue
        msgs = ctx.get("messages") or []
        if not msgs:
            continue
        # Treat THIS forge row as a trace in itself (already a session).
        # Sub-traces spawned via task tool are inside the same `context`,
        # so we count tool_use of name=='task' as `subagents_spawned`.
        tools_used: list[str] = []
        file_edits = file_reads = subs = sub_agents = 0
        in_tok = out_tok = 0
        model = "unknown"
        started_at = meta["created_at"]
        last_ts = meta["updated_at"]
        reflection = False
        recovery = False
        first_user_len = 0
        for m in msgs:
            t = m.get("message", {}) or {}
            c = t.get("content") or []
            if not isinstance(c, list):
                continue
            if t.get("model"):
                model = t["model"]
            usage = t.get("usage") or {}
            in_tok += usage.get("input_tokens", 0) or 0
            out_tok += usage.get("output_tokens", 0) or 0
            for blk in c:
                if not isinstance(blk, dict):
                    continue
                btype = blk.get("type")
                if btype == "tool_use":
                    tn = blk.get("name", "")
                    tools_used.append(tn)
                    if tn in ("write", "edit", "patch", "Write", "Edit"):
                        file_edits += 1
                    elif tn in ("read", "grep", "fs_search", "Read", "Search", "Grep", "sem_search", "Shell", "task"):
                        file_reads += 1
                    if tn in ("shell", "bash", "Shell", "execute_command"):
                        subs += 1
                    if tn == "task":
                        sub_agents += 1
                elif btype == "text":
                    txt = blk.get("text", "")
                    if any(k in txt.lower() for k in ("plan:", "summary:", "i will now", "todo:", "## plan")):
                        reflection = True
                elif btype == "tool_result":
                    tres = blk.get("content", "")
                    if isinstance(tres, list):
                        for r in tres:
                            if isinstance(r, dict) and "error" in (r.get("text", "").lower()):
                                recovery = True

        # First user message length
        for m in msgs:
            t = m.get("message", {}) or {}
            if t.get("role") == "user":
                c = t.get("content") or []
                if isinstance(c, list):
                    parts = [p.get("text", "") for p in c if isinstance(p, dict) and p.get("type") == "text"]
                    first_user_len = sum(len(p) for p in parts)
                break

        try:
            from datetime import datetime
            t1 = datetime.fromisoformat(started_at.replace("Z", "+00:00"))
            t2 = datetime.fromisoformat(last_ts.replace("Z", "+00:00"))
            duration_s = max(0.0, (t2 - t1).total_seconds())
        except Exception:
            duration_s = 0.0

        is_human = meta["parent_id"] is None
        yield TraceSummary(
            trace_id=f"forge::{meta['conversation_id']}",
            origin="forge",
            source="Web",
            started_at=started_at,
            duration_s=duration_s,
            turns=sum(1 for m in msgs if (m.get("message") or {}).get("role") in ("user", "assistant")),
            tools_used=sorted(set(tools_used)),
            file_edits=file_edits,
            file_reads=file_reads,
            subprocess_invocations=subs,
            subagents_spawned=sub_agents,
            model=model,
            total_tokens_in=in_tok,
            total_tokens_out=out_tok,
            est_cost_usd=_estimate_cost(model, in_tok, out_tok),
            is_human_driven=bool(is_human),
            parent_session_id=meta["parent_id"],
            workspace_id=meta["workspace_id"],
            intent_len_chars=first_user_len,
            first_user_msg_len_chars=first_user_len,
            has_assistant_reflection=reflection,
            has_recovery=recovery,
            path_to_full=f"~/forge/.forge.db#{meta['conversation_id']}",
        )
    con.close()


# ----------------------------------------------------------------------
# Claude Code adapter (parses JSONL files)
# ----------------------------------------------------------------------


def extract_claude(limit: int | None = None) -> Iterable[TraceSummary]:
    projects = Path("/Users/kooshapari/.claude/projects")
    if not projects.exists():
        return
    files = list(projects.glob("**/*.jsonl"))
    if limit:
        files = files[:limit]
    for fpath in files:
        try:
            size = fpath.stat().st_size
            lines = fpath.read_text(errors="replace").splitlines()
        except Exception:
            continue
        if size < 500:           # skip trivial stubs (matches original curate.py)
            continue
        tools_used: list[str] = []
        file_edits = file_reads = subs = sub_agents = 0
        in_tok = out_tok = 0
        model = "claude"
        started_at = ""
        last_ts = ""
        role = "human"
        first_user_len = 0
        reflection = recovery = False
        turns = 0
        for line in lines:
            try:
                d = json.loads(line)
            except Exception:
                continue
            t = d.get("type")
            ts = d.get("timestamp") or ""
            if ts:
                if not started_at:
                    started_at = ts
                last_ts = ts
            if t == "user":
                turns += 1
                if not first_user_len:
                    msg = d.get("message") or {}
                    c = msg.get("content") if isinstance(msg, dict) else d.get("content")
                    if isinstance(c, str):
                        first_user_len = len(c)
                    elif isinstance(c, list):
                        first_user_len = sum(len(b.get("text", "")) for b in c if isinstance(b, dict))
                role = "human"
            elif t == "assistant":
                turns += 1
                role = "assistant"
                msg = d.get("message") or {}
                if isinstance(msg, dict):
                    model = msg.get("model", model)
                    usage = msg.get("usage") or {}
                    in_tok += usage.get("input_tokens", 0) or 0
                    out_tok += usage.get("output_tokens", 0) or 0
                    for b in msg.get("content") or []:
                        if isinstance(b, dict) and b.get("type") == "tool_use":
                            tn = b.get("name", "")
                            tools_used.append(tn)
                            if tn in ("write", "edit", "patch", "Write", "Edit", "MultiPatch"):
                                file_edits += 1
                            elif tn in ("read", "Read", "fs_search", "shell", "grep", "Search", "Grep"):
                                file_reads += 1
                            if tn in ("shell", "bash"):
                                subs += 1
                            if tn in ("task", "Task"):
                                sub_agents += 1
        try:
            from datetime import datetime
            t1 = datetime.fromisoformat(started_at.replace("Z", "+00:00"))
            t2 = datetime.fromisoformat(last_ts.replace("Z", "+00:00"))
            duration_s = max(0.0, (t2 - t1).total_seconds())
        except Exception:
            duration_s = 0.0
        if first_user_len == 0:
            continue
        yield TraceSummary(
            trace_id=f"claude::{fpath.stem}",
            origin="claude",
            source="CLI",
            started_at=started_at or "1970-01-01T00:00:00Z",
            duration_s=duration_s,
            turns=turns,
            tools_used=sorted(set(tools_used)),
            file_edits=file_edits,
            file_reads=file_reads,
            subprocess_invocations=subs,
            subagents_spawned=sub_agents,
            model=model,
            total_tokens_in=in_tok,
            total_tokens_out=out_tok,
            est_cost_usd=_estimate_cost(model, in_tok, out_tok),
            is_human_driven=(role == "human"),
            parent_session_id=None,
            workspace_id=fpath.parent.name,
            intent_len_chars=first_user_len,
            first_user_msg_len_chars=first_user_len,
            has_assistant_reflection=reflection,
            has_recovery=recovery,
            path_to_full=f"~/.claude/projects/{fpath.parent.name}/{fpath.name}",
        )


# ----------------------------------------------------------------------
# Codex adapter
# ----------------------------------------------------------------------


def extract_codex(limit: int | None = None) -> Iterable[TraceSummary]:
    sessions = Path("/Users/kooshapari/.codex/sessions")
    if not sessions.exists():
        return
    files = list(sessions.glob("**/*.jsonl"))
    if limit:
        files = files[:limit]
    for fpath in files:
        try:
            lines = fpath.read_text(errors="replace").splitlines()
            if len(lines) < 2:
                continue
        except Exception:
            continue
        tools_used: list[str] = []
        file_edits = file_reads = subs = sub_agents = 0
        in_tok = out_tok = 0
        model = "codex"
        started_at = ""
        last_ts = ""
        first_user_len = 0
        turns = 0
        is_human = True           # default
        parent_tid = None         # set if this is a sub-agent rollout
        reflection = False
        recovery = False
        for line in lines:
            try:
                d = json.loads(line)
            except Exception:
                continue
            ts = d.get("timestamp") or d.get("ts") or ""
            if ts:
                if not started_at:
                    started_at = ts
                last_ts = ts
            kind = d.get("type")
            payload = d.get("payload") or {}
            # Codex schema (2026+):
            #  - type=="session_meta": payload = {session_id, parent_thread_id, originator, source: {subagent: ...}}
            #  - type=="event_msg"   : payload = {type:"user_message|assistant_message|..."}
            #  - type=="response_item": payload = {role, content: [{type, text|input_args|...}]}
            if kind == "session_meta":
                parent_tid = payload.get("parent_thread_id")
                # subagent spawns have nested source.subagent
                src = payload.get("source") or {}
                if isinstance(src, dict) and "subagent" in src:
                    is_human = False
                # originator may carry a model name
                originator = payload.get("originator") or ""
                if originator:
                    model = originator
            elif kind == "event_msg":
                ptype = payload.get("type", "")
                if ptype == "user_message":
                    turns += 1
                    msg = payload.get("message") or ""
                    if isinstance(msg, str) and not first_user_len:
                        first_user_len = len(msg)
                elif ptype == "agent_message":
                    turns += 1
                    msg = payload.get("message") or ""
                    if isinstance(msg, str):
                        # cheap recovery detection
                        if "error" in msg.lower() or "fail" in msg.lower():
                            recovery = True
                elif ptype == "token_count":
                    usage = payload.get("info") or {}
                    if isinstance(usage, dict):
                        in_tok += usage.get("total_token_usage", {}).get("input_tokens", 0) or 0
                        out_tok += usage.get("total_token_usage", {}).get("output_tokens", 0) or 0
            elif kind == "response_item":
                role = payload.get("role")
                content = payload.get("content") or []
                if isinstance(content, list):
                    for b in content:
                        if not isinstance(b, dict):
                            continue
                        btype = b.get("type")
                        if btype == "input_text":
                            txt = b.get("text") or ""
                            if role in ("user", "user_instructions") and not first_user_len:
                                first_user_len = len(txt)
                        elif btype == "output_text":
                            txt = b.get("text") or ""
                            if role == "assistant":
                                if any(k in txt.lower() for k in ("plan:", "summary:", "i will now", "## ")):
                                    reflection = True
                        elif btype == "function_call":
                            tn = b.get("name", "tool")
                            tools_used.append(tn)
                            if "patch" in tn.lower() or tn in ("write_file", "edit_file", "update_plan"):
                                file_edits += 1
                            elif tn in ("read_file", "shell_command", "grep_files"):
                                file_reads += 1
                            if tn == "shell_command":
                                subs += 1
        try:
            from datetime import datetime
            t1 = datetime.fromisoformat(started_at.replace("Z", "+00:00"))
            t2 = datetime.fromisoformat(last_ts.replace("Z", "+00:00"))
            duration_s = max(0.0, (t2 - t1).total_seconds())
        except Exception:
            duration_s = 0.0
        if first_user_len == 0:
            continue
        yield TraceSummary(
            trace_id=f"codex::{fpath.stem}",
            origin="codex",
            source="CLI",
            started_at=started_at,
            duration_s=duration_s,
            turns=turns,
            tools_used=sorted(set(tools_used)),
            file_edits=file_edits,
            file_reads=file_reads,
            subprocess_invocations=subs,
            subagents_spawned=sub_agents,
            model=model,
            total_tokens_in=in_tok,
            total_tokens_out=out_tok,
            est_cost_usd=_estimate_cost(model, in_tok, out_tok),
            is_human_driven=bool(is_human),
            parent_session_id=parent_tid,
            workspace_id=fpath.parent.name,
            intent_len_chars=first_user_len,
            first_user_msg_len_chars=first_user_len,
            has_assistant_reflection=reflection,
            has_recovery=recovery,
            path_to_full=str(fpath),
        )


# ----------------------------------------------------------------------
# 2. Bucket / pick / split helpers (mirror original curate.py logic)
# ----------------------------------------------------------------------


def classify_long_short(traces: list[TraceSummary]) -> tuple[list, list, list]:
    long_, short_, mid_ = [], [], []
    for t in traces:
        if t.duration_s >= 24 * 3600 or t.turns >= 200:
            long_.append(t)
        elif t.duration_s <= 600 and t.turns <= 15:
            short_.append(t)
        else:
            mid_.append(t)
    return long_, short_, mid_


def pick_from_pool(n: int, longs: list, shorts: list, mids: list) -> list[TraceSummary]:
    target = n
    pick_long = min(len(longs), n // 3)
    pick_short = min(len(shorts), n // 3)
    remaining = target - pick_long - pick_short
    pick_mid = min(len(mids), remaining)
    result = longs[:pick_long] + shorts[:pick_short] + mids[:pick_mid]
    random.shuffle(result)
    # backfill if still short
    if len(result) < n:
        pool = [t for t in longs[pick_long:] + shorts[pick_short:] + mids[pick_mid:]]
        random.shuffle(pool)
        result.extend(pool[:n - len(result)])
    return result[:n]


def curate_set(traces: list[TraceSummary], n: int, origin: str) -> list[TraceSummary]:
    humans = [t for t in traces if t.is_human_driven]
    agents = [t for t in traces if not t.is_human_driven]
    h_long, h_short, h_mid = classify_long_short(humans)
    a_long, a_short, a_mid = classify_long_short(agents)
    n_human = n // 2
    n_agent = n - n_human
    if not agents and humans:
        n_human = n
        n_agent = 0
    elif not humans and agents:
        n_human = 0
        n_agent = n
    picked_h = pick_from_pool(n_human, h_long, h_short, h_mid)
    picked_a = pick_from_pool(n_agent, a_long, a_short, a_mid)
    return picked_h + picked_a


# ----------------------------------------------------------------------
# 3. Main driver
# ----------------------------------------------------------------------

FULL_ROW_TARGET = 100
LITE_ROW_TARGET = 2


def write_csv(rows: list[TraceSummary], fname: str) -> None:
    fname = str(fname)
    with open(fname, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow([
            "trace_id", "origin", "source", "started_at", "duration_s", "turns",
            "tools_used", "file_edits", "file_reads", "subprocess_invocations",
            "subagents_spawned", "model", "total_tokens_in", "total_tokens_out",
            "est_cost_usd", "is_human_driven", "parent_session_id", "workspace_id",
            "intent_len_chars", "first_user_msg_len_chars",
            "has_assistant_reflection", "has_recovery", "path_to_full",
        ])
        for t in rows:
            w.writerow([
                t.trace_id, t.origin, t.source, t.started_at, round(t.duration_s, 1), t.turns,
                ";".join(t.tools_used), t.file_edits, t.file_reads,
                t.subprocess_invocations, t.subagents_spawned,
                t.model, t.total_tokens_in, t.total_tokens_out, t.est_cost_usd,
                int(t.is_human_driven), t.parent_session_id or "",
                t.workspace_id or "", t.intent_len_chars, t.first_user_msg_len_chars,
                int(t.has_assistant_reflection), int(t.has_recovery), t.path_to_full,
            ])


def main() -> int:
    print("Extracting ForgeCode traces  ...", flush=True)
    forge_traces = list(extract_forge())
    print(f"  forge: {len(forge_traces)} raw candidates")

    print("Extracting Claude Code traces ...", flush=True)
    claude_traces = list(extract_claude())
    print(f"  claude: {len(claude_traces)} raw candidates")

    print("Extracting Codex traces     ...", flush=True)
    codex_traces = list(extract_codex())
    print(f"  codex: {len(codex_traces)} raw candidates")

    full_rows: list[TraceSummary] = []
    lite_rows: list[TraceSummary] = []

    for origin, pool in (("forge", forge_traces), ("claude", claude_traces), ("codex", codex_traces)):
        if not pool:
            print(f"  SKIP {origin}: empty pool")
            continue
        f = curate_set(pool, FULL_ROW_TARGET, origin)
        l = curate_set(pool, LITE_ROW_TARGET, origin)
        full_rows.extend(f)
        lite_rows.extend(l)
        print(f"  {origin}: full={len(f)} lite={len(l)}")

    full_path = OUT_DIR / "Full.csv"
    lite_path = OUT_DIR / "Lite.csv"
    write_csv(full_rows, full_path)
    write_csv(lite_rows, lite_path)

    manifest = {
        "generated_at": "2026-07-04",
        "task_spec": {
            "full_target_per_origin": FULL_ROW_TARGET,
            "lite_target_per_origin": LITE_ROW_TARGET,
            "human_agent_split": 0.5,
            "long_horizon_definition": "duration_s >= 86400 OR turns >= 200",
            "short_definition": "duration_s <= 600 AND turns <= 15",
        },
        "counts": {
            "raw_forge": len(forge_traces),
            "raw_claude": len(claude_traces),
            "raw_codex": len(codex_traces),
            "full_total": len(full_rows),
            "lite_total": len(lite_rows),
        },
    }
    (OUT_DIR / "manifest.json").write_text(json.dumps(manifest, indent=2))

    print(f"\nWrote: {full_path} ({len(full_rows)} rows)")
    print(f"Wrote: {lite_path} ({len(lite_rows)} rows)")
    print(f"Wrote: {OUT_DIR/'manifest.json'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
