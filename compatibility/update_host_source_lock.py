"""Refresh every locked host source against one final reviewed repository commit."""

import hashlib
import json
import subprocess
import sys
from pathlib import Path


def git(host: Path, *args: str) -> bytes:
    return subprocess.run(
        ["git", "-C", str(host), *args],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ).stdout


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: update_host_source_lock.py HOST_REPOSITORY FINAL_HOST_COMMIT")

    repo = Path(__file__).resolve().parents[1]
    host = Path(sys.argv[1]).resolve()
    requested_commit = sys.argv[2]
    commit = git(host, "rev-parse", "--verify", requested_commit + "^{commit}").decode().strip()
    lock_path = repo / "compatibility/host-source-lock.json"
    lock = json.loads(lock_path.read_text())

    files = {}
    blobs = {}
    for path in lock["files"]:
        content = git(host, "show", f"{commit}:{path}")
        files[path] = hashlib.sha256(content).hexdigest()
        git_blob = b"blob " + str(len(content)).encode() + b"\0" + content
        blobs[path] = hashlib.sha1(git_blob).hexdigest()

    lock["commit"] = commit
    lock["files"] = files
    lock["gitBlobSha"] = blobs
    lock_path.write_text(json.dumps(lock, indent=2) + "\n")


if __name__ == "__main__":
    main()
