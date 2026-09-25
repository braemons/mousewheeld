#!/bin/sh
# Create the user, pick up the unit and the rule, and leave the daemon stopped.
#
# **Not started here.** A rig has one wheel and one board, and starting a daemon
# that grabs a serial port during an install — possibly mid-session on a machine
# somebody is using — is not the installer's decision to make. The last line
# says what to run.
set -e

systemd-sysusers /usr/lib/sysusers.d/mousewheeld.conf >/dev/null 2>&1 || true
systemctl daemon-reload >/dev/null 2>&1 || true
udevadm control --reload-rules >/dev/null 2>&1 || true
udevadm trigger --subsystem-match=tty >/dev/null 2>&1 || true

if [ ! -d /var/lib/braemons/mousewheeld ]; then
  mkdir -p /var/lib/braemons/mousewheeld
  chown mousewheeld:mousewheeld /var/lib/braemons/mousewheeld 2>/dev/null || true
fi

cat <<'MESSAGE'
mousewheeld is installed and not running.

  Edit  /etc/braemons/mousewheeld-rig-config.toml   (the port, the lines, the starting calibration)
  Then  systemctl enable --now mousewheeld
  Panels at  http://<this rig>:8083/     gRPC on the same port (grpcurl -plaintext <rig>:8083 list)

With no board attached, `mousewheeld serve --simulate` runs a wheel on a thread.
MESSAGE
