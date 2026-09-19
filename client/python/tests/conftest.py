# SPDX-License-Identifier: AGPL-3.0-or-later
"""A daemon to talk to: `--simulate`, on a port nobody else has.

`--simulate` is a wheel on a thread behind a real pty, so the daemon runs the
link code it will run against a board. That is what makes these tests worth
having: they exercise the same services, the same refusals and the same stream
a rig does, and they need no hardware.
"""

from __future__ import annotations

import os
import socket
import subprocess
import tempfile
import time
from pathlib import Path

import pytest

from mousewheeld import MousewheeldClient

#: Built by `cargo build --release` in the repository root, two levels up.
DAEMON = Path(__file__).resolve().parents[3] / "target" / "release" / "mousewheeld"
RIG_CONFIG = Path(__file__).resolve().parents[3] / "packaging" / "mousewheeld-rig-config.toml"


def a_free_port() -> int:
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


@pytest.fixture(scope="session")
def wheel():
    if not DAEMON.exists():
        pytest.skip(f"no daemon at {DAEMON} — run `cargo build --release` first")

    port = a_free_port()
    with tempfile.TemporaryDirectory() as scratch:
        # A copy of the rig config, never the repository's own: **the daemon
        # rewrites it** when a calibration is applied, and one test run would
        # otherwise leave the packaged file rewritten and stripped of its
        # comments.
        config = Path(scratch) / "rig-config.toml"
        config.write_text(RIG_CONFIG.read_text())

        daemon = subprocess.Popen(
            [
                str(DAEMON), "serve",
                "--simulate",
                "--port", str(port),
                "--rig-config", str(config),
                "--storage-dir", str(Path(scratch) / "store"),
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        try:
            client = MousewheeldClient(f"127.0.0.1:{port}")
            client.wait_until_ready(timeout_s=10)
            # The link opens on its own, but the first sample takes a moment to
            # arrive; a test that read the state immediately would see a rig
            # that is connected and has never moved, which is true and useless.
            _wait_for_a_sample(client)
            yield client
            client.close()
        finally:
            daemon.terminate()
            daemon.wait(timeout=5)


def _wait_for_a_sample(client: MousewheeldClient, timeout_s: float = 5.0) -> None:
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        state = client.state()
        if state.connected and not state.health.stale:
            return
        time.sleep(0.05)
    raise TimeoutError("the simulated board never sent a sample")


@pytest.fixture
def disarmed(wheel):
    """Nothing armed before the test, and nothing left armed after it."""
    wheel.disarm()
    yield wheel
    wheel.disarm()
