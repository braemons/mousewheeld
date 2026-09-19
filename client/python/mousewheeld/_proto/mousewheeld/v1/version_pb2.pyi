from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class VersionReport(_message.Message):
    __slots__ = ("daemon", "api", "device_protocol", "vinput_layout")
    DAEMON_FIELD_NUMBER: _ClassVar[int]
    API_FIELD_NUMBER: _ClassVar[int]
    DEVICE_PROTOCOL_FIELD_NUMBER: _ClassVar[int]
    VINPUT_LAYOUT_FIELD_NUMBER: _ClassVar[int]
    daemon: str
    api: int
    device_protocol: DeviceProtocol
    vinput_layout: int
    def __init__(self, daemon: _Optional[str] = ..., api: _Optional[int] = ..., device_protocol: _Optional[_Union[DeviceProtocol, _Mapping]] = ..., vinput_layout: _Optional[int] = ...) -> None: ...

class DeviceProtocol(_message.Message):
    __slots__ = ("speaks", "floor", "board")
    SPEAKS_FIELD_NUMBER: _ClassVar[int]
    FLOOR_FIELD_NUMBER: _ClassVar[int]
    BOARD_FIELD_NUMBER: _ClassVar[int]
    speaks: int
    floor: int
    board: int
    def __init__(self, speaks: _Optional[int] = ..., floor: _Optional[int] = ..., board: _Optional[int] = ...) -> None: ...
