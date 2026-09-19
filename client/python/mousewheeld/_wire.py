# SPDX-License-Identifier: AGPL-3.0-or-later
"""The channel, and the one place a gRPC failure becomes this package's own.

Nothing above this imports grpc, and nothing below it knows what a zone is.
"""

from __future__ import annotations

from collections.abc import Callable, Iterator
from typing import TypeVar

import grpc

from mousewheeld.v1 import error_pb2  # ty: ignore[unresolved-import]  (resolved at runtime by __init__'s __path__)

#: Where the daemon puts the refusal as itself. See `daemon/src/grpc/mod.rs`.
REFUSAL_METADATA_KEY = "mousewheeld-error-bin"


class Refused(Exception):
    """The daemon refused, and said what to change.

    Four things, and all four are worth having:

    * `error` — stable and machine-readable: `no_device`, `no_such_zone_set`,
      `unresolved_reference`. **This is the one to branch on.** It is additive
      within an API major version: new codes appear, none change meaning.
    * `detail` — one sentence, for a person.
    * `context` — *what to change*: the zone set's name, the field that did not
      fit. Usually the answer.
    * `status` — the gRPC code, as its own name. The category rather than the
      case, and the one thing a caller may act on without reading the rest:
      `unavailable` means nothing about the request is wrong and it will work
      when the link comes back.

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


class NotConnected(Refused):
    """`unavailable` — the daemon is not there, or no board is.

    Its own class because it is the one a rig script legitimately waits on: a
    daemon starting, a board being plugged in. Everything else is a refusal to
    fix rather than to wait out.
    """


def refusal_of(error: grpc.RpcError) -> Refused:
    """A gRPC failure, as this package's refusal.

    A call that never landed — no daemon, a closed channel, a deadline — has no
    `mousewheeld.v1.Error` to carry, because nothing refused anything. It keeps
    the gRPC code and whatever the transport said.
    """
    status = error.code().name.lower()  # ty: ignore[unresolved-attribute]
    detail = error.details() or status  # ty: ignore[unresolved-attribute]
    body = _typed_refusal(error)
    if body is None:
        body = error_pb2.Error(error=status, detail=detail)
    kind = NotConnected if status == "unavailable" else Refused
    return kind(status, body.error, body.detail, body.context)


def _typed_refusal(error: grpc.RpcError) -> error_pb2.Error | None:
    for key, value in error.trailing_metadata() or ():  # ty: ignore[unresolved-attribute]
        if key == REFUSAL_METADATA_KEY:
            try:
                return error_pb2.Error.FromString(value)
            except Exception:
                return None  # a refusal we cannot read is still a refusal
    return None


_T = TypeVar("_T")


def call(action: Callable[[], _T]) -> _T:
    """One rpc, with a gRPC failure turned into a refusal."""
    try:
        return action()
    except grpc.RpcError as error:
        raise refusal_of(error) from None


def stream(action: Callable[[], Iterator[_T]]) -> Iterator[_T]:
    """One server-streaming rpc.

    The refusal has to be raised from inside the iteration, because that is
    where gRPC raises it: a stream that fails after a thousand frames fails on
    the thousand-and-first `next`, not on the call that opened it.
    """
    try:
        yield from action()
    except grpc.RpcError as error:
        if error.code() is grpc.StatusCode.CANCELLED:  # ty: ignore[unresolved-attribute]
            return  # our own close, not a failure
        raise refusal_of(error) from None
