#!/usr/bin/env python3
"""Pure fail-closed readiness classifier for the accepted phone management plane.

This module consumes normalized non-secret observations only. It performs no
network, SSH, DNS, subprocess, filesystem mutation, or device operation.
"""
from __future__ import annotations

import argparse
import json
import sys
from typing import NamedTuple

DEVICE_OFFLINE = "DEVICE_OFFLINE"
TAILNET_ONLY = "TAILNET_ONLY"
SSH_READY = "SSH_READY"
UBUNTU_PERSISTENT_READY = "UBUNTU_PERSISTENT_READY"

FIELDS = (
    "tailnet_reachable",
    "ssh_tcp_reachable",
    "ssh_authenticated",
    "ubuntu_context_reachable",
    "persistent_context_proven",
)


class Decision(NamedTuple):
    state: str
    claim_allowed: bool
    reason: str


def _valid_observation(value: object) -> bool:
    return value is None or isinstance(value, bool)


def _contradictory(snapshot: dict[str, object]) -> bool:
    tailnet = snapshot.get("tailnet_reachable")
    ssh_tcp = snapshot.get("ssh_tcp_reachable")
    ssh_auth = snapshot.get("ssh_authenticated")
    ubuntu = snapshot.get("ubuntu_context_reachable")
    persistent = snapshot.get("persistent_context_proven")

    if tailnet is False and any(v is True for v in (ssh_tcp, ssh_auth, ubuntu, persistent)):
        return True
    if ssh_tcp is False and any(v is True for v in (ssh_auth, ubuntu, persistent)):
        return True
    if ssh_auth is False and any(v is True for v in (ubuntu, persistent)):
        return True
    if ubuntu is False and persistent is True:
        return True
    return False


def _lowest_safe_state(snapshot: dict[str, object]) -> str:
    if snapshot.get("tailnet_reachable") is not True:
        return DEVICE_OFFLINE
    if snapshot.get("ssh_tcp_reachable") is not True or snapshot.get("ssh_authenticated") is not True:
        return TAILNET_ONLY
    if snapshot.get("ubuntu_context_reachable") is not True or snapshot.get("persistent_context_proven") is not True:
        return SSH_READY
    return UBUNTU_PERSISTENT_READY


def evaluate(snapshot: object) -> Decision:
    if not isinstance(snapshot, dict):
        return Decision(DEVICE_OFFLINE, False, "snapshot-invalid")

    for field in FIELDS:
        if field not in snapshot:
            return Decision(_lowest_safe_state(snapshot), False, f"{field}-missing")
        if not _valid_observation(snapshot[field]):
            return Decision(_lowest_safe_state(snapshot), False, f"{field}-invalid")

    if _contradictory(snapshot):
        return Decision(_lowest_safe_state(snapshot), False, "contradictory-evidence")

    tailnet = snapshot["tailnet_reachable"]
    ssh_tcp = snapshot["ssh_tcp_reachable"]
    ssh_auth = snapshot["ssh_authenticated"]
    ubuntu = snapshot["ubuntu_context_reachable"]
    persistent = snapshot["persistent_context_proven"]

    if tailnet is not True:
        return Decision(DEVICE_OFFLINE, False, "tailnet-unreachable" if tailnet is False else "tailnet-not-proven")
    if ssh_tcp is not True:
        return Decision(TAILNET_ONLY, False, "ssh-tcp-unreachable" if ssh_tcp is False else "ssh-tcp-not-proven")
    if ssh_auth is not True:
        return Decision(TAILNET_ONLY, False, "ssh-auth-failed" if ssh_auth is False else "ssh-auth-not-proven")
    if ubuntu is not True:
        return Decision(SSH_READY, False, "ubuntu-context-unreachable" if ubuntu is False else "ubuntu-context-not-proven")
    if persistent is not True:
        return Decision(SSH_READY, False, "persistent-context-failed" if persistent is False else "persistent-context-not-proven")

    return Decision(UBUNTU_PERSISTENT_READY, True, "authorized")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.parse_args()
    try:
        snapshot = json.load(sys.stdin)
    except (json.JSONDecodeError, UnicodeError):
        snapshot = None
    decision = evaluate(snapshot)
    print(json.dumps({"state": decision.state, "claim_allowed": decision.claim_allowed, "reason": decision.reason}, separators=(",", ":")))
    return 0 if decision.claim_allowed else 3


if __name__ == "__main__":
    raise SystemExit(main())
