from google.protobuf import struct_pb2 as _struct_pb2
from mousewheeld_client._proto.mousewheeld.v1 import device_pb2 as _device_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ZoneShape(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ZONE_SHAPE_UNSPECIFIED: _ClassVar[ZoneShape]
    ZONE_SHAPE_RECT: _ClassVar[ZoneShape]

class ZoneMetric(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ZONE_METRIC_UNSPECIFIED: _ClassVar[ZoneMetric]
    ZONE_METRIC_DISPLACEMENT: _ClassVar[ZoneMetric]
    ZONE_METRIC_DISTANCE: _ClassVar[ZoneMetric]

class FireRule(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FIRE_RULE_UNSPECIFIED: _ClassVar[FireRule]
    FIRE_RULE_ONCE: _ClassVar[FireRule]
    FIRE_RULE_REARM: _ClassVar[FireRule]

class OutputAction(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    OUTPUT_ACTION_UNSPECIFIED: _ClassVar[OutputAction]
    OUTPUT_ACTION_PULSE: _ClassVar[OutputAction]
    OUTPUT_ACTION_LEVEL: _ClassVar[OutputAction]

class ArmOrigin(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ARM_ORIGIN_UNSPECIFIED: _ClassVar[ArmOrigin]
    ARM_ORIGIN_CURRENT: _ClassVar[ArmOrigin]
    ARM_ORIGIN_ABSOLUTE: _ClassVar[ArmOrigin]
ZONE_SHAPE_UNSPECIFIED: ZoneShape
ZONE_SHAPE_RECT: ZoneShape
ZONE_METRIC_UNSPECIFIED: ZoneMetric
ZONE_METRIC_DISPLACEMENT: ZoneMetric
ZONE_METRIC_DISTANCE: ZoneMetric
FIRE_RULE_UNSPECIFIED: FireRule
FIRE_RULE_ONCE: FireRule
FIRE_RULE_REARM: FireRule
OUTPUT_ACTION_UNSPECIFIED: OutputAction
OUTPUT_ACTION_PULSE: OutputAction
OUTPUT_ACTION_LEVEL: OutputAction
ARM_ORIGIN_UNSPECIFIED: ArmOrigin
ARM_ORIGIN_CURRENT: ArmOrigin
ARM_ORIGIN_ABSOLUTE: ArmOrigin

class ZoneSetNames(_message.Message):
    __slots__ = ("zone_sets",)
    ZONE_SETS_FIELD_NUMBER: _ClassVar[int]
    zone_sets: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, zone_sets: _Optional[_Iterable[str]] = ...) -> None: ...

class ZoneSetFile(_message.Message):
    __slots__ = ("name", "text")
    NAME_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    name: str
    text: str
    def __init__(self, name: _Optional[str] = ..., text: _Optional[str] = ...) -> None: ...

class ZoneSetName(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class WriteZoneSet(_message.Message):
    __slots__ = ("name", "zone_set")
    NAME_FIELD_NUMBER: _ClassVar[int]
    ZONE_SET_FIELD_NUMBER: _ClassVar[int]
    name: str
    zone_set: ZoneSet
    def __init__(self, name: _Optional[str] = ..., zone_set: _Optional[_Union[ZoneSet, _Mapping]] = ...) -> None: ...

class ZoneSet(_message.Message):
    __slots__ = ("schema_url", "zone_set_version", "zones")
    SCHEMA_URL_FIELD_NUMBER: _ClassVar[int]
    ZONE_SET_VERSION_FIELD_NUMBER: _ClassVar[int]
    ZONES_FIELD_NUMBER: _ClassVar[int]
    schema_url: str
    zone_set_version: int
    zones: _containers.RepeatedCompositeFieldContainer[Zone]
    def __init__(self, schema_url: _Optional[str] = ..., zone_set_version: _Optional[int] = ..., zones: _Optional[_Iterable[_Union[Zone, _Mapping]]] = ...) -> None: ...

class Zone(_message.Message):
    __slots__ = ("name", "shape", "axes", "metric", "min_cm", "max_cm", "wrap_cm", "fire", "hysteresis_cm", "level", "output")
    NAME_FIELD_NUMBER: _ClassVar[int]
    SHAPE_FIELD_NUMBER: _ClassVar[int]
    AXES_FIELD_NUMBER: _ClassVar[int]
    METRIC_FIELD_NUMBER: _ClassVar[int]
    MIN_CM_FIELD_NUMBER: _ClassVar[int]
    MAX_CM_FIELD_NUMBER: _ClassVar[int]
    WRAP_CM_FIELD_NUMBER: _ClassVar[int]
    FIRE_FIELD_NUMBER: _ClassVar[int]
    HYSTERESIS_CM_FIELD_NUMBER: _ClassVar[int]
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    name: str
    shape: ZoneShape
    axes: _containers.RepeatedScalarFieldContainer[str]
    metric: ZoneMetric
    min_cm: _containers.RepeatedCompositeFieldContainer[ZoneBound]
    max_cm: _containers.RepeatedCompositeFieldContainer[ZoneBound]
    wrap_cm: float
    fire: FireRule
    hysteresis_cm: float
    level: bool
    output: ZoneOutput
    def __init__(self, name: _Optional[str] = ..., shape: _Optional[_Union[ZoneShape, str]] = ..., axes: _Optional[_Iterable[str]] = ..., metric: _Optional[_Union[ZoneMetric, str]] = ..., min_cm: _Optional[_Iterable[_Union[ZoneBound, _Mapping]]] = ..., max_cm: _Optional[_Iterable[_Union[ZoneBound, _Mapping]]] = ..., wrap_cm: _Optional[float] = ..., fire: _Optional[_Union[FireRule, str]] = ..., hysteresis_cm: _Optional[float] = ..., level: _Optional[bool] = ..., output: _Optional[_Union[ZoneOutput, _Mapping]] = ...) -> None: ...

class ZoneBound(_message.Message):
    __slots__ = ("value", "reference")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    value: float
    reference: str
    def __init__(self, value: _Optional[float] = ..., reference: _Optional[str] = ...) -> None: ...

class ZoneOutput(_message.Message):
    __slots__ = ("line", "action", "ms")
    LINE_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    MS_FIELD_NUMBER: _ClassVar[int]
    line: str
    action: OutputAction
    ms: int
    def __init__(self, line: _Optional[str] = ..., action: _Optional[_Union[OutputAction, str]] = ..., ms: _Optional[int] = ...) -> None: ...

class ArmRequest(_message.Message):
    __slots__ = ("zone_set", "patch", "origin", "label")
    class PatchEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: float
        def __init__(self, key: _Optional[str] = ..., value: _Optional[float] = ...) -> None: ...
    ZONE_SET_FIELD_NUMBER: _ClassVar[int]
    PATCH_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    zone_set: str
    patch: _containers.ScalarMap[str, float]
    origin: ArmOrigin
    label: str
    def __init__(self, zone_set: _Optional[str] = ..., patch: _Optional[_Mapping[str, float]] = ..., origin: _Optional[_Union[ArmOrigin, str]] = ..., label: _Optional[str] = ...) -> None: ...

class ArmedZones(_message.Message):
    __slots__ = ("zone_set", "zone_set_version", "arm_id", "label", "zones")
    ZONE_SET_FIELD_NUMBER: _ClassVar[int]
    ZONE_SET_VERSION_FIELD_NUMBER: _ClassVar[int]
    ARM_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    ZONES_FIELD_NUMBER: _ClassVar[int]
    zone_set: str
    zone_set_version: int
    arm_id: int
    label: str
    zones: _containers.RepeatedCompositeFieldContainer[ZoneStatus]
    def __init__(self, zone_set: _Optional[str] = ..., zone_set_version: _Optional[int] = ..., arm_id: _Optional[int] = ..., label: _Optional[str] = ..., zones: _Optional[_Iterable[_Union[ZoneStatus, _Mapping]]] = ...) -> None: ...

class ZoneStatus(_message.Message):
    __slots__ = ("name", "armed", "fired", "inside", "fired_at_cm", "min_cm", "max_cm", "wrap_cm", "metric")
    NAME_FIELD_NUMBER: _ClassVar[int]
    ARMED_FIELD_NUMBER: _ClassVar[int]
    FIRED_FIELD_NUMBER: _ClassVar[int]
    INSIDE_FIELD_NUMBER: _ClassVar[int]
    FIRED_AT_CM_FIELD_NUMBER: _ClassVar[int]
    MIN_CM_FIELD_NUMBER: _ClassVar[int]
    MAX_CM_FIELD_NUMBER: _ClassVar[int]
    WRAP_CM_FIELD_NUMBER: _ClassVar[int]
    METRIC_FIELD_NUMBER: _ClassVar[int]
    name: str
    armed: bool
    fired: bool
    inside: bool
    fired_at_cm: float
    min_cm: _containers.RepeatedCompositeFieldContainer[ResolvedBound]
    max_cm: _containers.RepeatedCompositeFieldContainer[ResolvedBound]
    wrap_cm: float
    metric: ZoneMetric
    def __init__(self, name: _Optional[str] = ..., armed: _Optional[bool] = ..., fired: _Optional[bool] = ..., inside: _Optional[bool] = ..., fired_at_cm: _Optional[float] = ..., min_cm: _Optional[_Iterable[_Union[ResolvedBound, _Mapping]]] = ..., max_cm: _Optional[_Iterable[_Union[ResolvedBound, _Mapping]]] = ..., wrap_cm: _Optional[float] = ..., metric: _Optional[_Union[ZoneMetric, str]] = ...) -> None: ...

class ResolvedBound(_message.Message):
    __slots__ = ("value",)
    VALUE_FIELD_NUMBER: _ClassVar[int]
    value: float
    def __init__(self, value: _Optional[float] = ...) -> None: ...

class ValidationReport(_message.Message):
    __slots__ = ("ok", "problem", "zone_count", "counts_per_cm", "references")
    OK_FIELD_NUMBER: _ClassVar[int]
    PROBLEM_FIELD_NUMBER: _ClassVar[int]
    ZONE_COUNT_FIELD_NUMBER: _ClassVar[int]
    COUNTS_PER_CM_FIELD_NUMBER: _ClassVar[int]
    REFERENCES_FIELD_NUMBER: _ClassVar[int]
    ok: bool
    problem: str
    zone_count: int
    counts_per_cm: float
    references: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, ok: _Optional[bool] = ..., problem: _Optional[str] = ..., zone_count: _Optional[int] = ..., counts_per_cm: _Optional[float] = ..., references: _Optional[_Iterable[str]] = ...) -> None: ...

class ListZoneSetsRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReadZoneSetSchemaRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReadArmedRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class DisarmRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SaveToFlashRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...
