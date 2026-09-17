#!/bin/sh
# Stop the daemon before the binary goes. The rig config and the zone-set store
# stay: they are the rig's, not the package's.
set -e
systemctl stop mousewheeld >/dev/null 2>&1 || true
systemctl disable mousewheeld >/dev/null 2>&1 || true
