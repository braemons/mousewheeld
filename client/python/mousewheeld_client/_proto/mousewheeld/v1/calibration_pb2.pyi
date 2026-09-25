from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CalibrationState(_message.Message):
    __slots__ = ("axes", "ball")
    AXES_FIELD_NUMBER: _ClassVar[int]
    BALL_FIELD_NUMBER: _ClassVar[int]
    axes: _containers.RepeatedCompositeFieldContainer[AxisCalibration]
    ball: BallCalibration
    def __init__(self, axes: _Optional[_Iterable[_Union[AxisCalibration, _Mapping]]] = ..., ball: _Optional[_Union[BallCalibration, _Mapping]] = ...) -> None: ...

class AxisCalibration(_message.Message):
    __slots__ = ("name", "counts_per_cm", "counts_per_rev", "diameter_cm", "invert", "measured_at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    COUNTS_PER_CM_FIELD_NUMBER: _ClassVar[int]
    COUNTS_PER_REV_FIELD_NUMBER: _ClassVar[int]
    DIAMETER_CM_FIELD_NUMBER: _ClassVar[int]
    INVERT_FIELD_NUMBER: _ClassVar[int]
    MEASURED_AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    counts_per_cm: float
    counts_per_rev: int
    diameter_cm: float
    invert: bool
    measured_at: str
    def __init__(self, name: _Optional[str] = ..., counts_per_cm: _Optional[float] = ..., counts_per_rev: _Optional[int] = ..., diameter_cm: _Optional[float] = ..., invert: _Optional[bool] = ..., measured_at: _Optional[str] = ...) -> None: ...

class BallCalibration(_message.Message):
    __slots__ = ("diameter_cm", "sensor_angles_deg")
    DIAMETER_CM_FIELD_NUMBER: _ClassVar[int]
    SENSOR_ANGLES_DEG_FIELD_NUMBER: _ClassVar[int]
    diameter_cm: float
    sensor_angles_deg: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, diameter_cm: _Optional[float] = ..., sensor_angles_deg: _Optional[_Iterable[float]] = ...) -> None: ...

class CalibrationPatch(_message.Message):
    __slots__ = ("axes",)
    AXES_FIELD_NUMBER: _ClassVar[int]
    axes: _containers.RepeatedCompositeFieldContainer[AxisCalibrationPatch]
    def __init__(self, axes: _Optional[_Iterable[_Union[AxisCalibrationPatch, _Mapping]]] = ...) -> None: ...

class AxisCalibrationPatch(_message.Message):
    __slots__ = ("name", "counts_per_cm", "counts_per_rev", "diameter_cm", "invert")
    NAME_FIELD_NUMBER: _ClassVar[int]
    COUNTS_PER_CM_FIELD_NUMBER: _ClassVar[int]
    COUNTS_PER_REV_FIELD_NUMBER: _ClassVar[int]
    DIAMETER_CM_FIELD_NUMBER: _ClassVar[int]
    INVERT_FIELD_NUMBER: _ClassVar[int]
    name: str
    counts_per_cm: float
    counts_per_rev: int
    diameter_cm: float
    invert: bool
    def __init__(self, name: _Optional[str] = ..., counts_per_cm: _Optional[float] = ..., counts_per_rev: _Optional[int] = ..., diameter_cm: _Optional[float] = ..., invert: _Optional[bool] = ...) -> None: ...

class StartMeasurement(_message.Message):
    __slots__ = ("axis", "known_distance_cm")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    KNOWN_DISTANCE_CM_FIELD_NUMBER: _ClassVar[int]
    axis: str
    known_distance_cm: float
    def __init__(self, axis: _Optional[str] = ..., known_distance_cm: _Optional[float] = ...) -> None: ...

class MeasurementStarted(_message.Message):
    __slots__ = ("axis", "known_distance_cm", "counts_at_start", "counts")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    KNOWN_DISTANCE_CM_FIELD_NUMBER: _ClassVar[int]
    COUNTS_AT_START_FIELD_NUMBER: _ClassVar[int]
    COUNTS_FIELD_NUMBER: _ClassVar[int]
    axis: str
    known_distance_cm: float
    counts_at_start: int
    counts: int
    def __init__(self, axis: _Optional[str] = ..., known_distance_cm: _Optional[float] = ..., counts_at_start: _Optional[int] = ..., counts: _Optional[int] = ...) -> None: ...

class MeasurementResult(_message.Message):
    __slots__ = ("axis", "known_distance_cm", "counts", "measured_counts_per_cm", "configured_counts_per_cm", "counts_per_rev", "nominal_counts_per_cm", "implied_circumference_cm", "decoding_suspect")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    KNOWN_DISTANCE_CM_FIELD_NUMBER: _ClassVar[int]
    COUNTS_FIELD_NUMBER: _ClassVar[int]
    MEASURED_COUNTS_PER_CM_FIELD_NUMBER: _ClassVar[int]
    CONFIGURED_COUNTS_PER_CM_FIELD_NUMBER: _ClassVar[int]
    COUNTS_PER_REV_FIELD_NUMBER: _ClassVar[int]
    NOMINAL_COUNTS_PER_CM_FIELD_NUMBER: _ClassVar[int]
    IMPLIED_CIRCUMFERENCE_CM_FIELD_NUMBER: _ClassVar[int]
    DECODING_SUSPECT_FIELD_NUMBER: _ClassVar[int]
    axis: str
    known_distance_cm: float
    counts: int
    measured_counts_per_cm: float
    configured_counts_per_cm: float
    counts_per_rev: int
    nominal_counts_per_cm: float
    implied_circumference_cm: float
    decoding_suspect: str
    def __init__(self, axis: _Optional[str] = ..., known_distance_cm: _Optional[float] = ..., counts: _Optional[int] = ..., measured_counts_per_cm: _Optional[float] = ..., configured_counts_per_cm: _Optional[float] = ..., counts_per_rev: _Optional[int] = ..., nominal_counts_per_cm: _Optional[float] = ..., implied_circumference_cm: _Optional[float] = ..., decoding_suspect: _Optional[str] = ...) -> None: ...

class MeasurementApplied(_message.Message):
    __slots__ = ("axis", "counts_per_cm", "zone_sets_invalidated")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    COUNTS_PER_CM_FIELD_NUMBER: _ClassVar[int]
    ZONE_SETS_INVALIDATED_FIELD_NUMBER: _ClassVar[int]
    axis: str
    counts_per_cm: float
    zone_sets_invalidated: bool
    def __init__(self, axis: _Optional[str] = ..., counts_per_cm: _Optional[float] = ..., zone_sets_invalidated: _Optional[bool] = ...) -> None: ...

class ReadCalibrationRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReadBallRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class FinishMeasuringRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ApplyMeasurementRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...
