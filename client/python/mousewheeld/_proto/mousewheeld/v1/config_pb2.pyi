from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ConfigView(_message.Message):
    __slots__ = ("rate_hz", "display_hz", "ring_minutes", "shm_name", "shm_open", "shm_writes", "event_port", "port", "starves_the_display")
    RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_HZ_FIELD_NUMBER: _ClassVar[int]
    RING_MINUTES_FIELD_NUMBER: _ClassVar[int]
    SHM_NAME_FIELD_NUMBER: _ClassVar[int]
    SHM_OPEN_FIELD_NUMBER: _ClassVar[int]
    SHM_WRITES_FIELD_NUMBER: _ClassVar[int]
    EVENT_PORT_FIELD_NUMBER: _ClassVar[int]
    PORT_FIELD_NUMBER: _ClassVar[int]
    STARVES_THE_DISPLAY_FIELD_NUMBER: _ClassVar[int]
    rate_hz: int
    display_hz: int
    ring_minutes: int
    shm_name: str
    shm_open: bool
    shm_writes: int
    event_port: int
    port: str
    starves_the_display: bool
    def __init__(self, rate_hz: _Optional[int] = ..., display_hz: _Optional[int] = ..., ring_minutes: _Optional[int] = ..., shm_name: _Optional[str] = ..., shm_open: _Optional[bool] = ..., shm_writes: _Optional[int] = ..., event_port: _Optional[int] = ..., port: _Optional[str] = ..., starves_the_display: _Optional[bool] = ...) -> None: ...

class ConfigPatch(_message.Message):
    __slots__ = ("rate_hz", "display_hz", "ring_minutes")
    RATE_HZ_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_HZ_FIELD_NUMBER: _ClassVar[int]
    RING_MINUTES_FIELD_NUMBER: _ClassVar[int]
    rate_hz: int
    display_hz: int
    ring_minutes: int
    def __init__(self, rate_hz: _Optional[int] = ..., display_hz: _Optional[int] = ..., ring_minutes: _Optional[int] = ...) -> None: ...

class LineMap(_message.Message):
    __slots__ = ("lines",)
    LINES_FIELD_NUMBER: _ClassVar[int]
    lines: _containers.RepeatedCompositeFieldContainer[OutputLine]
    def __init__(self, lines: _Optional[_Iterable[_Union[OutputLine, _Mapping]]] = ...) -> None: ...

class OutputLine(_message.Message):
    __slots__ = ("name", "index", "pin", "safe_high")
    NAME_FIELD_NUMBER: _ClassVar[int]
    INDEX_FIELD_NUMBER: _ClassVar[int]
    PIN_FIELD_NUMBER: _ClassVar[int]
    SAFE_HIGH_FIELD_NUMBER: _ClassVar[int]
    name: str
    index: int
    pin: int
    safe_high: bool
    def __init__(self, name: _Optional[str] = ..., index: _Optional[int] = ..., pin: _Optional[int] = ..., safe_high: _Optional[bool] = ...) -> None: ...

class ReadConfigRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReadLinesRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...
