#!/usr/bin/env python3
"""Render a send-ready ChatGPT Pro prompt from one context manifest."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import sys
from typing import Any


class PromptPreparationError(RuntimeError):
    """Raised when manifest-bound prompt preparation must fail closed."""


SOURCE_INVENTORY_FIELDS = (
    "path", "exists", "tracked", "committed_at_head", "working_copy_dirty",
    "head_blob", "working_sha256", "source_route", "reason",
)
MANIFEST_SCHEMA = "chatgpt-pro-feature-planner/context-manifest"
PROVENANCE_SCHEMA = "chatgpt-pro-feature-planner/provenance"
RESULT_SCHEMA = "chatgpt-pro-feature-planner/prepared-prompt"


def canonical_digest(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_string(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value:
        raise PromptPreparationError(f"field must be a non-empty string: {field}")
    return value


def validate_path(value: Any, field: str) -> str:
    path = require_string(value, field)
    pure = PurePosixPath(path)
    if pure.is_absolute() or any(part in {"", ".", ".."} for part in pure.parts):
        raise PromptPreparationError(f"unsafe repository-relative path in {field}: {path}")
    return path


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        manifest = json.loads(path.read_bytes())
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise PromptPreparationError(f"invalid context manifest: {path}") from error
    if not isinstance(manifest, dict) or manifest.get("schema") != MANIFEST_SCHEMA:
        raise PromptPreparationError("unsupported context manifest schema")
    return manifest


def validate_manifest(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    records = manifest.get("paths")
    if not isinstance(records, list) or any(not isinstance(record, dict) for record in records):
        raise PromptPreparationError("manifest paths must be an array of records")

    paths = [validate_path(record.get("path"), "paths[].path") for record in records]
    if paths != sorted(paths) or len(paths) != len(set(paths)):
        raise PromptPreparationError("manifest paths must be unique and sorted")

    inventory: list[dict[str, Any]] = []
    for record in records:
        missing = [field for field in SOURCE_INVENTORY_FIELDS if field not in record]
        if missing:
            raise PromptPreparationError(
                f"manifest path record is missing inventory field: {missing[0]}"
            )
        route = record["source_route"]
        if route not in {"github", "local_inline_or_upload", "blocked"}:
            raise PromptPreparationError(f"unsupported manifest route: {route}")
        inventory.append({field: record[field] for field in SOURCE_INVENTORY_FIELDS})

    inventory_digest = canonical_digest(inventory)
    if manifest.get("source_inventory_digest") != inventory_digest:
        raise PromptPreparationError("manifest source-inventory digest mismatch")

    remote = manifest.get("remote_branch")
    if not isinstance(remote, dict):
        raise PromptPreparationError("manifest remote_branch must be an object")
    provenance = {
        "schema": PROVENANCE_SCHEMA,
        "repository_slug": manifest.get("repository_slug"),
        "branch": manifest.get("branch"),
        "local_head": manifest.get("local_head"),
        "remote_branch": {
            field: remote.get(field)
            for field in ("mode", "sha", "verified", "error")
        },
        "exact_local_head_is_verified_remote_head": manifest.get(
            "exact_local_head_is_verified_remote_head"
        ),
        "source_inventory_digest": inventory_digest,
    }
    if manifest.get("provenance_digest") != canonical_digest(provenance):
        raise PromptPreparationError("manifest provenance digest mismatch")

    blocked = [record["path"] for record in records if record["source_route"] == "blocked"]
    if blocked:
        raise PromptPreparationError(f"blocked manifest route: {blocked[0]}")
    if manifest.get("exact_local_head_is_verified_remote_head") is not True:
        raise PromptPreparationError("manifest source commit is not the live verified remote head")
    require_string(manifest.get("repository_slug"), "repository_slug")
    require_string(manifest.get("branch"), "branch")
    head = require_string(manifest.get("local_head"), "local_head")
    if remote.get("verified") is not True or remote.get("sha") != head:
        raise PromptPreparationError("manifest remote branch does not verify the local head")
    return records


def local_content(manifest: dict[str, Any], record: dict[str, Any]) -> str:
    root = Path(require_string(manifest.get("repository_root"), "repository_root")).resolve()
    relative = validate_path(record.get("path"), "paths[].path")
    candidate = root / relative
    try:
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(root)
        content = resolved.read_bytes()
    except (OSError, ValueError) as error:
        raise PromptPreparationError(f"local-only manifest path is unavailable: {relative}") from error
    expected_digest = require_string(record.get("working_sha256"), "paths[].working_sha256")
    if sha256_bytes(content) != expected_digest:
        raise PromptPreparationError(f"local-only content no longer matches the manifest: {relative}")
    try:
        return content.decode("utf-8")
    except UnicodeDecodeError as error:
        raise PromptPreparationError(f"local-only content is not UTF-8: {relative}") from error


def markdown_fence(content: str) -> str:
    longest = max((len(match.group(0)) for match in re.finditer(r"`+", content)), default=0)
    fence = "`" * max(3, longest + 1)
    suffix = "" if content.endswith("\n") else "\n"
    return f"{fence}markdown\n{content}{suffix}{fence}"


def render_local_blocks(manifest: dict[str, Any], records: list[dict[str, Any]]) -> str:
    local_records = [
        record for record in records if record["source_route"] == "local_inline_or_upload"
    ]
    if not local_records:
        return "None."
    base = require_string(manifest.get("local_head"), "local_head")
    blocks = []
    for record in local_records:
        path = record["path"]
        digest = require_string(record.get("working_sha256"), "paths[].working_sha256")
        blocks.append(
            "\n".join(
                (
                    f"### `local-only:{path}`",
                    "",
                    f"- Base commit: `{base}`",
                    f"- Working-copy SHA-256: `{digest}`",
                    "",
                    markdown_fence(local_content(manifest, record)),
                )
            )
        )
    return "\n\n".join(blocks)


def prompt_template(path: Path) -> str:
    try:
        source = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise PromptPreparationError(f"planning prompt template is unavailable: {path}") from error
    marker = "```text\n"
    start = source.find(marker)
    end = source.rfind("\n```")
    if start < 0 or end <= start:
        raise PromptPreparationError("planning prompt template has no canonical text block")
    return source[start + len(marker):end]


def fill_template(template: str, replacements: dict[str, str]) -> str:
    for placeholder in replacements:
        if template.count(placeholder) != 1:
            raise PromptPreparationError(
                f"planning prompt template must contain exactly one {placeholder}"
            )
    placeholder_pattern = r"<[A-Z][A-Z0-9_/ -]*>"
    unresolved = [
        placeholder
        for placeholder in re.findall(placeholder_pattern, template)
        if placeholder not in replacements
    ]
    if unresolved:
        raise PromptPreparationError(f"unresolved planning prompt placeholder: {unresolved[0]}")
    prompt = re.sub(
        placeholder_pattern,
        lambda match: replacements[match.group(0)],
        template,
    )
    return prompt.rstrip() + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render a send-ready planning prompt from one context manifest."
    )
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--work-id", required=True)
    parser.add_argument("--work-bundle", required=True)
    parser.add_argument("--specification-digest", required=True)
    parser.add_argument("--work-goal", required=True)
    parser.add_argument("--decision-and-acceptance-anchors", required=True)
    parser.add_argument(
        "--requested-path",
        action="append",
        default=[],
        help="Initial evidence path requested by the caller; repeatable.",
    )
    parser.add_argument(
        "--template",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "assets/planning-prompt.md",
    )
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        manifest = load_manifest(args.manifest)
        records = validate_manifest(manifest)
        work_id = require_string(args.work_id, "work_id")
        work_bundle = validate_path(args.work_bundle, "work_bundle")
        specification_digest = require_string(
            args.specification_digest, "specification_digest"
        )
        if re.fullmatch(r"[0-9a-f]{64}", specification_digest) is None:
            raise PromptPreparationError(
                "specification_digest must be a lowercase SHA-256 digest"
            )
        work_goal = require_string(args.work_goal, "work_goal")
        anchors = require_string(
            args.decision_and_acceptance_anchors,
            "decision_and_acceptance_anchors",
        )
        inventory = {record["path"] for record in records}
        for requested in args.requested_path:
            path = validate_path(requested, "requested_path")
            if path not in inventory:
                raise PromptPreparationError(
                    f"requested path is absent from the manifest: {path}"
                )

        github_paths = [
            record["path"] for record in records if record["source_route"] == "github"
        ]
        github_list = "\n".join(f"  - `{path}`" for path in github_paths) or "  - None."
        remote = manifest["remote_branch"]
        replacements = {
            "<WORK_ID>": work_id,
            "<WORK_BUNDLE_PATH>": work_bundle,
            "<SPECIFICATION_DIGEST>": specification_digest,
            "<OWNER/REPO>": manifest["repository_slug"],
            "<BRANCH>": manifest["branch"],
            "<REMOTE_COMMIT_SHA>": remote["sha"],
            "<PROVENANCE_DIGEST>": manifest["provenance_digest"],
            "<SOURCE_INVENTORY_DIGEST>": manifest["source_inventory_digest"],
            "<GITHUB_PATH_LIST>": github_list,
            "<WORK_GOAL>": work_goal,
            "<DECISION_AND_ACCEPTANCE_IDS>": anchors,
            "<LOCAL_BASE_COMMIT>": manifest["local_head"],
            "<PATH_OR_DIFF_LABEL>": "manifest-approved-path-or-diff-label",
            "<LOCAL_ONLY_BLOCKS_OR_NONE>": render_local_blocks(manifest, records),
        }
        prompt = fill_template(prompt_template(args.template), replacements)
        prompt_bytes = prompt.encode("utf-8")
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_bytes(prompt_bytes)
        result = {
            "schema": RESULT_SCHEMA,
            "output": str(args.output),
            "prompt_bytes": len(prompt_bytes),
            "prompt_sha256": sha256_bytes(prompt_bytes),
            "provenance_digest": manifest["provenance_digest"],
            "source_inventory_digest": manifest["source_inventory_digest"],
            "github_path_count": len(github_paths),
            "local_only_path_count": sum(
                record["source_route"] == "local_inline_or_upload" for record in records
            ),
        }
        json.dump(result, sys.stdout, sort_keys=True)
        sys.stdout.write("\n")
        return 0
    except (PromptPreparationError, OSError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
