from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class WatchStateRequest(_message.Message):
    __slots__ = ("rate_hz",)
    RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    rate_hz: int
    def __init__(self, rate_hz: _Optional[int] = ...) -> None: ...

class ZeroRequest(_message.Message):
    __slots__ = ("axes",)
    AXES_FIELD_NUMBER: _ClassVar[int]
    axes: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, axes: _Optional[_Iterable[str]] = ...) -> None: ...

class SetPositionRequest(_message.Message):
    __slots__ = ("axes", "position_cm")
    AXES_FIELD_NUMBER: _ClassVar[int]
    POSITION_CM_FIELD_NUMBER: _ClassVar[int]
    axes: _containers.RepeatedScalarFieldContainer[str]
    position_cm: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, axes: _Optional[_Iterable[str]] = ..., position_cm: _Optional[_Iterable[float]] = ...) -> None: ...

class RigState(_message.Message):
    __slots__ = ("axes", "health", "link")
    AXES_FIELD_NUMBER: _ClassVar[int]
    HEALTH_FIELD_NUMBER: _ClassVar[int]
    LINK_FIELD_NUMBER: _ClassVar[int]
    axes: _containers.RepeatedCompositeFieldContainer[AxisState]
    health: LinkHealth
    link: LinkState
    def __init__(self, axes: _Optional[_Iterable[_Union[AxisState, _Mapping]]] = ..., health: _Optional[_Union[LinkHealth, _Mapping]] = ..., link: _Optional[_Union[LinkState, _Mapping]] = ...) -> None: ...

class LinkState(_message.Message):
    __slots__ = ("connected",)
    CONNECTED_FIELD_NUMBER: _ClassVar[int]
    connected: bool
    def __init__(self, connected: _Optional[bool] = ...) -> None: ...

class AxisState(_message.Message):
    __slots__ = ("name", "counts", "position_cm", "distance_cm", "velocity_cm_s", "device_velocity_cm_s")
    NAME_FIELD_NUMBER: _ClassVar[int]
    COUNTS_FIELD_NUMBER: _ClassVar[int]
    POSITION_CM_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_CM_FIELD_NUMBER: _ClassVar[int]
    VELOCITY_CM_S_FIELD_NUMBER: _ClassVar[int]
    DEVICE_VELOCITY_CM_S_FIELD_NUMBER: _ClassVar[int]
    name: str
    counts: int
    position_cm: float
    distance_cm: float
    velocity_cm_s: float
    device_velocity_cm_s: float
    def __init__(self, name: _Optional[str] = ..., counts: _Optional[int] = ..., position_cm: _Optional[float] = ..., distance_cm: _Optional[float] = ..., velocity_cm_s: _Optional[float] = ..., device_velocity_cm_s: _Optional[float] = ...) -> None: ...

class LinkHealth(_message.Message):
    __slots__ = ("seq_gaps", "ring_drops", "measured_rate_hz", "stale")
    SEQ_GAPS_FIELD_NUMBER: _ClassVar[int]
    RING_DROPS_FIELD_NUMBER: _ClassVar[int]
    MEASURED_RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    STALE_FIELD_NUMBER: _ClassVar[int]
    seq_gaps: int
    ring_drops: int
    measured_rate_hz: float
    stale: bool
    def __init__(self, seq_gaps: _Optional[int] = ..., ring_drops: _Optional[int] = ..., measured_rate_hz: _Optional[float] = ..., stale: _Optional[bool] = ...) -> None: ...

class StreamFrame(_message.Message):
    __slots__ = ("sample", "zone_hit")
    SAMPLE_FIELD_NUMBER: _ClassVar[int]
    ZONE_HIT_FIELD_NUMBER: _ClassVar[int]
    sample: Sample
    zone_hit: ZoneHitEvent
    def __init__(self, sample: _Optional[_Union[Sample, _Mapping]] = ..., zone_hit: _Optional[_Union[ZoneHitEvent, _Mapping]] = ...) -> None: ...

class Sample(_message.Message):
    __slots__ = ("seq", "device_us", "host_monotonic_ns", "lost_before", "axes")
    SEQ_FIELD_NUMBER: _ClassVar[int]
    DEVICE_US_FIELD_NUMBER: _ClassVar[int]
    HOST_MONOTONIC_NS_FIELD_NUMBER: _ClassVar[int]
    LOST_BEFORE_FIELD_NUMBER: _ClassVar[int]
    AXES_FIELD_NUMBER: _ClassVar[int]
    seq: int
    device_us: int
    host_monotonic_ns: int
    lost_before: int
    axes: _containers.RepeatedCompositeFieldContainer[AxisState]
    def __init__(self, seq: _Optional[int] = ..., device_us: _Optional[int] = ..., host_monotonic_ns: _Optional[int] = ..., lost_before: _Optional[int] = ..., axes: _Optional[_Iterable[_Union[AxisState, _Mapping]]] = ...) -> None: ...

class ZoneHitEvent(_message.Message):
    __slots__ = ("zone", "arm_id", "seq", "host_monotonic_ns", "position_cm")
    ZONE_FIELD_NUMBER: _ClassVar[int]
    ARM_ID_FIELD_NUMBER: _ClassVar[int]
    SEQ_FIELD_NUMBER: _ClassVar[int]
    HOST_MONOTONIC_NS_FIELD_NUMBER: _ClassVar[int]
    POSITION_CM_FIELD_NUMBER: _ClassVar[int]
    zone: str
    arm_id: int
    seq: int
    host_monotonic_ns: int
    position_cm: float
    def __init__(self, zone: _Optional[str] = ..., arm_id: _Optional[int] = ..., seq: _Optional[int] = ..., host_monotonic_ns: _Optional[int] = ..., position_cm: _Optional[float] = ...) -> None: ...

class ReadStateRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...
