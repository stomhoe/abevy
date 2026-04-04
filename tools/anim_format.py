from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Mapping


def extract_anim_blocks(text: str) -> list[str]:
    blocks: list[str] = []
    current: list[str] = []
    depth = 0
    in_block = False

    for line in text.splitlines():
        stripped = line.lstrip()
        if not in_block:
            if not stripped.startswith("anim "):
                continue
            in_block = True
            current = [line]
            depth = line.count("{") - line.count("}")
            continue

        current.append(line)
        depth += line.count("{") - line.count("}")
        if depth == 0:
            blocks.append("\n".join(current))
            in_block = False

    return blocks


@dataclass(frozen=True)
class AnimHeader:
    anim_id: str
    base_id: str | None
    is_abstract: bool


def parse_header(block: str) -> AnimHeader:
    header = block.splitlines()[0].strip()
    if not header.endswith("{"):
        raise ValueError("animation block header must end with '{'")

    tokens = header[:-1].strip().split()
    if len(tokens) < 2 or tokens[0] != "anim":
        raise ValueError("animation block header must start with 'anim <id>'")

    anim_id = tokens[1]
    base_id = None
    is_abstract = False

    idx = 2
    while idx < len(tokens):
        token = tokens[idx]
        if token == "extends":
            if idx + 1 >= len(tokens):
                raise ValueError("extends is missing a base id")
            base_id = tokens[idx + 1]
            idx += 2
            continue
        if token == "abstract":
            is_abstract = True
            idx += 1
            continue
        raise ValueError(f"unexpected token in animation header: {token}")

    return AnimHeader(anim_id=anim_id, base_id=base_id, is_abstract=is_abstract)


def get_field_value(block: str, field: str) -> str | None:
    match = re.search(rf"^\s*{re.escape(field)}\s*:\s*(.*?)\s*$", block, re.M)
    if match is not None:
        return match.group(1)
    match = re.search(rf"^\s*{re.escape(field)}\s*=\s*(.*?)\s*$", block, re.M)
    if match is not None:
        return match.group(1)
    return None


def parse_text_value(raw: str) -> str:
    raw = raw.strip().rstrip(",").strip()
    if len(raw) >= 2 and raw[0] == raw[-1] == '"':
        return raw[1:-1]
    return raw


def extract_section(block: str, section_name: str) -> str | None:
    start = None
    lines = block.splitlines()
    for idx, line in enumerate(lines):
        if line.strip() == f"{section_name} {{" or line.strip() == f"{section_name}: {{" or line.strip() == f"{section_name}:":
            start = idx
            break
    if start is None:
        return None

    body_lines: list[str] = []
    depth = 0
    for line in lines[start:]:
        depth += line.count("{") - line.count("}")
        if depth <= 0:
            break
        if len(body_lines) == 0:
            body_lines.append("")
            continue
        body_lines.append(line)
    if not body_lines:
        return ""

    body_lines = body_lines[1:]
    while body_lines and not body_lines[-1].strip():
        body_lines.pop()
    return "\n".join(body_lines)


def parse_clip_line(line: str) -> dict[str, object]:
    tokens = line.strip().split()
    if not tokens:
        raise ValueError("empty clip line")
    if tokens[0] == "clip":
        tokens = tokens[1:]
    if len(tokens) < 2:
        raise ValueError("clip line is missing a target")

    kind = tokens[0]
    if kind not in {"row", "col", "column"}:
        raise ValueError(f"clip line must start with row/col/column, found '{kind}'")

    target = int(tokens[1])
    is_row = kind == "row"
    if kind == "column":
        is_row = False

    out: dict[str, object] = {
        "target": target,
        "is_row": is_row,
        "partial": None,
        "start_frame": None,
        "dir": None,
        "reps": None,
        "dur_frame": None,
        "dur_rep": None,
    }

    idx = 2
    while idx < len(tokens):
        key = tokens[idx]
        idx += 1
        if key == "is_row":
            out["is_row"] = tokens[idx].lower() == "true"
            idx += 1
        elif key == "partial":
            out["partial"] = (int(tokens[idx]), int(tokens[idx + 1]))
            idx += 2
        elif key in {"start", "start_frame"}:
            out["start_frame"] = int(tokens[idx])
            idx += 1
        elif key == "dir":
            out["dir"] = tokens[idx]
            idx += 1
        elif key == "reps":
            out["reps"] = int(tokens[idx])
            idx += 1
        elif key == "dur_frame":
            out["dur_frame"] = int(tokens[idx])
            idx += 1
        elif key == "dur_rep":
            out["dur_rep"] = int(tokens[idx])
            idx += 1
        else:
            raise ValueError(f"unknown clip token '{key}'")

    return out


def format_clip_line(clip: Mapping[str, object], indent: str = "        ") -> str:
    parts = [f"{indent}{'row' if clip['is_row'] else 'col'} {clip['target']}"]
    if clip["partial"] is not None:
        start, end = clip["partial"]  # type: ignore[misc]
        parts.append(f"partial {start} {end}")
    if clip["start_frame"] is not None:
        parts.append(f"start_frame {clip['start_frame']}")
    if clip["dir"] is not None:
        parts.append(f"dir {clip['dir']}")
    if clip["reps"] is not None:
        parts.append(f"reps {clip['reps']}")
    if clip["dur_frame"] is not None:
        parts.append(f"dur_frame {clip['dur_frame']}")
    if clip["dur_rep"] is not None:
        parts.append(f"dur_rep {clip['dur_rep']}")
    return " ".join(parts)
