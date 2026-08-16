#!/usr/bin/env python3
"""Classify spec-work planning sources by exact GitHub visibility.

The script emits metadata and hashes only. It never emits file contents or diff text.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
import sys
from typing import Any
from urllib.parse import urlparse


class ManifestError(RuntimeError):
    """Raised for deterministic manifest failures."""


PRO_PRIMARY_EVIDENCE = "discovery/chatgpt-pro-primary.md"
SOURCE_INVENTORY_FIELDS = (
    "path", "exists", "tracked", "committed_at_head", "working_copy_dirty",
    "head_blob", "working_sha256", "source_route", "reason",
)


def canonical_digest(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def run_git(
    repo: Path,
    args: list[str],
    *,
    check: bool = True,
    timeout: int = 20,
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        ["git", "-C", str(repo), *args],
        check=False,
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    if check and result.returncode != 0:
        command = "git " + " ".join(args)
        message = result.stderr.strip() or result.stdout.strip() or "unknown git error"
        raise ManifestError(f"{command} failed: {message}")
    return result


def git_text(repo: Path, args: list[str], *, check: bool = True) -> str:
    return run_git(repo, args, check=check).stdout.strip()


def repository_root(repo_arg: str) -> Path:
    candidate = Path(repo_arg).expanduser().resolve()
    root = git_text(candidate, ["rev-parse", "--show-toplevel"])
    return Path(root).resolve()


def relative_path(root: Path, value: str) -> str:
    """Return a lexical repo-relative path without following filesystem links."""
    candidate = Path(value).expanduser()
    lexical = Path(os.path.abspath(candidate if candidate.is_absolute() else root / candidate))
    try:
        relative = lexical.relative_to(root)
    except ValueError as error:
        raise ManifestError(f"Path escapes repository root: {value}") from error
    return relative.as_posix()


def lexical_metadata(root: Path, relative: str) -> os.stat_result | None:
    """Reject link components before returning non-following final metadata."""
    current = root
    for part in Path(relative).parts:
        current /= part
        try:
            metadata = os.lstat(current)
        except FileNotFoundError:
            return None
        if stat.S_ISLNK(metadata.st_mode):
            raise ManifestError(f"Symlink paths are not planning inputs: {relative}")
    return metadata if relative else os.lstat(root)


def require_regular_file(root: Path, relative: str) -> os.stat_result | None:
    metadata = lexical_metadata(root, relative)
    if metadata is not None and not stat.S_ISREG(metadata.st_mode):
        raise ManifestError(f"Requested path is not a regular file: {relative}")
    return metadata


def github_slug(remote_url: str) -> str | None:
    value = remote_url.strip()
    scp_match = re.match(r"^(?:[^@]+@)?([^:]+):(.+)$", value)
    if scp_match and "://" not in value:
        host, path = scp_match.groups()
    else:
        parsed = urlparse(value)
        host = parsed.hostname or ""
        path = parsed.path.lstrip("/")
    if host.lower() != "github.com" or not path:
        return None
    return path.removesuffix(".git")


def sanitized_remote(remote_url: str) -> str:
    slug = github_slug(remote_url)
    if slug:
        return f"https://github.com/{slug}.git"
    parsed = urlparse(remote_url)
    if parsed.scheme and parsed.hostname:
        port = f":{parsed.port}" if parsed.port else ""
        return f"{parsed.scheme}://{parsed.hostname}{port}{parsed.path}"
    return "non-github-remote"


def sha256_file(path: Path) -> str | None:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def evidence_exclusion_paths(
    root: Path,
    bundle: str | None,
    values: list[str],
) -> set[str]:
    """Return the coordinator-owned, repo-relative Pro artifact inventory."""
    if bundle is None:
        if values:
            raise ManifestError("--exclude-evidence-path requires --bundle")
        return set()
    bundle_relative = relative_path(root, bundle)
    excluded = set()
    for value in values:
        if not value or "\x00" in value or "\\" in value:
            raise ManifestError(
                f"Unsafe bundle-relative evidence exclusion path: {value}"
            )
        relative = Path(value)
        if relative.is_absolute() or any(
            part in {"", ".", ".."} for part in relative.parts
        ):
            raise ManifestError(
                f"Unsafe bundle-relative evidence exclusion path: {value}"
            )
        excluded.add(
            (PurePosixPath(bundle_relative) / PurePosixPath(value)).as_posix()
        )
    excluded.add(
        (PurePosixPath(bundle_relative) / PRO_PRIMARY_EVIDENCE).as_posix()
    )
    return excluded


def discover_bundle_paths(
    root: Path,
    bundle: str | None,
    excluded: set[str],
) -> list[str]:
    if bundle is None:
        return []
    bundle_relative = relative_path(root, bundle)
    bundle_root = root / bundle_relative
    bundle_metadata = lexical_metadata(root, bundle_relative)
    if bundle_metadata is None or not stat.S_ISDIR(bundle_metadata.st_mode):
        raise ManifestError(f"Spec-work bundle is not a directory: {bundle_relative}")

    bundle_parts = PurePosixPath(bundle_relative).parts

    state_relative = (PurePosixPath(bundle_relative) / "state.json").as_posix()
    state_metadata = require_regular_file(root, state_relative)
    if state_metadata is None:
        raise ManifestError(f"Canonical spec-work state is required: {state_relative}")
    try:
        state = json.loads((root / state_relative).read_text(encoding="utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ManifestError(f"Invalid spec-work state: {state_relative}") from error
    if not isinstance(state, dict) or state.get("schema") != "spec-workflow-state":
        raise ManifestError("Unsupported spec-work state; archive or manually restart the stale bundle")
    work_kind = state.get("work_kind")
    artifacts = state.get("artifacts")
    selected_primary = artifacts.get("primary") if isinstance(artifacts, dict) else None
    expected_primary = {"feature": "FEATURE.md", "bug_fix": "BUG_FIX.md"}.get(work_kind)
    expected_root = ("docs", "features") if work_kind == "feature" else ("docs", "bug_fixes")
    if expected_primary is None or selected_primary != expected_primary or bundle_parts[:2] != expected_root:
        raise ManifestError("Spec-work state has an invalid work_kind/artifacts.primary identity")

    values: list[str] = [
        (PurePosixPath(bundle_relative) / selected_primary).as_posix()
    ]
    for name in ("INTERVIEW.md", "DECISIONS.md", "ACCEPTANCE.md"):
        path = bundle_root / name
        relative = path.relative_to(root).as_posix()
        if require_regular_file(root, relative) is not None:
            values.append(path.relative_to(root).as_posix())
    discovery = bundle_root / "discovery"
    discovery_relative = discovery.relative_to(root).as_posix()
    discovery_metadata = lexical_metadata(root, discovery_relative)
    if discovery_metadata is not None:
        if not stat.S_ISDIR(discovery_metadata.st_mode):
            raise ManifestError(f"Discovery path is not a directory: {discovery_relative}")
        for directory, directory_names, file_names in os.walk(discovery, followlinks=False):
            directory_path = Path(directory)
            for name in list(directory_names):
                relative = (directory_path / name).relative_to(root).as_posix()
                metadata = lexical_metadata(root, relative)
                if metadata is not None and not stat.S_ISDIR(metadata.st_mode):
                    raise ManifestError(f"Discovery path is not a directory: {relative}")
            for name in sorted(file_names):
                if not name.endswith(".md"):
                    continue
                path = directory_path / name
                repo_relative = path.relative_to(root).as_posix()
                require_regular_file(root, repo_relative)
                bundle_path = path.relative_to(bundle_root).as_posix()
                if (
                    bundle_path != PRO_PRIMARY_EVIDENCE
                    and repo_relative not in excluded
                ):
                    values.append(repo_relative)
    return values


def reject_prior_pro_evidence(
    root: Path,
    bundle: str | None,
    paths: list[str],
    excluded: set[str],
) -> None:
    """Prevent a primary planner from consuming its own prior output."""
    forbidden_paths = {
        path
        for path in paths
        if path == PRO_PRIMARY_EVIDENCE or path.endswith(f"/{PRO_PRIMARY_EVIDENCE}")
    }
    forbidden_paths.update(set(paths).intersection(excluded))
    if forbidden_paths:
        forbidden = sorted(forbidden_paths)[0]
        raise ManifestError(f"Prior Pro planning evidence is not an input: {forbidden}")


def remote_head(
    root: Path,
    remote: str,
    branch: str | None,
    verify_remote: bool,
) -> dict[str, Any]:
    if not branch:
        return {"mode": "unavailable", "sha": None, "verified": False, "error": "detached_head"}

    ref = f"refs/heads/{branch}"
    if verify_remote:
        result = run_git(root, ["ls-remote", "--heads", remote, ref], check=False)
        if result.returncode != 0:
            return {
                "mode": "live",
                "sha": None,
                "verified": False,
                "error": "remote_query_failed",
            }
        lines = [line for line in result.stdout.splitlines() if line.strip()]
        if len(lines) != 1:
            return {
                "mode": "live",
                "sha": None,
                "verified": False,
                "error": "remote_branch_not_found",
            }
        return {"mode": "live", "sha": lines[0].split()[0], "verified": True, "error": None}

    tracking_ref = f"refs/remotes/{remote}/{branch}"
    result = run_git(root, ["rev-parse", "--verify", tracking_ref], check=False)
    return {
        "mode": "remote_tracking",
        "sha": result.stdout.strip() if result.returncode == 0 else None,
        "verified": False,
        "error": None if result.returncode == 0 else "remote_tracking_ref_missing",
    }


def path_record(
    root: Path,
    relative: str,
    head: str,
    exact_remote_head: bool,
    live_verified: bool,
    repository_slug: str | None,
) -> dict[str, Any]:
    path = root / relative
    metadata = require_regular_file(root, relative)
    exists = metadata is not None
    tracked = run_git(root, ["ls-files", "--error-unmatch", "--", relative], check=False).returncode == 0
    committed = run_git(root, ["cat-file", "-e", f"{head}:{relative}"], check=False).returncode == 0
    status = git_text(root, ["status", "--porcelain=v1", "--", relative], check=False)
    dirty = bool(status)
    ignored = (
        not tracked
        and run_git(root, ["check-ignore", "-q", "--", relative], check=False).returncode
        == 0
    )
    head_blob = git_text(root, ["rev-parse", f"{head}:{relative}"], check=False) if committed else None
    size = metadata.st_size if metadata is not None else None

    if tracked and exists:
        diff_result = run_git(root, ["diff", "--no-ext-diff", "--binary", head, "--", relative], check=False)
        local_delta_bytes = len(diff_result.stdout.encode("utf-8"))
    elif exists:
        local_delta_bytes = size
    else:
        local_delta_bytes = None

    if not exists:
        route = "blocked"
        reason = "path_missing"
    elif ignored:
        route = "blocked"
        reason = "path_is_git_ignored"
    elif repository_slug and live_verified and exact_remote_head and committed and not dirty:
        route = "github"
        reason = "exact_clean_path_at_verified_remote_commit"
    else:
        route = "local_inline_or_upload"
        if not repository_slug:
            reason = "remote_is_not_github_com"
        elif not live_verified:
            reason = "remote_visibility_not_live_verified"
        elif not exact_remote_head:
            reason = "local_head_not_remote_branch_head"
        elif not committed:
            reason = "path_not_committed_at_local_head"
        else:
            reason = "working_copy_differs_from_local_head"

    return {
        "path": relative,
        "exists": exists,
        "is_file": exists,
        "tracked": tracked,
        "committed_at_head": committed,
        "working_copy_dirty": dirty,
        "status": status or None,
        "head_blob": head_blob,
        "working_sha256": sha256_file(path) if exists else None,
        "working_bytes": size,
        "local_delta_bytes": local_delta_bytes,
        "source_route": route,
        "reason": reason,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Classify spec-work planning paths by exact GitHub visibility without emitting content."
    )
    parser.add_argument("--repo", default=".", help="Repository path (default: current directory).")
    parser.add_argument("--bundle", help="Spec-work bundle directory; adds semantic and discovery Markdown.")
    parser.add_argument("--path", action="append", default=[], help="Additional repo-relative path; repeatable.")
    parser.add_argument(
        "--exclude-evidence-path",
        action="append",
        default=[],
        help=(
            "Literal bundle-relative Pro answer or repair artifact recorded by state; "
            "requires --bundle and repeats for every state-owned artifact."
        ),
    )
    parser.add_argument("--remote", default="origin", help="Git remote name (default: origin).")
    parser.add_argument("--branch", help="Branch override (default: current symbolic branch).")
    parser.add_argument(
        "--verify-remote",
        action="store_true",
        help="Use read-only git ls-remote to verify the exact remote branch head.",
    )
    parser.add_argument("--pretty", action="store_true", help="Pretty-print JSON output.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        root = repository_root(args.repo)
        head = git_text(root, ["rev-parse", "HEAD"])
        branch = args.branch or git_text(root, ["symbolic-ref", "--quiet", "--short", "HEAD"], check=False) or None
        remote_url_result = run_git(root, ["config", "--get", f"remote.{args.remote}.url"], check=False)
        if remote_url_result.returncode != 0:
            raise ManifestError(f"Git remote not found: {args.remote}")
        remote_url = remote_url_result.stdout.strip()
        repository_slug = github_slug(remote_url)
        remote = remote_head(root, args.remote, branch, args.verify_remote)
        exact_remote_head = bool(remote["verified"] and remote["sha"] == head)

        excluded = evidence_exclusion_paths(
            root, args.bundle, args.exclude_evidence_path
        )
        requested = discover_bundle_paths(root, args.bundle, excluded)
        explicit = [relative_path(root, value) for value in args.path]
        reject_prior_pro_evidence(root, args.bundle, explicit, excluded)
        requested.extend(explicit)
        paths = sorted(dict.fromkeys(requested))
        records = [
            path_record(root, path, head, exact_remote_head, bool(remote["verified"]), repository_slug)
            for path in paths
        ]

        routes: dict[str, int] = {}
        for record in records:
            route = record["source_route"]
            routes[route] = routes.get(route, 0) + 1

        source_inventory = [
            {field: record[field] for field in SOURCE_INVENTORY_FIELDS}
            for record in records
        ]
        source_inventory_digest = canonical_digest(source_inventory)
        provenance = {
            "schema": "chatgpt-pro-feature-planner/provenance",
            "repository_slug": repository_slug,
            "branch": branch,
            "local_head": head,
            "remote_branch": {
                field: remote[field]
                for field in ("mode", "sha", "verified", "error")
            },
            "exact_local_head_is_verified_remote_head": exact_remote_head,
            "source_inventory_digest": source_inventory_digest,
        }

        manifest = {
            "schema": "chatgpt-pro-feature-planner/context-manifest",
            "repository_root": str(root),
            "repository_slug": repository_slug,
            "remote": args.remote,
            "remote_url": sanitized_remote(remote_url),
            "branch": branch,
            "local_head": head,
            "remote_branch": remote,
            "exact_local_head_is_verified_remote_head": exact_remote_head,
            "paths": records,
            "source_inventory_digest": source_inventory_digest,
            "provenance_digest": canonical_digest(provenance),
            "summary": {"path_count": len(records), "routes": routes},
        }
        json.dump(manifest, sys.stdout, indent=2 if args.pretty else None, sort_keys=True)
        sys.stdout.write("\n")
        return 0
    except (ManifestError, OSError, subprocess.SubprocessError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
