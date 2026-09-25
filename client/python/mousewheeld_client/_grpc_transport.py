# SPDX-License-Identifier: AGPL-3.0-or-later
"""The gRPC channel, and the one place a transport failure becomes a refusal.

Nothing above this imports grpc, and nothing below it knows what a zone is.
The refusals themselves are `daemon_refusals.py`, because those are public and
this is not.
"""

from __future__ import annotations

from collections.abc import Callable, Iterator
from typing import TypeVar

import grpc

from mousewheeld_client._proto.mousewheeld.v1 import error_pb2

from .daemon_refusals import DaemonOrBoardIsUnavailable, DaemonRefusedTheRequest

#: Where the daemon puts the refusal as itself. See `daemon/src/grpc/mod.rs`.
REFUSAL_METADATA_KEY = "mousewheeld-error-bin"


def refusal_of(error: grpc.RpcError) -> DaemonRefusedTheRequest:
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
    kind = DaemonOrBoardIsUnavailable if status == "unavailable" else DaemonRefusedTheRequest
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
