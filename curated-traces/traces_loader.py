"""
Curated traces loader.

Loads the AI-DD curated trace dataset from
`/Users/kooshapari/CodeProjects/Phenotype/repos/curated-traces/`

The dataset contains raw conversation traces from three orchestrators:
- forge (KooshaPari's ForgeCode provider, ~/forge/.forge.db)
- claudecode (Claude Code, ~/.claude/projects)
- codex (Codex CLI, ~/.codex/sessions)

Two files:
- traces_full.csv: 200 traces (100 forge + 100 claudecode) with full raw payload
- traces_lite.csv: 4 curated traces for smoke tests

Each row's `raw` column holds the full conversation JSON-encoded.
"""

from __future__ import annotations

import csv
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterator

# Default location of the curated traces dataset
DEFAULT_TRACES_DIR = Path(
    "/Users/kooshapari/CodeProjects/Phenotype/repos/curated-traces"
)
FULL_CSV = "traces_full.csv"
LITE_CSV = "traces_lite.csv"
MANIFEST = "manifest.json"

# Bump CSV field size limit so the `raw` column parses without truncation.
csv.field_size_limit(sys.maxsize)


@dataclass
class CuratedTrace:
    """A single curated conversation trace."""

    agent: str
    conversation_id: str
    title: str
    started_at: str
    duration_seconds: int
    message_count: int
    tool_count: int
    subagent_count: int
    user_intent: str
    is_human_driven: bool
    is_long_horizon: bool
    model: str
    raw: str = ""

    # Parsed view of `raw` for downstream consumers. Populated on demand
    # via `parse_raw()` so the loader stays cheap.
    _raw_parsed: object = field(default=None, repr=False)

    def parse_raw(self) -> object:
        """Decode the `raw` column as JSON if possible.

        Returns the parsed object (dict / list / scalar) on success,
        or the original string if it isn't valid JSON.
        Cached so repeated calls are cheap.
        """
        if self._raw_parsed is None:
            try:
                self._raw_parsed = json.loads(self.raw)
            except (json.JSONDecodeError, TypeError):
                self._raw_parsed = self.raw
        return self._raw_parsed

    def extract_user_intent(self) -> str:
        """Extract user intent from the raw trace payload.

        Strategy per agent type:

        - **forge**: look for the first message where
          ``message.text.role`` (case-insensitive) is ``"User"``.  If no
          user message is stored (common – forge traces only capture the
          system prompt), fall back to ``self.title`` which holds the
          full task description.

        - **claudecode**: the ``raw`` column is ``{"path": "…", "format":
          "jsonl"}``.  Read the referenced JSONL file and return the
          ``content`` of the first entry where ``type == "user"`` and
          ``message.role == "user"`` with a string value.

        Returns the extracted text (or empty string if nothing found).
        """
        if self.user_intent:
            return self.user_intent

        parsed = self.parse_raw()
        if not isinstance(parsed, dict):
            return self.title or ""

        # ── Forge format ──────────────────────────────────────────
        if self.agent == "forge":
            messages = parsed.get("messages", [])
            for msg in messages:
                text = msg.get("message", {}).get("text", {})
                role = text.get("role", "")
                if role.lower() == "user":
                    content = text.get("content", "")
                    if content:
                        return content.strip()
            # forge traces store the user prompt in self.title
            return self.title or ""

        # ── ClaudeCode format ─────────────────────────────────────
        if self.agent == "claudecode":
            path = parsed.get("path", "")
            if not path:
                return self.title or ""
            try:
                with open(path, encoding="utf-8") as fh:
                    for line in fh:
                        line = line.strip()
                        if not line:
                            continue
                        entry = json.loads(line)
                        if (
                            entry.get("type") == "user"
                            and entry.get("message", {}).get("role") == "user"
                            and isinstance(entry["message"].get("content"), str)
                        ):
                            return entry["message"]["content"].strip()
            except (FileNotFoundError, json.JSONDecodeError, IOError):
                pass
            return self.title or ""

        # ── Unknown / other agent formats ─────────────────────────
        return self.title or ""

    def to_dict(self) -> dict:
        """Return a flat dict suitable for writing back to CSV."""
        return {
            "agent": self.agent,
            "conversation_id": self.conversation_id,
            "title": self.title,
            "started_at": self.started_at,
            "duration_seconds": self.duration_seconds,
            "message_count": self.message_count,
            "tool_count": self.tool_count,
            "subagent_count": self.subagent_count,
            "user_intent": self.user_intent,
            "is_human_driven": str(self.is_human_driven),
            "is_long_horizon": str(self.is_long_horizon),
            "model": self.model,
            "raw": self.raw,
        }


def _coerce_int(v: str) -> int:
    try:
        return int(v) if v else 0
    except (ValueError, TypeError):
        return 0


def _coerce_bool(v: str) -> bool:
    if isinstance(v, bool):
        return v
    return str(v).strip().lower() in ("true", "1", "yes", "y")


def load_traces(
    csv_name: str = FULL_CSV,
    traces_dir: Path | str = DEFAULT_TRACES_DIR,
    parse_raw: bool = False,
) -> list[CuratedTrace]:
    """Load all rows from a curated traces CSV.

    Args:
        csv_name: File name within `traces_dir` (default traces_full.csv).
        traces_dir: Directory containing the CSV (default canonical location).
        parse_raw: If True, eagerly parse each row's `raw` JSON. Default
            False keeps the loader O(N) in row count and cheap to run.

    Returns:
        List of CuratedTrace objects.
    """
    path = Path(traces_dir) / csv_name
    if not path.exists():
        raise FileNotFoundError(f"Curated traces file not found: {path}")

    traces: list[CuratedTrace] = []
    with path.open(newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            t = CuratedTrace(
                agent=row.get("agent", ""),
                conversation_id=row.get("conversation_id", ""),
                title=row.get("title", ""),
                started_at=row.get("started_at", ""),
                duration_seconds=_coerce_int(row.get("duration_seconds", "0")),
                message_count=_coerce_int(row.get("message_count", "0")),
                tool_count=_coerce_int(row.get("tool_count", "0")),
                subagent_count=_coerce_int(row.get("subagent_count", "0")),
                user_intent=row.get("user_intent", ""),
                is_human_driven=_coerce_bool(row.get("is_human_driven", "")),
                is_long_horizon=_coerce_bool(row.get("is_long_horizon", "")),
                model=row.get("model", ""),
                raw=row.get("raw", ""),
            )
            # Populate user_intent from the raw payload if the CSV column is empty
            if not t.user_intent:
                t.user_intent = t.extract_user_intent()
            if parse_raw and t.raw:
                t.parse_raw()
            traces.append(t)
    return traces


def iter_traces(
    csv_name: str = FULL_CSV,
    traces_dir: Path | str = DEFAULT_TRACES_DIR,
) -> Iterator[CuratedTrace]:
    """Yield traces lazily without keeping all rows in memory."""
    path = Path(traces_dir) / csv_name
    if not path.exists():
        raise FileNotFoundError(f"Curated traces file not found: {path}")
    with path.open(newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            yield CuratedTrace(
                agent=row.get("agent", ""),
                conversation_id=row.get("conversation_id", ""),
                title=row.get("title", ""),
                started_at=row.get("started_at", ""),
                duration_seconds=_coerce_int(row.get("duration_seconds", "0")),
                message_count=_coerce_int(row.get("message_count", "0")),
                tool_count=_coerce_int(row.get("tool_count", "0")),
                subagent_count=_coerce_int(row.get("subagent_count", "0")),
                user_intent=row.get("user_intent", ""),
                is_human_driven=_coerce_bool(row.get("is_human_driven", "")),
                is_long_horizon=_coerce_bool(row.get("is_long_horizon", "")),
                model=row.get("model", ""),
                raw=row.get("raw", ""),
            )


def filter_traces(
    traces: list[CuratedTrace],
    agent: str | None = None,
    human_only: bool = False,
    long_horizon_only: bool = False,
    min_tool_count: int = 0,
) -> list[CuratedTrace]:
    """Filter a list of CuratedTrace objects.

    Args:
        agent: Keep only rows where `agent == agent`.
        human_only: Keep only rows where `is_human_driven == True`.
        long_horizon_only: Keep only rows where `is_long_horizon == True`.
        min_tool_count: Keep only rows with `tool_count >= min_tool_count`.
    """
    out: list[CuratedTrace] = []
    for t in traces:
        if agent is not None and t.agent != agent:
            continue
        if human_only and not t.is_human_driven:
            continue
        if long_horizon_only and not t.is_long_horizon:
            continue
        if t.tool_count < min_tool_count:
            continue
        out.append(t)
    return out


def summary(traces: list[CuratedTrace]) -> dict:
    """Aggregate stats over a list of traces."""
    by_agent: dict[str, dict] = {}
    for t in traces:
        bucket = by_agent.setdefault(
            t.agent,
            {
                "count": 0,
                "human": 0,
                "long_horizon": 0,
                "total_messages": 0,
                "total_tools": 0,
                "total_subagents": 0,
            },
        )
        bucket["count"] += 1
        if t.is_human_driven:
            bucket["human"] += 1
        if t.is_long_horizon:
            bucket["long_horizon"] += 1
        bucket["total_messages"] += t.message_count
        bucket["total_tools"] += t.tool_count
        bucket["total_subagents"] += t.subagent_count
    return {
        "total": len(traces),
        "human": sum(1 for t in traces if t.is_human_driven),
        "long_horizon": sum(1 for t in traces if t.is_long_horizon),
        "by_agent": by_agent,
    }


__all__ = [
    "DEFAULT_TRACES_DIR",
    "FULL_CSV",
    "LITE_CSV",
    "MANIFEST",
    "CuratedTrace",
    "load_traces",
    "iter_traces",
    "filter_traces",
    "summary",
]