from mousewheeld_client._proto.mousewheeld.v1 import version_pb2 as _version_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class WireDirection(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    WIRE_DIRECTION_UNSPECIFIED: _ClassVar[WireDirection]
    WIRE_DIRECTION_OUT: _ClassVar[WireDirection]
    WIRE_DIRECTION_IN: _ClassVar[WireDirection]

class WireLevel(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    WIRE_LEVEL_UNSPECIFIED: _ClassVar[WireLevel]
    WIRE_LEVEL_INFO: _ClassVar[WireLevel]
    WIRE_LEVEL_ERROR: _ClassVar[WireLevel]
WIRE_DIRECTION_UNSPECIFIED: WireDirection
WIRE_DIRECTION_OUT: WireDirection
WIRE_DIRECTION_IN: WireDirection
WIRE_LEVEL_UNSPECIFIED: WireLevel
WIRE_LEVEL_INFO: WireLevel
WIRE_LEVEL_ERROR: WireLevel

class DeviceInfo(_message.Message):
    __slots__ = ("connected", "port", "board", "firmware_version", "protocol_version", "uptime_device_us", "capacities", "flashed_zone_set", "link")
    CONNECTED_FIELD_NUMBER: _ClassVar[int]
    PORT_FIELD_NUMBER: _ClassVar[int]
    BOARD_FIELD_NUMBER: _ClassVar[int]
    FIRMWARE_VERSION_FIELD_NUMBER: _ClassVar[int]
    PROTOCOL_VERSION_FIELD_NUMBER: _ClassVar[int]
    UPTIME_DEVICE_US_FIELD_NUMBER: _ClassVar[int]
    CAPACITIES_FIELD_NUMBER: _ClassVar[int]
    FLASHED_ZONE_SET_FIELD_NUMBER: _ClassVar[int]
    LINK_FIELD_NUMBER: _ClassVar[int]
    connected: bool
    port: str
    board: str
    firmware_version: str
    protocol_version: int
    uptime_device_us: int
    capacities: Capacities
    flashed_zone_set: FlashedZoneSet
    link: LinkStats
    def __init__(self, connected: _Optional[bool] = ..., port: _Optional[str] = ..., board: _Optional[str] = ..., firmware_version: _Optional[str] = ..., protocol_version: _Optional[int] = ..., uptime_device_us: _Optional[int] = ..., capacities: _Optional[_Union[Capacities, _Mapping]] = ..., flashed_zone_set: _Optional[_Union[FlashedZoneSet, _Mapping]] = ..., link: _Optional[_Union[LinkStats, _Mapping]] = ...) -> None: ...

class Capacities(_message.Message):
    __slots__ = ("n_axes", "max_zones", "max_lines", "scan_hz")
    N_AXES_FIELD_NUMBER: _ClassVar[int]
    MAX_ZONES_FIELD_NUMBER: _ClassVar[int]
    MAX_LINES_FIELD_NUMBER: _ClassVar[int]
    SCAN_HZ_FIELD_NUMBER: _ClassVar[int]
    n_axes: int
    max_zones: int
    max_lines: int
    scan_hz: int
    def __init__(self, n_axes: _Optional[int] = ..., max_zones: _Optional[int] = ..., max_lines: _Optional[int] = ..., scan_hz: _Optional[int] = ...) -> None: ...

class FlashedZoneSet(_message.Message):
    __slots__ = ("name", "version")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    name: str
    version: int
    def __init__(self, name: _Optional[str] = ..., version: _Optional[int] = ...) -> None: ...

class LinkStats(_message.Message):
    __slots__ = ("connection_count", "last_error")
    CONNECTION_COUNT_FIELD_NUMBER: _ClassVar[int]
    LAST_ERROR_FIELD_NUMBER: _ClassVar[int]
    connection_count: int
    last_error: str
    def __init__(self, connection_count: _Optional[int] = ..., last_error: _Optional[str] = ...) -> None: ...

class WireLine(_message.Message):
    __slots__ = ("host_monotonic_ns", "direction", "text", "level")
    HOST_MONOTONIC_NS_FIELD_NUMBER: _ClassVar[int]
    DIRECTION_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    host_monotonic_ns: int
    direction: WireDirection
    text: str
    level: WireLevel
    def __init__(self, host_monotonic_ns: _Optional[int] = ..., direction: _Optional[_Union[WireDirection, str]] = ..., text: _Optional[str] = ..., level: _Optional[_Union[WireLevel, str]] = ...) -> None: ...

class WireLog(_message.Message):
    __slots__ = ("lines",)
    LINES_FIELD_NUMBER: _ClassVar[int]
    lines: _containers.RepeatedCompositeFieldContainer[WireLine]
    def __init__(self, lines: _Optional[_Iterable[_Union[WireLine, _Mapping]]] = ...) -> None: ...

class FirmwareVersions(_message.Message):
    __slots__ = ("running", "available")
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    AVAILABLE_FIELD_NUMBER: _ClassVar[int]
    running: str
    available: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, running: _Optional[str] = ..., available: _Optional[_Iterable[str]] = ...) -> None: ...

class ReadVersionRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReadDeviceRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class OpenLinkRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReadFirmwareRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReadWireLogRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class WatchWireRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...
