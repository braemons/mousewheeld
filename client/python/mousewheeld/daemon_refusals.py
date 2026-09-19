# SPDX-License-Identifier: AGPL-3.0-or-later
"""What this client raises when the daemon says no.

Public, and in a module of its own, because these are the names a caller writes
in an `except` — and the whole point of `mousewheeld.v1.Error` travelling as
itself is that there is something worth catching by name.
"""

from __future__ import annotations


class DaemonRefusedTheRequest(Exception):
    """The daemon refused, and said what to change.

    The same name the browser client uses, deliberately: one API, two clients,
    one word for the thing that happened.

    Four things, and all four are worth having:

    * `error` — stable and machine-readable: `no_device`, `no_such_zone_set`,
      `unresolved_reference`. **This is the one to branch on.** It is additive
      within an API major version: new codes appear, none change meaning.
    * `detail` — one sentence, for a person.
    * `context` — *what to change*: the zone set's name, the field that did not
      fit. Usually the answer.
    * `status` — the gRPC code, as its own name. The category rather than the
      case, and the one thing a caller may act on without reading the rest.

    The first three come from `mousewheeld.v1.Error` in the call's trailing
    metadata, not from parsing `detail`: a client that reads a sentence to find
    out which refusal it was breaks when the sentence is reworded.
    """

    def __init__(self, status: str, error: str, detail: str, context: str = "") -> None:
        super().__init__(f"{error}: {detail}" + (f" ({context})" if context else ""))
        self.status = status
        self.error = error
        self.detail = detail
        self.context = context

    @property
    def retryable(self) -> bool:
        """Whether retrying the identical request could work.

        Only `unavailable`. Everything else is a request to change something,
        and a loop that retried them would hammer a daemon about a typo.
        """
        return self.status == "unavailable"


class DaemonOrBoardIsUnavailable(DaemonRefusedTheRequest):
    """`unavailable` — the daemon is not there, or no board is.

    Named for both cases because both arrive here and a caller treats them the
    same way: nothing about the request is wrong, and it will work when the
    thing that is missing turns up. Its own class because it is the one a rig
    script legitimately *waits* on — a daemon starting, a board being plugged
    in — and waiting on it should not mean catching everything.
    """
