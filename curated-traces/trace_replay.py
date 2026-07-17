"""
Trace replay eval.

For each curated trace, extract the first user prompt and the expected
agent response shape, then run forge against the same prompt and compare.

Produces per-trace metrics:
- intent_match:    did forge understand the original user intent?
- tool_sequence_match: how close is forge's tool call sequence to the original?
- subagent_topology_match: did forge delegate to the same number/kind of sub-agents?
- output_similarity: linguistic similarity of forge output vs original output.
- response_latency_s: end-to-end latency.

These metrics feed back into the AI-DD eval suite via the standard
ExtendedMetricsEngine, with Ling2.6 Flash (OpenRouter) as the judge.
"""

from __future__ import annotations

import csv
import json
import sys
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Any

# `curated-traces` is the directory name (hyphenated, not importable as a
# Python package), so we add it to sys.path explicitly.
_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from traces_loader import (
    DEFAULT_TRACES_DIR,
    FULL_CSV,
    LITE_CSV,
    CuratedTrace,
    load_traces,
)


@dataclass
class TraceReplayResult:
    """Outcome of replaying one curated trace through forge."""

    trace_id: str
    agent: str
    title: str
    is_human_driven: bool
    is_long_horizon: bool
    original_messages: int
    original_tools: int
    original_subagents: int
    original_duration_s: int

    # Replay output
    forge_success: bool = False
    forge_output_length: int = 0
    forge_latency_s: float = 0.0
    forge_error: str = ""

    # Extracted comparison
    extracted_intent: str = ""
    extracted_first_user_msg: str = ""

    # Scores (filled by judge)
    intent_match: float = 0.0
    tool_sequence_match: float = 0.0
    subagent_topology_match: float = 0.0
    output_similarity: float = 0.0
    overall_replay_score: float = 0.0
    judge_feedback: str = ""
    judge_error: str = ""

    # Per-role assignment derived from trace
    inferred_role: str = ""

    # For serialization
    def to_dict(self) -> dict:
        return asdict(self)


# -----------------------------------------------------------------------------
# Raw-payload extraction
# -----------------------------------------------------------------------------


def _extract_messages_list(parsed_raw: Any) -> list[dict]:
    """Pull a `messages` list out of a parsed raw payload.

    Each curator encoded conversations slightly differently. We try the
    common shapes and return the first one that yields a non-empty list.
    """
    if isinstance(parsed_raw, dict):
        for key in ("messages", "conversation", "events", "turns"):
            v = parsed_raw.get(key)
            if isinstance(v, list) and v:
                return v
        # Some payloads nest under {data: {messages: [...]}}
        data = parsed_raw.get("data")
        if isinstance(data, dict):
            v = data.get("messages")
            if isinstance(v, list) and v:
                return v
    if isinstance(parsed_raw, list) and parsed_raw:
        return parsed_raw
    return []


def _extract_role_and_content(block: Any) -> tuple[str, str]:
    """Get (role, text) from a message block in any of the supported shapes."""
    if not isinstance(block, dict):
        return ("?", str(block)[:200])
    # Direct
    role = block.get("role") or block.get("type") or block.get("speaker") or "?"
    content = (
        block.get("content")
        or block.get("text")
        or block.get("message")
        or block.get("payload")
    )
    if isinstance(content, list):
        parts = []
        for p in content:
            if isinstance(p, dict):
                if p.get("type") == "text":
                    parts.append(p.get("text", ""))
                elif p.get("type") == "tool_use":
                    parts.append(f"[tool_use {p.get('name','?')}]")
                elif p.get("type") == "tool_result":
                    res = p.get("content", "")
                    if isinstance(res, list):
                        res = " ".join(str(r) for r in res)
                    parts.append(f"[tool_result: {str(res)[:200]}]")
                else:
                    parts.append(str(p)[:200])
            else:
                parts.append(str(p)[:200])
        content = "\n".join(parts)
    if isinstance(content, dict):
        # Nested Anthropic shape: {message: {text: {role, content}}}
        m = content.get("message") if isinstance(content, dict) else None
        if isinstance(m, dict):
            inner_text = m.get("text") or m.get("content")
            if isinstance(inner_text, dict):
                content = inner_text.get("content", "")
            elif isinstance(inner_text, list):
                content = " ".join(str(x) for x in inner_text)
            else:
                content = str(inner_text)
        else:
            content = json.dumps(content)[:500]
    if not isinstance(content, str):
        content = str(content)
    return (str(role), content)


def extract_first_user_msg(trace: CuratedTrace) -> str:
    """Return the first user-role message in the trace, or the title if absent."""
    parsed = trace.parse_raw()
    messages = _extract_messages_list(parsed)
    if not messages:
        return trace.user_intent or trace.title
    for m in messages:
        role, content = _extract_role_and_content(m)
        if role.lower() in ("user", "human"):
            return content.strip()
    # Fall back to title
    return trace.title


def extract_intent(trace: CuratedTrace) -> str:
    """Return the high-level intent. Prefer the explicit `user_intent` column
    if populated; otherwise the first user message."""
    if trace.user_intent and trace.user_intent.strip():
        return trace.user_intent.strip()
    return extract_first_user_msg(trace)


def infer_role(trace: CuratedTrace) -> str:
    """Heuristic role inference from tool count, subagent count, and message count."""
    if trace.subagent_count >= 1 and trace.tool_count >= 10:
        return "orchestrator"
    if trace.tool_count >= 30 and trace.message_count >= 50:
        return "implementer"
    if trace.tool_count >= 10 and trace.message_count < 30:
        return "debugger"
    if trace.message_count >= 30 and trace.tool_count < 10:
        return "reviewer"
    if trace.message_count < 10:
        return "researcher"
    return "implementer"


def extract_tool_sequence(trace: CuratedTrace) -> list[str]:
    """Best-effort extraction of tool names from the raw payload."""
    parsed = trace.parse_raw()
    messages = _extract_messages_list(parsed)
    tools: list[str] = []
    if not messages:
        return tools
    for m in messages:
        if not isinstance(m, dict):
            continue
        content = m.get("content")
        if not isinstance(content, list):
            continue
        for block in content:
            if not isinstance(block, dict):
                continue
            if block.get("type") == "tool_use":
                tools.append(str(block.get("name", "?")))
    return tools


# -----------------------------------------------------------------------------
# Forge invocation (mirrors eval_runner.run_forge_task)
# -----------------------------------------------------------------------------


def run_forge_task(
    instruction: str,
    timeout_s: int = 180,
    sandbox_name: str | None = None,
) -> tuple[bool, str, float, str]:
    """Run forge against a single instruction.

    Returns (success, output, latency_s, error).
    """
    import subprocess

    cmd = ["forge", "-p", instruction]
    if sandbox_name:
        cmd.extend(["--sandbox", sandbox_name])

    t0 = time.time()
    try:
        proc = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout_s,
            check=False,
        )
        latency = time.time() - t0
        out = (proc.stdout or "") + ("\n" + proc.stderr if proc.stderr else "")
        # Strip ANSI
        import re

        out = re.sub(r"\x1b\[[0-9;]*[a-zA-Z]", "", out)
        return (proc.returncode == 0, out, latency, "")
    except subprocess.TimeoutExpired:
        latency = time.time() - t0
        return (False, "", latency, f"timeout after {timeout_s}s")
    except Exception as e:  # noqa: BLE001
        return (False, "", time.time() - t0, str(e))


# -----------------------------------------------------------------------------
# Scoring via Ling2.6 Flash judge (OpenRouter)
# -----------------------------------------------------------------------------


def _judge_replay(
    original_intent: str,
    original_tools: list[str],
    original_subagents: int,
    forge_output: str,
    judge_model: str = "inclusionai/ling-2.6-flash",
    api_key: str | None = None,
    base_url: str = "https://openrouter.ai/api/v1",
) -> dict:
    """Call Ling2.6 Flash to score the replay.

    Returns dict with keys:
      intent_match, tool_sequence_match, subagent_topology_match,
      output_similarity, overall_score, feedback, error
    """
    import os
    import urllib.request

    key = api_key or os.environ.get("JUDGE_API_KEY") or os.environ.get(
        "OPENROUTER_API_KEY", ""
    )
    if not key:
        return {
            "intent_match": 0.0,
            "tool_sequence_match": 0.0,
            "subagent_topology_match": 0.0,
            "output_similarity": 0.0,
            "overall_score": 0.0,
            "feedback": "",
            "error": "no API key configured",
        }

    sys_prompt = (
        "You are an evaluator comparing a model's replay of an agent task "
        "against the original. Score on four dimensions 0.0-1.0. Reply with "
        "ONLY valid JSON.\n\n"
        "Dimensions:\n"
        "- intent_match: did the model understand the original intent? "
        "(0=no, 1=perfect)\n"
        "- tool_sequence_match: how close is the model's tool sequence to "
        f"the original? Original tools: {json.dumps(original_tools[:50])}\n"
        "- subagent_topology_match: did the model delegate to the right "
        f"number of sub-agents? Original: {original_subagents}\n"
        "- output_similarity: linguistic similarity of model output to "
        "the original (1=highly similar, 0=unrelated).\n"
    )

    user_msg = (
        f"ORIGINAL INTENT:\n{original_intent[:1500]}\n\n"
        f"MODEL REPLAY OUTPUT:\n{forge_output[:4000]}\n\n"
        "Return JSON exactly like:\n"
        '{"intent_match": 0.0, "tool_sequence_match": 0.0, '
        '"subagent_topology_match": 0.0, "output_similarity": 0.0, '
        '"feedback": "one sentence"}'
    )

    body = json.dumps(
        {
            "model": judge_model,
            "messages": [
                {"role": "system", "content": sys_prompt},
                {"role": "user", "content": user_msg},
            ],
            "temperature": 0.0,
        }
    ).encode()

    req = urllib.request.Request(
        f"{base_url}/chat/completions",
        data=body,
        headers={
            "Authorization": f"Bearer {key}",
            "Content-Type": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            data = json.loads(resp.read())
        content = data["choices"][0]["message"]["content"]
        # Strip code fence if present
        if content.startswith("```"):
            content = content.strip("`")
            if content.startswith("json"):
                content = content[4:]
            content = content.strip()
        parsed = json.loads(content)
        overall = (
            parsed["intent_match"] * 0.4
            + parsed["tool_sequence_match"] * 0.25
            + parsed["subagent_topology_match"] * 0.15
            + parsed["output_similarity"] * 0.2
        )
        return {
            "intent_match": float(parsed["intent_match"]),
            "tool_sequence_match": float(parsed["tool_sequence_match"]),
            "subagent_topology_match": float(parsed["subagent_topology_match"]),
            "output_similarity": float(parsed["output_similarity"]),
            "overall_score": round(overall, 4),
            "feedback": str(parsed.get("feedback", ""))[:500],
            "error": "",
        }
    except Exception as e:  # noqa: BLE001
        return {
            "intent_match": 0.0,
            "tool_sequence_match": 0.0,
            "subagent_topology_match": 0.0,
            "output_similarity": 0.0,
            "overall_score": 0.0,
            "feedback": "",
            "error": f"judge call failed: {e}",
        }


# -----------------------------------------------------------------------------
# Top-level replay runner
# -----------------------------------------------------------------------------


def replay_one(trace: CuratedTrace, judge_api_key: str | None = None) -> TraceReplayResult:
    """Replay one curated trace through forge and score the replay."""
    intent = extract_intent(trace)
    first_user_msg = extract_first_user_msg(trace)
    tool_seq = extract_tool_sequence(trace)
    role = infer_role(trace)

    result = TraceReplayResult(
        trace_id=trace.conversation_id,
        agent=trace.agent,
        title=trace.title[:200],
        is_human_driven=trace.is_human_driven,
        is_long_horizon=trace.is_long_horizon,
        original_messages=trace.message_count,
        original_tools=trace.tool_count,
        original_subagents=trace.subagent_count,
        original_duration_s=trace.duration_seconds,
        extracted_intent=intent[:500],
        extracted_first_user_msg=first_user_msg[:500],
        inferred_role=role,
    )

    # Replay through forge
    success, out, latency, err = run_forge_task(
        instruction=intent or first_user_msg,
        timeout_s=180,
        sandbox_name=f"trace-replay-{trace.conversation_id[:8]}",
    )
    result.forge_success = success
    result.forge_output_length = len(out)
    result.forge_latency_s = latency
    result.forge_error = err

    # Judge the replay
    judge = _judge_replay(
        original_intent=intent or first_user_msg,
        original_tools=tool_seq,
        original_subagents=trace.subagent_count,
        forge_output=out,
        api_key=judge_api_key,
    )
    result.intent_match = judge["intent_match"]
    result.tool_sequence_match = judge["tool_sequence_match"]
    result.subagent_topology_match = judge["subagent_topology_match"]
    result.output_similarity = judge["output_similarity"]
    result.overall_replay_score = judge["overall_score"]
    result.judge_feedback = judge["feedback"]
    result.judge_error = judge["error"]
    return result


def replay_all(
    csv_name: str = LITE_CSV,
    judge_api_key: str | None = None,
    traces_dir: Path | str = DEFAULT_TRACES_DIR,
) -> list[TraceReplayResult]:
    """Replay every trace in the given CSV."""
    traces = load_traces(csv_name, traces_dir=traces_dir)
    out: list[TraceReplayResult] = []
    for t in traces:
        out.append(replay_one(t, judge_api_key=judge_api_key))
    return out


def write_results(results: list[TraceReplayResult], out_path: Path | str) -> Path:
    """Write replay results to a JSON file."""
    out_path = Path(out_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "n": len(results),
        "avg_overall_replay_score": (
            round(sum(r.overall_replay_score for r in results) / len(results), 4)
            if results
            else 0.0
        ),
        "avg_intent_match": (
            round(sum(r.intent_match for r in results) / len(results), 4)
            if results
            else 0.0
        ),
        "avg_tool_sequence_match": (
            round(sum(r.tool_sequence_match for r in results) / len(results), 4)
            if results
            else 0.0
        ),
        "avg_subagent_topology_match": (
            round(sum(r.subagent_topology_match for r in results) / len(results), 4)
            if results
            else 0.0
        ),
        "avg_output_similarity": (
            round(sum(r.output_similarity for r in results) / len(results), 4)
            if results
            else 0.0
        ),
        "forge_success_rate": (
            round(sum(1 for r in results if r.forge_success) / len(results), 4)
            if results
            else 0.0
        ),
        "results": [r.to_dict() for r in results],
    }
    with out_path.open("w") as f:
        json.dump(payload, f, indent=2)
    return out_path


# -----------------------------------------------------------------------------
# CLI
# -----------------------------------------------------------------------------


def _cli() -> int:
    import argparse

    p = argparse.ArgumentParser(description="Replay curated traces through forge")
    p.add_argument(
        "--csv",
        default=LITE_CSV,
        choices=[FULL_CSV, LITE_CSV],
        help="Which curated CSV to replay (default: lite, for quick iteration)",
    )
    p.add_argument(
        "--output",
        default="benchmark/results/trace_replay.json",
        help="Where to write the results JSON",
    )
    p.add_argument(
        "--api-key",
        default=None,
        help="OpenRouter API key (default: $JUDGE_API_KEY or $OPENROUTER_API_KEY)",
    )
    args = p.parse_args()

    results = replay_all(csv_name=args.csv, judge_api_key=args.api_key)
    path = write_results(results, args.output)
    print(f"Wrote {len(results)} replay results to {path}")
    if results:
        s = results[0]
        print(
            f"  avg overall_replay_score={sum(r.overall_replay_score for r in results)/len(results):.3f}"
        )
        print(
            f"  avg intent_match={sum(r.intent_match for r in results)/len(results):.3f}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(_cli())