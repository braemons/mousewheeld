impl serde::Serialize for ArmOrigin {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::ArmOriginUnspecified => "ARM_ORIGIN_UNSPECIFIED",
            Self::ArmOriginCurrent => "ARM_ORIGIN_CURRENT",
            Self::ArmOriginAbsolute => "ARM_ORIGIN_ABSOLUTE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ArmOrigin {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ARM_ORIGIN_UNSPECIFIED",
            "ARM_ORIGIN_CURRENT",
            "ARM_ORIGIN_ABSOLUTE",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = ArmOrigin;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "ARM_ORIGIN_UNSPECIFIED" => Ok(ArmOrigin::ArmOriginUnspecified),
                    "ARM_ORIGIN_CURRENT" => Ok(ArmOrigin::ArmOriginCurrent),
                    "ARM_ORIGIN_ABSOLUTE" => Ok(ArmOrigin::ArmOriginAbsolute),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ArmRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ArmRequest", len)?;
        if true {
            struct_ser.serialize_field("zone_set", &self.zone_set)?;
        }
        if true {
            struct_ser.serialize_field("patch", &self.patch)?;
        }
        if true {
            let v = ArmOrigin::try_from(self.origin)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.origin)))?;
            struct_ser.serialize_field("origin", &v)?;
        }
        if let Some(v) = self.label.as_ref() {
            struct_ser.serialize_field("label", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ArmRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "zone_set",
            "patch",
            "origin",
            "label",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ZoneSet,
            Patch,
            Origin,
            Label,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "zone_set" => Ok(GeneratedField::ZoneSet),
                            "patch" => Ok(GeneratedField::Patch),
                            "origin" => Ok(GeneratedField::Origin),
                            "label" => Ok(GeneratedField::Label),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ArmRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ArmRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ArmRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut zone_set__ = None;
                let mut patch__ = None;
                let mut origin__ = None;
                let mut label__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ZoneSet => {
                            if zone_set__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zone_set"));
                            }
                            zone_set__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Patch => {
                            if patch__.is_some() {
                                return Err(serde::de::Error::duplicate_field("patch"));
                            }
                            patch__ = Some(
                                map_.next_value::<std::collections::HashMap<_, ::pbjson::private::NumberDeserialize<f64>>>()?
                                    .into_iter().map(|(k,v)| (k, v.0)).collect()
                            );
                        }
                        GeneratedField::Origin => {
                            if origin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("origin"));
                            }
                            origin__ = Some(map_.next_value::<ArmOrigin>()? as i32);
                        }
                        GeneratedField::Label => {
                            if label__.is_some() {
                                return Err(serde::de::Error::duplicate_field("label"));
                            }
                            label__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ArmRequest {
                    zone_set: zone_set__.unwrap_or_default(),
                    patch: patch__.unwrap_or_default(),
                    origin: origin__.unwrap_or_default(),
                    label: label__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ArmRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ArmedZones {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ArmedZones", len)?;
        if let Some(v) = self.zone_set.as_ref() {
            struct_ser.serialize_field("zone_set", v)?;
        }
        if let Some(v) = self.zone_set_version.as_ref() {
            struct_ser.serialize_field("zone_set_version", v)?;
        }
        if let Some(v) = self.arm_id.as_ref() {
            struct_ser.serialize_field("arm_id", v)?;
        }
        if let Some(v) = self.label.as_ref() {
            struct_ser.serialize_field("label", v)?;
        }
        if true {
            struct_ser.serialize_field("zones", &self.zones)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ArmedZones {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "zone_set",
            "zone_set_version",
            "arm_id",
            "label",
            "zones",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ZoneSet,
            ZoneSetVersion,
            ArmId,
            Label,
            Zones,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "zone_set" => Ok(GeneratedField::ZoneSet),
                            "zone_set_version" => Ok(GeneratedField::ZoneSetVersion),
                            "arm_id" => Ok(GeneratedField::ArmId),
                            "label" => Ok(GeneratedField::Label),
                            "zones" => Ok(GeneratedField::Zones),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ArmedZones;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ArmedZones")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ArmedZones, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut zone_set__ = None;
                let mut zone_set_version__ = None;
                let mut arm_id__ = None;
                let mut label__ = None;
                let mut zones__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ZoneSet => {
                            if zone_set__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zone_set"));
                            }
                            zone_set__ = map_.next_value()?;
                        }
                        GeneratedField::ZoneSetVersion => {
                            if zone_set_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zone_set_version"));
                            }
                            zone_set_version__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::ArmId => {
                            if arm_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("arm_id"));
                            }
                            arm_id__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Label => {
                            if label__.is_some() {
                                return Err(serde::de::Error::duplicate_field("label"));
                            }
                            label__ = map_.next_value()?;
                        }
                        GeneratedField::Zones => {
                            if zones__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zones"));
                            }
                            zones__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ArmedZones {
                    zone_set: zone_set__,
                    zone_set_version: zone_set_version__,
                    arm_id: arm_id__,
                    label: label__,
                    zones: zones__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ArmedZones", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AxisCalibration {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.AxisCalibration", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            struct_ser.serialize_field("counts_per_cm", &self.counts_per_cm)?;
        }
        if let Some(v) = self.counts_per_rev.as_ref() {
            struct_ser.serialize_field("counts_per_rev", v)?;
        }
        if let Some(v) = self.diameter_cm.as_ref() {
            struct_ser.serialize_field("diameter_cm", v)?;
        }
        if true {
            struct_ser.serialize_field("invert", &self.invert)?;
        }
        if let Some(v) = self.measured_at.as_ref() {
            struct_ser.serialize_field("measured_at", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AxisCalibration {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "counts_per_cm",
            "counts_per_rev",
            "diameter_cm",
            "invert",
            "measured_at",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            CountsPerCm,
            CountsPerRev,
            DiameterCm,
            Invert,
            MeasuredAt,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "counts_per_cm" => Ok(GeneratedField::CountsPerCm),
                            "counts_per_rev" => Ok(GeneratedField::CountsPerRev),
                            "diameter_cm" => Ok(GeneratedField::DiameterCm),
                            "invert" => Ok(GeneratedField::Invert),
                            "measured_at" => Ok(GeneratedField::MeasuredAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AxisCalibration;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.AxisCalibration")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AxisCalibration, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut counts_per_cm__ = None;
                let mut counts_per_rev__ = None;
                let mut diameter_cm__ = None;
                let mut invert__ = None;
                let mut measured_at__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CountsPerCm => {
                            if counts_per_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts_per_cm"));
                            }
                            counts_per_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CountsPerRev => {
                            if counts_per_rev__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts_per_rev"));
                            }
                            counts_per_rev__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::DiameterCm => {
                            if diameter_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("diameter_cm"));
                            }
                            diameter_cm__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Invert => {
                            if invert__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invert"));
                            }
                            invert__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MeasuredAt => {
                            if measured_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("measured_at"));
                            }
                            measured_at__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AxisCalibration {
                    name: name__.unwrap_or_default(),
                    counts_per_cm: counts_per_cm__.unwrap_or_default(),
                    counts_per_rev: counts_per_rev__,
                    diameter_cm: diameter_cm__,
                    invert: invert__.unwrap_or_default(),
                    measured_at: measured_at__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.AxisCalibration", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AxisCalibrationPatch {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.AxisCalibrationPatch", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.counts_per_cm.as_ref() {
            struct_ser.serialize_field("counts_per_cm", v)?;
        }
        if let Some(v) = self.counts_per_rev.as_ref() {
            struct_ser.serialize_field("counts_per_rev", v)?;
        }
        if let Some(v) = self.diameter_cm.as_ref() {
            struct_ser.serialize_field("diameter_cm", v)?;
        }
        if let Some(v) = self.invert.as_ref() {
            struct_ser.serialize_field("invert", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AxisCalibrationPatch {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "counts_per_cm",
            "counts_per_rev",
            "diameter_cm",
            "invert",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            CountsPerCm,
            CountsPerRev,
            DiameterCm,
            Invert,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "counts_per_cm" => Ok(GeneratedField::CountsPerCm),
                            "counts_per_rev" => Ok(GeneratedField::CountsPerRev),
                            "diameter_cm" => Ok(GeneratedField::DiameterCm),
                            "invert" => Ok(GeneratedField::Invert),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AxisCalibrationPatch;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.AxisCalibrationPatch")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AxisCalibrationPatch, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut counts_per_cm__ = None;
                let mut counts_per_rev__ = None;
                let mut diameter_cm__ = None;
                let mut invert__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CountsPerCm => {
                            if counts_per_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts_per_cm"));
                            }
                            counts_per_cm__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::CountsPerRev => {
                            if counts_per_rev__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts_per_rev"));
                            }
                            counts_per_rev__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::DiameterCm => {
                            if diameter_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("diameter_cm"));
                            }
                            diameter_cm__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Invert => {
                            if invert__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invert"));
                            }
                            invert__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AxisCalibrationPatch {
                    name: name__.unwrap_or_default(),
                    counts_per_cm: counts_per_cm__,
                    counts_per_rev: counts_per_rev__,
                    diameter_cm: diameter_cm__,
                    invert: invert__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.AxisCalibrationPatch", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AxisState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.AxisState", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("counts", ToString::to_string(&self.counts).as_str())?;
        }
        if true {
            struct_ser.serialize_field("position_cm", &self.position_cm)?;
        }
        if true {
            struct_ser.serialize_field("distance_cm", &self.distance_cm)?;
        }
        if true {
            struct_ser.serialize_field("velocity_cm_s", &self.velocity_cm_s)?;
        }
        if true {
            struct_ser.serialize_field("device_velocity_cm_s", &self.device_velocity_cm_s)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AxisState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "counts",
            "position_cm",
            "distance_cm",
            "velocity_cm_s",
            "device_velocity_cm_s",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Counts,
            PositionCm,
            DistanceCm,
            VelocityCmS,
            DeviceVelocityCmS,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "counts" => Ok(GeneratedField::Counts),
                            "position_cm" => Ok(GeneratedField::PositionCm),
                            "distance_cm" => Ok(GeneratedField::DistanceCm),
                            "velocity_cm_s" => Ok(GeneratedField::VelocityCmS),
                            "device_velocity_cm_s" => Ok(GeneratedField::DeviceVelocityCmS),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AxisState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.AxisState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AxisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut counts__ = None;
                let mut position_cm__ = None;
                let mut distance_cm__ = None;
                let mut velocity_cm_s__ = None;
                let mut device_velocity_cm_s__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Counts => {
                            if counts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts"));
                            }
                            counts__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PositionCm => {
                            if position_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position_cm"));
                            }
                            position_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DistanceCm => {
                            if distance_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("distance_cm"));
                            }
                            distance_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VelocityCmS => {
                            if velocity_cm_s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("velocity_cm_s"));
                            }
                            velocity_cm_s__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DeviceVelocityCmS => {
                            if device_velocity_cm_s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("device_velocity_cm_s"));
                            }
                            device_velocity_cm_s__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(AxisState {
                    name: name__.unwrap_or_default(),
                    counts: counts__.unwrap_or_default(),
                    position_cm: position_cm__.unwrap_or_default(),
                    distance_cm: distance_cm__.unwrap_or_default(),
                    velocity_cm_s: velocity_cm_s__.unwrap_or_default(),
                    device_velocity_cm_s: device_velocity_cm_s__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.AxisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BallCalibration {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.BallCalibration", len)?;
        if true {
            struct_ser.serialize_field("diameter_cm", &self.diameter_cm)?;
        }
        if true {
            struct_ser.serialize_field("sensor_angles_deg", &self.sensor_angles_deg)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BallCalibration {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "diameter_cm",
            "sensor_angles_deg",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DiameterCm,
            SensorAnglesDeg,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "diameter_cm" => Ok(GeneratedField::DiameterCm),
                            "sensor_angles_deg" => Ok(GeneratedField::SensorAnglesDeg),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BallCalibration;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.BallCalibration")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BallCalibration, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut diameter_cm__ = None;
                let mut sensor_angles_deg__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DiameterCm => {
                            if diameter_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("diameter_cm"));
                            }
                            diameter_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SensorAnglesDeg => {
                            if sensor_angles_deg__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sensor_angles_deg"));
                            }
                            sensor_angles_deg__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(BallCalibration {
                    diameter_cm: diameter_cm__.unwrap_or_default(),
                    sensor_angles_deg: sensor_angles_deg__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.BallCalibration", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CalibrationPatch {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.CalibrationPatch", len)?;
        if true {
            struct_ser.serialize_field("axes", &self.axes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CalibrationPatch {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "axes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Axes,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "axes" => Ok(GeneratedField::Axes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CalibrationPatch;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.CalibrationPatch")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CalibrationPatch, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut axes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Axes => {
                            if axes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("axes"));
                            }
                            axes__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CalibrationPatch {
                    axes: axes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.CalibrationPatch", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CalibrationState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.CalibrationState", len)?;
        if true {
            struct_ser.serialize_field("axes", &self.axes)?;
        }
        if let Some(v) = self.ball.as_ref() {
            struct_ser.serialize_field("ball", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CalibrationState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "axes",
            "ball",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Axes,
            Ball,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "axes" => Ok(GeneratedField::Axes),
                            "ball" => Ok(GeneratedField::Ball),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CalibrationState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.CalibrationState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CalibrationState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut axes__ = None;
                let mut ball__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Axes => {
                            if axes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("axes"));
                            }
                            axes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Ball => {
                            if ball__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ball"));
                            }
                            ball__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CalibrationState {
                    axes: axes__.unwrap_or_default(),
                    ball: ball__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.CalibrationState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Capacities {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.Capacities", len)?;
        if true {
            struct_ser.serialize_field("n_axes", &self.n_axes)?;
        }
        if true {
            struct_ser.serialize_field("max_zones", &self.max_zones)?;
        }
        if true {
            struct_ser.serialize_field("max_lines", &self.max_lines)?;
        }
        if true {
            struct_ser.serialize_field("scan_hz", &self.scan_hz)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Capacities {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "n_axes",
            "max_zones",
            "max_lines",
            "scan_hz",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NAxes,
            MaxZones,
            MaxLines,
            ScanHz,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "n_axes" => Ok(GeneratedField::NAxes),
                            "max_zones" => Ok(GeneratedField::MaxZones),
                            "max_lines" => Ok(GeneratedField::MaxLines),
                            "scan_hz" => Ok(GeneratedField::ScanHz),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Capacities;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.Capacities")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Capacities, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut n_axes__ = None;
                let mut max_zones__ = None;
                let mut max_lines__ = None;
                let mut scan_hz__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NAxes => {
                            if n_axes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("n_axes"));
                            }
                            n_axes__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxZones => {
                            if max_zones__.is_some() {
                                return Err(serde::de::Error::duplicate_field("max_zones"));
                            }
                            max_zones__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxLines => {
                            if max_lines__.is_some() {
                                return Err(serde::de::Error::duplicate_field("max_lines"));
                            }
                            max_lines__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ScanHz => {
                            if scan_hz__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scan_hz"));
                            }
                            scan_hz__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Capacities {
                    n_axes: n_axes__.unwrap_or_default(),
                    max_zones: max_zones__.unwrap_or_default(),
                    max_lines: max_lines__.unwrap_or_default(),
                    scan_hz: scan_hz__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.Capacities", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConfigPatch {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ConfigPatch", len)?;
        if let Some(v) = self.rate_hz.as_ref() {
            struct_ser.serialize_field("rate_hz", v)?;
        }
        if let Some(v) = self.display_hz.as_ref() {
            struct_ser.serialize_field("display_hz", v)?;
        }
        if let Some(v) = self.ring_minutes.as_ref() {
            struct_ser.serialize_field("ring_minutes", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConfigPatch {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rate_hz",
            "display_hz",
            "ring_minutes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RateHz,
            DisplayHz,
            RingMinutes,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rate_hz" => Ok(GeneratedField::RateHz),
                            "display_hz" => Ok(GeneratedField::DisplayHz),
                            "ring_minutes" => Ok(GeneratedField::RingMinutes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConfigPatch;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ConfigPatch")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ConfigPatch, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rate_hz__ = None;
                let mut display_hz__ = None;
                let mut ring_minutes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RateHz => {
                            if rate_hz__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rate_hz"));
                            }
                            rate_hz__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::DisplayHz => {
                            if display_hz__.is_some() {
                                return Err(serde::de::Error::duplicate_field("display_hz"));
                            }
                            display_hz__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::RingMinutes => {
                            if ring_minutes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ring_minutes"));
                            }
                            ring_minutes__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(ConfigPatch {
                    rate_hz: rate_hz__,
                    display_hz: display_hz__,
                    ring_minutes: ring_minutes__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ConfigPatch", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConfigView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ConfigView", len)?;
        if true {
            struct_ser.serialize_field("rate_hz", &self.rate_hz)?;
        }
        if true {
            struct_ser.serialize_field("display_hz", &self.display_hz)?;
        }
        if true {
            struct_ser.serialize_field("ring_minutes", &self.ring_minutes)?;
        }
        if true {
            struct_ser.serialize_field("shm_name", &self.shm_name)?;
        }
        if true {
            struct_ser.serialize_field("shm_open", &self.shm_open)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("shm_writes", ToString::to_string(&self.shm_writes).as_str())?;
        }
        if true {
            struct_ser.serialize_field("event_port", &self.event_port)?;
        }
        if true {
            struct_ser.serialize_field("port", &self.port)?;
        }
        if true {
            struct_ser.serialize_field("starves_the_display", &self.starves_the_display)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConfigView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rate_hz",
            "display_hz",
            "ring_minutes",
            "shm_name",
            "shm_open",
            "shm_writes",
            "event_port",
            "port",
            "starves_the_display",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RateHz,
            DisplayHz,
            RingMinutes,
            ShmName,
            ShmOpen,
            ShmWrites,
            EventPort,
            Port,
            StarvesTheDisplay,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rate_hz" => Ok(GeneratedField::RateHz),
                            "display_hz" => Ok(GeneratedField::DisplayHz),
                            "ring_minutes" => Ok(GeneratedField::RingMinutes),
                            "shm_name" => Ok(GeneratedField::ShmName),
                            "shm_open" => Ok(GeneratedField::ShmOpen),
                            "shm_writes" => Ok(GeneratedField::ShmWrites),
                            "event_port" => Ok(GeneratedField::EventPort),
                            "port" => Ok(GeneratedField::Port),
                            "starves_the_display" => Ok(GeneratedField::StarvesTheDisplay),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConfigView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ConfigView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ConfigView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rate_hz__ = None;
                let mut display_hz__ = None;
                let mut ring_minutes__ = None;
                let mut shm_name__ = None;
                let mut shm_open__ = None;
                let mut shm_writes__ = None;
                let mut event_port__ = None;
                let mut port__ = None;
                let mut starves_the_display__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RateHz => {
                            if rate_hz__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rate_hz"));
                            }
                            rate_hz__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DisplayHz => {
                            if display_hz__.is_some() {
                                return Err(serde::de::Error::duplicate_field("display_hz"));
                            }
                            display_hz__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RingMinutes => {
                            if ring_minutes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ring_minutes"));
                            }
                            ring_minutes__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ShmName => {
                            if shm_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shm_name"));
                            }
                            shm_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ShmOpen => {
                            if shm_open__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shm_open"));
                            }
                            shm_open__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ShmWrites => {
                            if shm_writes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shm_writes"));
                            }
                            shm_writes__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EventPort => {
                            if event_port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("event_port"));
                            }
                            event_port__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Port => {
                            if port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("port"));
                            }
                            port__ = Some(map_.next_value()?);
                        }
                        GeneratedField::StarvesTheDisplay => {
                            if starves_the_display__.is_some() {
                                return Err(serde::de::Error::duplicate_field("starves_the_display"));
                            }
                            starves_the_display__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ConfigView {
                    rate_hz: rate_hz__.unwrap_or_default(),
                    display_hz: display_hz__.unwrap_or_default(),
                    ring_minutes: ring_minutes__.unwrap_or_default(),
                    shm_name: shm_name__.unwrap_or_default(),
                    shm_open: shm_open__.unwrap_or_default(),
                    shm_writes: shm_writes__.unwrap_or_default(),
                    event_port: event_port__.unwrap_or_default(),
                    port: port__.unwrap_or_default(),
                    starves_the_display: starves_the_display__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ConfigView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeviceInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.DeviceInfo", len)?;
        if true {
            struct_ser.serialize_field("connected", &self.connected)?;
        }
        if true {
            struct_ser.serialize_field("port", &self.port)?;
        }
        if true {
            struct_ser.serialize_field("board", &self.board)?;
        }
        if true {
            struct_ser.serialize_field("firmware_version", &self.firmware_version)?;
        }
        if true {
            struct_ser.serialize_field("protocol_version", &self.protocol_version)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("uptime_device_us", ToString::to_string(&self.uptime_device_us).as_str())?;
        }
        if let Some(v) = self.capacities.as_ref() {
            struct_ser.serialize_field("capacities", v)?;
        }
        if let Some(v) = self.flashed_zone_set.as_ref() {
            struct_ser.serialize_field("flashed_zone_set", v)?;
        }
        if let Some(v) = self.link.as_ref() {
            struct_ser.serialize_field("link", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeviceInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connected",
            "port",
            "board",
            "firmware_version",
            "protocol_version",
            "uptime_device_us",
            "capacities",
            "flashed_zone_set",
            "link",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Connected,
            Port,
            Board,
            FirmwareVersion,
            ProtocolVersion,
            UptimeDeviceUs,
            Capacities,
            FlashedZoneSet,
            Link,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "connected" => Ok(GeneratedField::Connected),
                            "port" => Ok(GeneratedField::Port),
                            "board" => Ok(GeneratedField::Board),
                            "firmware_version" => Ok(GeneratedField::FirmwareVersion),
                            "protocol_version" => Ok(GeneratedField::ProtocolVersion),
                            "uptime_device_us" => Ok(GeneratedField::UptimeDeviceUs),
                            "capacities" => Ok(GeneratedField::Capacities),
                            "flashed_zone_set" => Ok(GeneratedField::FlashedZoneSet),
                            "link" => Ok(GeneratedField::Link),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeviceInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.DeviceInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeviceInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connected__ = None;
                let mut port__ = None;
                let mut board__ = None;
                let mut firmware_version__ = None;
                let mut protocol_version__ = None;
                let mut uptime_device_us__ = None;
                let mut capacities__ = None;
                let mut flashed_zone_set__ = None;
                let mut link__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Connected => {
                            if connected__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connected"));
                            }
                            connected__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Port => {
                            if port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("port"));
                            }
                            port__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Board => {
                            if board__.is_some() {
                                return Err(serde::de::Error::duplicate_field("board"));
                            }
                            board__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FirmwareVersion => {
                            if firmware_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("firmware_version"));
                            }
                            firmware_version__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProtocolVersion => {
                            if protocol_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("protocol_version"));
                            }
                            protocol_version__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UptimeDeviceUs => {
                            if uptime_device_us__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uptime_device_us"));
                            }
                            uptime_device_us__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Capacities => {
                            if capacities__.is_some() {
                                return Err(serde::de::Error::duplicate_field("capacities"));
                            }
                            capacities__ = map_.next_value()?;
                        }
                        GeneratedField::FlashedZoneSet => {
                            if flashed_zone_set__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flashed_zone_set"));
                            }
                            flashed_zone_set__ = map_.next_value()?;
                        }
                        GeneratedField::Link => {
                            if link__.is_some() {
                                return Err(serde::de::Error::duplicate_field("link"));
                            }
                            link__ = map_.next_value()?;
                        }
                    }
                }
                Ok(DeviceInfo {
                    connected: connected__.unwrap_or_default(),
                    port: port__.unwrap_or_default(),
                    board: board__.unwrap_or_default(),
                    firmware_version: firmware_version__.unwrap_or_default(),
                    protocol_version: protocol_version__.unwrap_or_default(),
                    uptime_device_us: uptime_device_us__.unwrap_or_default(),
                    capacities: capacities__,
                    flashed_zone_set: flashed_zone_set__,
                    link: link__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.DeviceInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeviceProtocol {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.DeviceProtocol", len)?;
        if true {
            struct_ser.serialize_field("speaks", &self.speaks)?;
        }
        if true {
            struct_ser.serialize_field("floor", &self.floor)?;
        }
        if let Some(v) = self.board.as_ref() {
            struct_ser.serialize_field("board", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeviceProtocol {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "speaks",
            "floor",
            "board",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Speaks,
            Floor,
            Board,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "speaks" => Ok(GeneratedField::Speaks),
                            "floor" => Ok(GeneratedField::Floor),
                            "board" => Ok(GeneratedField::Board),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeviceProtocol;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.DeviceProtocol")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeviceProtocol, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut speaks__ = None;
                let mut floor__ = None;
                let mut board__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Speaks => {
                            if speaks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("speaks"));
                            }
                            speaks__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Floor => {
                            if floor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("floor"));
                            }
                            floor__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Board => {
                            if board__.is_some() {
                                return Err(serde::de::Error::duplicate_field("board"));
                            }
                            board__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(DeviceProtocol {
                    speaks: speaks__.unwrap_or_default(),
                    floor: floor__.unwrap_or_default(),
                    board: board__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.DeviceProtocol", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Error {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.Error", len)?;
        if true {
            struct_ser.serialize_field("error", &self.error)?;
        }
        if true {
            struct_ser.serialize_field("detail", &self.detail)?;
        }
        if true {
            struct_ser.serialize_field("context", &self.context)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Error {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "error",
            "detail",
            "context",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Error,
            Detail,
            Context,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "error" => Ok(GeneratedField::Error),
                            "detail" => Ok(GeneratedField::Detail),
                            "context" => Ok(GeneratedField::Context),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Error;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.Error")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Error, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut error__ = None;
                let mut detail__ = None;
                let mut context__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Error => {
                            if error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Detail => {
                            if detail__.is_some() {
                                return Err(serde::de::Error::duplicate_field("detail"));
                            }
                            detail__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Context => {
                            if context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("context"));
                            }
                            context__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Error {
                    error: error__.unwrap_or_default(),
                    detail: detail__.unwrap_or_default(),
                    context: context__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.Error", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FireRule {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::FireRuleUnspecified => "FIRE_RULE_UNSPECIFIED",
            Self::FireRuleOnce => "FIRE_RULE_ONCE",
            Self::FireRuleRearm => "FIRE_RULE_REARM",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for FireRule {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "FIRE_RULE_UNSPECIFIED",
            "FIRE_RULE_ONCE",
            "FIRE_RULE_REARM",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = FireRule;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "FIRE_RULE_UNSPECIFIED" => Ok(FireRule::FireRuleUnspecified),
                    "FIRE_RULE_ONCE" => Ok(FireRule::FireRuleOnce),
                    "FIRE_RULE_REARM" => Ok(FireRule::FireRuleRearm),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for FirmwareVersions {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.FirmwareVersions", len)?;
        if true {
            struct_ser.serialize_field("running", &self.running)?;
        }
        if true {
            struct_ser.serialize_field("available", &self.available)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FirmwareVersions {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "running",
            "available",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Running,
            Available,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "running" => Ok(GeneratedField::Running),
                            "available" => Ok(GeneratedField::Available),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FirmwareVersions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.FirmwareVersions")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FirmwareVersions, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut running__ = None;
                let mut available__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Running => {
                            if running__.is_some() {
                                return Err(serde::de::Error::duplicate_field("running"));
                            }
                            running__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Available => {
                            if available__.is_some() {
                                return Err(serde::de::Error::duplicate_field("available"));
                            }
                            available__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(FirmwareVersions {
                    running: running__.unwrap_or_default(),
                    available: available__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.FirmwareVersions", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FlashedZoneSet {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.FlashedZoneSet", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            struct_ser.serialize_field("version", &self.version)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FlashedZoneSet {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "version",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Version,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "version" => Ok(GeneratedField::Version),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FlashedZoneSet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.FlashedZoneSet")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FlashedZoneSet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut version__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(FlashedZoneSet {
                    name: name__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.FlashedZoneSet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LineMap {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.LineMap", len)?;
        if true {
            struct_ser.serialize_field("lines", &self.lines)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LineMap {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "lines",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Lines,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "lines" => Ok(GeneratedField::Lines),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LineMap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.LineMap")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LineMap, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut lines__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Lines => {
                            if lines__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lines"));
                            }
                            lines__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(LineMap {
                    lines: lines__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.LineMap", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LinkHealth {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.LinkHealth", len)?;
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("seq_gaps", ToString::to_string(&self.seq_gaps).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("ring_drops", ToString::to_string(&self.ring_drops).as_str())?;
        }
        if true {
            struct_ser.serialize_field("measured_rate_hz", &self.measured_rate_hz)?;
        }
        if true {
            struct_ser.serialize_field("stale", &self.stale)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LinkHealth {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "seq_gaps",
            "ring_drops",
            "measured_rate_hz",
            "stale",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SeqGaps,
            RingDrops,
            MeasuredRateHz,
            Stale,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "seq_gaps" => Ok(GeneratedField::SeqGaps),
                            "ring_drops" => Ok(GeneratedField::RingDrops),
                            "measured_rate_hz" => Ok(GeneratedField::MeasuredRateHz),
                            "stale" => Ok(GeneratedField::Stale),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LinkHealth;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.LinkHealth")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LinkHealth, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut seq_gaps__ = None;
                let mut ring_drops__ = None;
                let mut measured_rate_hz__ = None;
                let mut stale__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SeqGaps => {
                            if seq_gaps__.is_some() {
                                return Err(serde::de::Error::duplicate_field("seq_gaps"));
                            }
                            seq_gaps__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RingDrops => {
                            if ring_drops__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ring_drops"));
                            }
                            ring_drops__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MeasuredRateHz => {
                            if measured_rate_hz__.is_some() {
                                return Err(serde::de::Error::duplicate_field("measured_rate_hz"));
                            }
                            measured_rate_hz__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Stale => {
                            if stale__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stale"));
                            }
                            stale__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(LinkHealth {
                    seq_gaps: seq_gaps__.unwrap_or_default(),
                    ring_drops: ring_drops__.unwrap_or_default(),
                    measured_rate_hz: measured_rate_hz__.unwrap_or_default(),
                    stale: stale__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.LinkHealth", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LinkState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.LinkState", len)?;
        if true {
            struct_ser.serialize_field("connected", &self.connected)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LinkState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connected",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Connected,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "connected" => Ok(GeneratedField::Connected),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LinkState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.LinkState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LinkState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connected__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Connected => {
                            if connected__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connected"));
                            }
                            connected__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(LinkState {
                    connected: connected__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.LinkState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LinkStats {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.LinkStats", len)?;
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("connection_count", ToString::to_string(&self.connection_count).as_str())?;
        }
        if let Some(v) = self.last_error.as_ref() {
            struct_ser.serialize_field("last_error", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LinkStats {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection_count",
            "last_error",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConnectionCount,
            LastError,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "connection_count" => Ok(GeneratedField::ConnectionCount),
                            "last_error" => Ok(GeneratedField::LastError),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LinkStats;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.LinkStats")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LinkStats, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection_count__ = None;
                let mut last_error__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ConnectionCount => {
                            if connection_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connection_count"));
                            }
                            connection_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LastError => {
                            if last_error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("last_error"));
                            }
                            last_error__ = map_.next_value()?;
                        }
                    }
                }
                Ok(LinkStats {
                    connection_count: connection_count__.unwrap_or_default(),
                    last_error: last_error__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.LinkStats", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MeasurementApplied {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.MeasurementApplied", len)?;
        if true {
            struct_ser.serialize_field("axis", &self.axis)?;
        }
        if true {
            struct_ser.serialize_field("counts_per_cm", &self.counts_per_cm)?;
        }
        if true {
            struct_ser.serialize_field("zone_sets_invalidated", &self.zone_sets_invalidated)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MeasurementApplied {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "axis",
            "counts_per_cm",
            "zone_sets_invalidated",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Axis,
            CountsPerCm,
            ZoneSetsInvalidated,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "axis" => Ok(GeneratedField::Axis),
                            "counts_per_cm" => Ok(GeneratedField::CountsPerCm),
                            "zone_sets_invalidated" => Ok(GeneratedField::ZoneSetsInvalidated),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MeasurementApplied;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.MeasurementApplied")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MeasurementApplied, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut axis__ = None;
                let mut counts_per_cm__ = None;
                let mut zone_sets_invalidated__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Axis => {
                            if axis__.is_some() {
                                return Err(serde::de::Error::duplicate_field("axis"));
                            }
                            axis__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CountsPerCm => {
                            if counts_per_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts_per_cm"));
                            }
                            counts_per_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ZoneSetsInvalidated => {
                            if zone_sets_invalidated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zone_sets_invalidated"));
                            }
                            zone_sets_invalidated__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(MeasurementApplied {
                    axis: axis__.unwrap_or_default(),
                    counts_per_cm: counts_per_cm__.unwrap_or_default(),
                    zone_sets_invalidated: zone_sets_invalidated__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.MeasurementApplied", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MeasurementResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.MeasurementResult", len)?;
        if true {
            struct_ser.serialize_field("axis", &self.axis)?;
        }
        if true {
            struct_ser.serialize_field("known_distance_cm", &self.known_distance_cm)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("counts", ToString::to_string(&self.counts).as_str())?;
        }
        if true {
            struct_ser.serialize_field("measured_counts_per_cm", &self.measured_counts_per_cm)?;
        }
        if true {
            struct_ser.serialize_field("configured_counts_per_cm", &self.configured_counts_per_cm)?;
        }
        if let Some(v) = self.counts_per_rev.as_ref() {
            struct_ser.serialize_field("counts_per_rev", v)?;
        }
        if let Some(v) = self.nominal_counts_per_cm.as_ref() {
            struct_ser.serialize_field("nominal_counts_per_cm", v)?;
        }
        if let Some(v) = self.implied_circumference_cm.as_ref() {
            struct_ser.serialize_field("implied_circumference_cm", v)?;
        }
        if let Some(v) = self.decoding_suspect.as_ref() {
            struct_ser.serialize_field("decoding_suspect", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MeasurementResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "axis",
            "known_distance_cm",
            "counts",
            "measured_counts_per_cm",
            "configured_counts_per_cm",
            "counts_per_rev",
            "nominal_counts_per_cm",
            "implied_circumference_cm",
            "decoding_suspect",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Axis,
            KnownDistanceCm,
            Counts,
            MeasuredCountsPerCm,
            ConfiguredCountsPerCm,
            CountsPerRev,
            NominalCountsPerCm,
            ImpliedCircumferenceCm,
            DecodingSuspect,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "axis" => Ok(GeneratedField::Axis),
                            "known_distance_cm" => Ok(GeneratedField::KnownDistanceCm),
                            "counts" => Ok(GeneratedField::Counts),
                            "measured_counts_per_cm" => Ok(GeneratedField::MeasuredCountsPerCm),
                            "configured_counts_per_cm" => Ok(GeneratedField::ConfiguredCountsPerCm),
                            "counts_per_rev" => Ok(GeneratedField::CountsPerRev),
                            "nominal_counts_per_cm" => Ok(GeneratedField::NominalCountsPerCm),
                            "implied_circumference_cm" => Ok(GeneratedField::ImpliedCircumferenceCm),
                            "decoding_suspect" => Ok(GeneratedField::DecodingSuspect),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MeasurementResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.MeasurementResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MeasurementResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut axis__ = None;
                let mut known_distance_cm__ = None;
                let mut counts__ = None;
                let mut measured_counts_per_cm__ = None;
                let mut configured_counts_per_cm__ = None;
                let mut counts_per_rev__ = None;
                let mut nominal_counts_per_cm__ = None;
                let mut implied_circumference_cm__ = None;
                let mut decoding_suspect__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Axis => {
                            if axis__.is_some() {
                                return Err(serde::de::Error::duplicate_field("axis"));
                            }
                            axis__ = Some(map_.next_value()?);
                        }
                        GeneratedField::KnownDistanceCm => {
                            if known_distance_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("known_distance_cm"));
                            }
                            known_distance_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Counts => {
                            if counts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts"));
                            }
                            counts__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MeasuredCountsPerCm => {
                            if measured_counts_per_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("measured_counts_per_cm"));
                            }
                            measured_counts_per_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ConfiguredCountsPerCm => {
                            if configured_counts_per_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("configured_counts_per_cm"));
                            }
                            configured_counts_per_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CountsPerRev => {
                            if counts_per_rev__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts_per_rev"));
                            }
                            counts_per_rev__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::NominalCountsPerCm => {
                            if nominal_counts_per_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nominal_counts_per_cm"));
                            }
                            nominal_counts_per_cm__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::ImpliedCircumferenceCm => {
                            if implied_circumference_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("implied_circumference_cm"));
                            }
                            implied_circumference_cm__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::DecodingSuspect => {
                            if decoding_suspect__.is_some() {
                                return Err(serde::de::Error::duplicate_field("decoding_suspect"));
                            }
                            decoding_suspect__ = map_.next_value()?;
                        }
                    }
                }
                Ok(MeasurementResult {
                    axis: axis__.unwrap_or_default(),
                    known_distance_cm: known_distance_cm__.unwrap_or_default(),
                    counts: counts__.unwrap_or_default(),
                    measured_counts_per_cm: measured_counts_per_cm__.unwrap_or_default(),
                    configured_counts_per_cm: configured_counts_per_cm__.unwrap_or_default(),
                    counts_per_rev: counts_per_rev__,
                    nominal_counts_per_cm: nominal_counts_per_cm__,
                    implied_circumference_cm: implied_circumference_cm__,
                    decoding_suspect: decoding_suspect__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.MeasurementResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MeasurementStarted {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.MeasurementStarted", len)?;
        if true {
            struct_ser.serialize_field("axis", &self.axis)?;
        }
        if true {
            struct_ser.serialize_field("known_distance_cm", &self.known_distance_cm)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("counts_at_start", ToString::to_string(&self.counts_at_start).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("counts", ToString::to_string(&self.counts).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MeasurementStarted {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "axis",
            "known_distance_cm",
            "counts_at_start",
            "counts",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Axis,
            KnownDistanceCm,
            CountsAtStart,
            Counts,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "axis" => Ok(GeneratedField::Axis),
                            "known_distance_cm" => Ok(GeneratedField::KnownDistanceCm),
                            "counts_at_start" => Ok(GeneratedField::CountsAtStart),
                            "counts" => Ok(GeneratedField::Counts),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MeasurementStarted;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.MeasurementStarted")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MeasurementStarted, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut axis__ = None;
                let mut known_distance_cm__ = None;
                let mut counts_at_start__ = None;
                let mut counts__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Axis => {
                            if axis__.is_some() {
                                return Err(serde::de::Error::duplicate_field("axis"));
                            }
                            axis__ = Some(map_.next_value()?);
                        }
                        GeneratedField::KnownDistanceCm => {
                            if known_distance_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("known_distance_cm"));
                            }
                            known_distance_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CountsAtStart => {
                            if counts_at_start__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts_at_start"));
                            }
                            counts_at_start__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Counts => {
                            if counts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts"));
                            }
                            counts__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MeasurementStarted {
                    axis: axis__.unwrap_or_default(),
                    known_distance_cm: known_distance_cm__.unwrap_or_default(),
                    counts_at_start: counts_at_start__.unwrap_or_default(),
                    counts: counts__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.MeasurementStarted", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputAction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::OutputActionUnspecified => "OUTPUT_ACTION_UNSPECIFIED",
            Self::OutputActionPulse => "OUTPUT_ACTION_PULSE",
            Self::OutputActionLevel => "OUTPUT_ACTION_LEVEL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for OutputAction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "OUTPUT_ACTION_UNSPECIFIED",
            "OUTPUT_ACTION_PULSE",
            "OUTPUT_ACTION_LEVEL",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = OutputAction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "OUTPUT_ACTION_UNSPECIFIED" => Ok(OutputAction::OutputActionUnspecified),
                    "OUTPUT_ACTION_PULSE" => Ok(OutputAction::OutputActionPulse),
                    "OUTPUT_ACTION_LEVEL" => Ok(OutputAction::OutputActionLevel),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for OutputLine {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.OutputLine", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            struct_ser.serialize_field("index", &self.index)?;
        }
        if true {
            struct_ser.serialize_field("pin", &self.pin)?;
        }
        if true {
            struct_ser.serialize_field("safe_high", &self.safe_high)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputLine {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "index",
            "pin",
            "safe_high",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Index,
            Pin,
            SafeHigh,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "index" => Ok(GeneratedField::Index),
                            "pin" => Ok(GeneratedField::Pin),
                            "safe_high" => Ok(GeneratedField::SafeHigh),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputLine;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.OutputLine")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OutputLine, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut index__ = None;
                let mut pin__ = None;
                let mut safe_high__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Pin => {
                            if pin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pin"));
                            }
                            pin__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SafeHigh => {
                            if safe_high__.is_some() {
                                return Err(serde::de::Error::duplicate_field("safe_high"));
                            }
                            safe_high__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(OutputLine {
                    name: name__.unwrap_or_default(),
                    index: index__.unwrap_or_default(),
                    pin: pin__.unwrap_or_default(),
                    safe_high: safe_high__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.OutputLine", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolvedBound {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ResolvedBound", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolvedBound {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolvedBound;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ResolvedBound")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ResolvedBound, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(ResolvedBound {
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ResolvedBound", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RigState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.RigState", len)?;
        if true {
            struct_ser.serialize_field("axes", &self.axes)?;
        }
        if let Some(v) = self.health.as_ref() {
            struct_ser.serialize_field("health", v)?;
        }
        if let Some(v) = self.link.as_ref() {
            struct_ser.serialize_field("link", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RigState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "axes",
            "health",
            "link",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Axes,
            Health,
            Link,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "axes" => Ok(GeneratedField::Axes),
                            "health" => Ok(GeneratedField::Health),
                            "link" => Ok(GeneratedField::Link),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RigState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.RigState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RigState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut axes__ = None;
                let mut health__ = None;
                let mut link__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Axes => {
                            if axes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("axes"));
                            }
                            axes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Health => {
                            if health__.is_some() {
                                return Err(serde::de::Error::duplicate_field("health"));
                            }
                            health__ = map_.next_value()?;
                        }
                        GeneratedField::Link => {
                            if link__.is_some() {
                                return Err(serde::de::Error::duplicate_field("link"));
                            }
                            link__ = map_.next_value()?;
                        }
                    }
                }
                Ok(RigState {
                    axes: axes__.unwrap_or_default(),
                    health: health__,
                    link: link__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.RigState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Sample {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.Sample", len)?;
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("seq", ToString::to_string(&self.seq).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("device_us", ToString::to_string(&self.device_us).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("host_monotonic_ns", ToString::to_string(&self.host_monotonic_ns).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("lost_before", ToString::to_string(&self.lost_before).as_str())?;
        }
        if true {
            struct_ser.serialize_field("axes", &self.axes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Sample {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "seq",
            "device_us",
            "host_monotonic_ns",
            "lost_before",
            "axes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Seq,
            DeviceUs,
            HostMonotonicNs,
            LostBefore,
            Axes,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "seq" => Ok(GeneratedField::Seq),
                            "device_us" => Ok(GeneratedField::DeviceUs),
                            "host_monotonic_ns" => Ok(GeneratedField::HostMonotonicNs),
                            "lost_before" => Ok(GeneratedField::LostBefore),
                            "axes" => Ok(GeneratedField::Axes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Sample;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.Sample")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Sample, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut seq__ = None;
                let mut device_us__ = None;
                let mut host_monotonic_ns__ = None;
                let mut lost_before__ = None;
                let mut axes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Seq => {
                            if seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("seq"));
                            }
                            seq__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DeviceUs => {
                            if device_us__.is_some() {
                                return Err(serde::de::Error::duplicate_field("device_us"));
                            }
                            device_us__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::HostMonotonicNs => {
                            if host_monotonic_ns__.is_some() {
                                return Err(serde::de::Error::duplicate_field("host_monotonic_ns"));
                            }
                            host_monotonic_ns__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LostBefore => {
                            if lost_before__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lost_before"));
                            }
                            lost_before__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Axes => {
                            if axes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("axes"));
                            }
                            axes__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Sample {
                    seq: seq__.unwrap_or_default(),
                    device_us: device_us__.unwrap_or_default(),
                    host_monotonic_ns: host_monotonic_ns__.unwrap_or_default(),
                    lost_before: lost_before__.unwrap_or_default(),
                    axes: axes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.Sample", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StartMeasurement {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.StartMeasurement", len)?;
        if true {
            struct_ser.serialize_field("axis", &self.axis)?;
        }
        if true {
            struct_ser.serialize_field("known_distance_cm", &self.known_distance_cm)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StartMeasurement {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "axis",
            "known_distance_cm",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Axis,
            KnownDistanceCm,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "axis" => Ok(GeneratedField::Axis),
                            "known_distance_cm" => Ok(GeneratedField::KnownDistanceCm),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StartMeasurement;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.StartMeasurement")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<StartMeasurement, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut axis__ = None;
                let mut known_distance_cm__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Axis => {
                            if axis__.is_some() {
                                return Err(serde::de::Error::duplicate_field("axis"));
                            }
                            axis__ = Some(map_.next_value()?);
                        }
                        GeneratedField::KnownDistanceCm => {
                            if known_distance_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("known_distance_cm"));
                            }
                            known_distance_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(StartMeasurement {
                    axis: axis__.unwrap_or_default(),
                    known_distance_cm: known_distance_cm__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.StartMeasurement", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StreamFrame {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.frame.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.StreamFrame", len)?;
        if let Some(v) = self.frame.as_ref() {
            match v {
                stream_frame::Frame::Sample(v) => {
                    struct_ser.serialize_field("sample", v)?;
                }
                stream_frame::Frame::ZoneHit(v) => {
                    struct_ser.serialize_field("zone_hit", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StreamFrame {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sample",
            "zone_hit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sample,
            ZoneHit,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "sample" => Ok(GeneratedField::Sample),
                            "zone_hit" => Ok(GeneratedField::ZoneHit),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StreamFrame;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.StreamFrame")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<StreamFrame, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut frame__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Sample => {
                            if frame__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sample"));
                            }
                            frame__ = map_.next_value::<::std::option::Option<_>>()?.map(stream_frame::Frame::Sample)
;
                        }
                        GeneratedField::ZoneHit => {
                            if frame__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zone_hit"));
                            }
                            frame__ = map_.next_value::<::std::option::Option<_>>()?.map(stream_frame::Frame::ZoneHit)
;
                        }
                    }
                }
                Ok(StreamFrame {
                    frame: frame__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.StreamFrame", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidationReport {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ValidationReport", len)?;
        if true {
            struct_ser.serialize_field("ok", &self.ok)?;
        }
        if let Some(v) = self.problem.as_ref() {
            struct_ser.serialize_field("problem", v)?;
        }
        if true {
            struct_ser.serialize_field("zone_count", &self.zone_count)?;
        }
        if true {
            struct_ser.serialize_field("counts_per_cm", &self.counts_per_cm)?;
        }
        if true {
            struct_ser.serialize_field("references", &self.references)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidationReport {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ok",
            "problem",
            "zone_count",
            "counts_per_cm",
            "references",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Ok,
            Problem,
            ZoneCount,
            CountsPerCm,
            References,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "ok" => Ok(GeneratedField::Ok),
                            "problem" => Ok(GeneratedField::Problem),
                            "zone_count" => Ok(GeneratedField::ZoneCount),
                            "counts_per_cm" => Ok(GeneratedField::CountsPerCm),
                            "references" => Ok(GeneratedField::References),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidationReport;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ValidationReport")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidationReport, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ok__ = None;
                let mut problem__ = None;
                let mut zone_count__ = None;
                let mut counts_per_cm__ = None;
                let mut references__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Ok => {
                            if ok__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ok"));
                            }
                            ok__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Problem => {
                            if problem__.is_some() {
                                return Err(serde::de::Error::duplicate_field("problem"));
                            }
                            problem__ = map_.next_value()?;
                        }
                        GeneratedField::ZoneCount => {
                            if zone_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zone_count"));
                            }
                            zone_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CountsPerCm => {
                            if counts_per_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counts_per_cm"));
                            }
                            counts_per_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::References => {
                            if references__.is_some() {
                                return Err(serde::de::Error::duplicate_field("references"));
                            }
                            references__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ValidationReport {
                    ok: ok__.unwrap_or_default(),
                    problem: problem__,
                    zone_count: zone_count__.unwrap_or_default(),
                    counts_per_cm: counts_per_cm__.unwrap_or_default(),
                    references: references__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ValidationReport", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VersionReport {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.VersionReport", len)?;
        if true {
            struct_ser.serialize_field("daemon", &self.daemon)?;
        }
        if true {
            struct_ser.serialize_field("api", &self.api)?;
        }
        if let Some(v) = self.device_protocol.as_ref() {
            struct_ser.serialize_field("device_protocol", v)?;
        }
        if true {
            struct_ser.serialize_field("vinput_layout", &self.vinput_layout)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VersionReport {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "daemon",
            "api",
            "device_protocol",
            "vinput_layout",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Daemon,
            Api,
            DeviceProtocol,
            VinputLayout,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "daemon" => Ok(GeneratedField::Daemon),
                            "api" => Ok(GeneratedField::Api),
                            "device_protocol" => Ok(GeneratedField::DeviceProtocol),
                            "vinput_layout" => Ok(GeneratedField::VinputLayout),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VersionReport;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.VersionReport")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<VersionReport, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut daemon__ = None;
                let mut api__ = None;
                let mut device_protocol__ = None;
                let mut vinput_layout__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Daemon => {
                            if daemon__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daemon"));
                            }
                            daemon__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Api => {
                            if api__.is_some() {
                                return Err(serde::de::Error::duplicate_field("api"));
                            }
                            api__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DeviceProtocol => {
                            if device_protocol__.is_some() {
                                return Err(serde::de::Error::duplicate_field("device_protocol"));
                            }
                            device_protocol__ = map_.next_value()?;
                        }
                        GeneratedField::VinputLayout => {
                            if vinput_layout__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vinput_layout"));
                            }
                            vinput_layout__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(VersionReport {
                    daemon: daemon__.unwrap_or_default(),
                    api: api__.unwrap_or_default(),
                    device_protocol: device_protocol__,
                    vinput_layout: vinput_layout__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.VersionReport", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WatchStateRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.WatchStateRequest", len)?;
        if true {
            struct_ser.serialize_field("rate_hz", &self.rate_hz)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WatchStateRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rate_hz",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RateHz,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rate_hz" => Ok(GeneratedField::RateHz),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WatchStateRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.WatchStateRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WatchStateRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rate_hz__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RateHz => {
                            if rate_hz__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rate_hz"));
                            }
                            rate_hz__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(WatchStateRequest {
                    rate_hz: rate_hz__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.WatchStateRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WireDirection {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::WireDirectionUnspecified => "WIRE_DIRECTION_UNSPECIFIED",
            Self::WireDirectionOut => "WIRE_DIRECTION_OUT",
            Self::WireDirectionIn => "WIRE_DIRECTION_IN",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for WireDirection {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "WIRE_DIRECTION_UNSPECIFIED",
            "WIRE_DIRECTION_OUT",
            "WIRE_DIRECTION_IN",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = WireDirection;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "WIRE_DIRECTION_UNSPECIFIED" => Ok(WireDirection::WireDirectionUnspecified),
                    "WIRE_DIRECTION_OUT" => Ok(WireDirection::WireDirectionOut),
                    "WIRE_DIRECTION_IN" => Ok(WireDirection::WireDirectionIn),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for WireLevel {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::WireLevelUnspecified => "WIRE_LEVEL_UNSPECIFIED",
            Self::WireLevelInfo => "WIRE_LEVEL_INFO",
            Self::WireLevelError => "WIRE_LEVEL_ERROR",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for WireLevel {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "WIRE_LEVEL_UNSPECIFIED",
            "WIRE_LEVEL_INFO",
            "WIRE_LEVEL_ERROR",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = WireLevel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "WIRE_LEVEL_UNSPECIFIED" => Ok(WireLevel::WireLevelUnspecified),
                    "WIRE_LEVEL_INFO" => Ok(WireLevel::WireLevelInfo),
                    "WIRE_LEVEL_ERROR" => Ok(WireLevel::WireLevelError),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for WireLine {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.WireLine", len)?;
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("host_monotonic_ns", ToString::to_string(&self.host_monotonic_ns).as_str())?;
        }
        if true {
            let v = WireDirection::try_from(self.direction)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.direction)))?;
            struct_ser.serialize_field("direction", &v)?;
        }
        if true {
            struct_ser.serialize_field("text", &self.text)?;
        }
        if true {
            let v = WireLevel::try_from(self.level)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.level)))?;
            struct_ser.serialize_field("level", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WireLine {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "host_monotonic_ns",
            "direction",
            "text",
            "level",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            HostMonotonicNs,
            Direction,
            Text,
            Level,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "host_monotonic_ns" => Ok(GeneratedField::HostMonotonicNs),
                            "direction" => Ok(GeneratedField::Direction),
                            "text" => Ok(GeneratedField::Text),
                            "level" => Ok(GeneratedField::Level),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WireLine;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.WireLine")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WireLine, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut host_monotonic_ns__ = None;
                let mut direction__ = None;
                let mut text__ = None;
                let mut level__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::HostMonotonicNs => {
                            if host_monotonic_ns__.is_some() {
                                return Err(serde::de::Error::duplicate_field("host_monotonic_ns"));
                            }
                            host_monotonic_ns__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Direction => {
                            if direction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("direction"));
                            }
                            direction__ = Some(map_.next_value::<WireDirection>()? as i32);
                        }
                        GeneratedField::Text => {
                            if text__.is_some() {
                                return Err(serde::de::Error::duplicate_field("text"));
                            }
                            text__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Level => {
                            if level__.is_some() {
                                return Err(serde::de::Error::duplicate_field("level"));
                            }
                            level__ = Some(map_.next_value::<WireLevel>()? as i32);
                        }
                    }
                }
                Ok(WireLine {
                    host_monotonic_ns: host_monotonic_ns__.unwrap_or_default(),
                    direction: direction__.unwrap_or_default(),
                    text: text__.unwrap_or_default(),
                    level: level__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.WireLine", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WireLog {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.WireLog", len)?;
        if true {
            struct_ser.serialize_field("lines", &self.lines)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WireLog {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "lines",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Lines,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "lines" => Ok(GeneratedField::Lines),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WireLog;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.WireLog")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WireLog, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut lines__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Lines => {
                            if lines__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lines"));
                            }
                            lines__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(WireLog {
                    lines: lines__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.WireLog", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WriteZoneSet {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.WriteZoneSet", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.zone_set.as_ref() {
            struct_ser.serialize_field("zone_set", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WriteZoneSet {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "zone_set",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            ZoneSet,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "zone_set" => Ok(GeneratedField::ZoneSet),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WriteZoneSet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.WriteZoneSet")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WriteZoneSet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut zone_set__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ZoneSet => {
                            if zone_set__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zone_set"));
                            }
                            zone_set__ = map_.next_value()?;
                        }
                    }
                }
                Ok(WriteZoneSet {
                    name: name__.unwrap_or_default(),
                    zone_set: zone_set__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.WriteZoneSet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZeroRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ZeroRequest", len)?;
        if true {
            struct_ser.serialize_field("axes", &self.axes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZeroRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "axes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Axes,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "axes" => Ok(GeneratedField::Axes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZeroRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ZeroRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZeroRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut axes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Axes => {
                            if axes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("axes"));
                            }
                            axes__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ZeroRequest {
                    axes: axes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ZeroRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Zone {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.Zone", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            let v = ZoneShape::try_from(self.shape)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.shape)))?;
            struct_ser.serialize_field("shape", &v)?;
        }
        if true {
            struct_ser.serialize_field("axes", &self.axes)?;
        }
        if true {
            let v = ZoneMetric::try_from(self.metric)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.metric)))?;
            struct_ser.serialize_field("metric", &v)?;
        }
        if true {
            struct_ser.serialize_field("min_cm", &self.min_cm)?;
        }
        if true {
            struct_ser.serialize_field("max_cm", &self.max_cm)?;
        }
        if let Some(v) = self.wrap_cm.as_ref() {
            struct_ser.serialize_field("wrap_cm", v)?;
        }
        if true {
            let v = FireRule::try_from(self.fire)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.fire)))?;
            struct_ser.serialize_field("fire", &v)?;
        }
        if let Some(v) = self.hysteresis_cm.as_ref() {
            struct_ser.serialize_field("hysteresis_cm", v)?;
        }
        if true {
            struct_ser.serialize_field("level", &self.level)?;
        }
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Zone {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "shape",
            "axes",
            "metric",
            "min_cm",
            "max_cm",
            "wrap_cm",
            "fire",
            "hysteresis_cm",
            "level",
            "output",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Shape,
            Axes,
            Metric,
            MinCm,
            MaxCm,
            WrapCm,
            Fire,
            HysteresisCm,
            Level,
            Output,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "shape" => Ok(GeneratedField::Shape),
                            "axes" => Ok(GeneratedField::Axes),
                            "metric" => Ok(GeneratedField::Metric),
                            "min_cm" => Ok(GeneratedField::MinCm),
                            "max_cm" => Ok(GeneratedField::MaxCm),
                            "wrap_cm" => Ok(GeneratedField::WrapCm),
                            "fire" => Ok(GeneratedField::Fire),
                            "hysteresis_cm" => Ok(GeneratedField::HysteresisCm),
                            "level" => Ok(GeneratedField::Level),
                            "output" => Ok(GeneratedField::Output),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Zone;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.Zone")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Zone, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut shape__ = None;
                let mut axes__ = None;
                let mut metric__ = None;
                let mut min_cm__ = None;
                let mut max_cm__ = None;
                let mut wrap_cm__ = None;
                let mut fire__ = None;
                let mut hysteresis_cm__ = None;
                let mut level__ = None;
                let mut output__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Shape => {
                            if shape__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shape"));
                            }
                            shape__ = Some(map_.next_value::<ZoneShape>()? as i32);
                        }
                        GeneratedField::Axes => {
                            if axes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("axes"));
                            }
                            axes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Metric => {
                            if metric__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metric"));
                            }
                            metric__ = Some(map_.next_value::<ZoneMetric>()? as i32);
                        }
                        GeneratedField::MinCm => {
                            if min_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("min_cm"));
                            }
                            min_cm__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MaxCm => {
                            if max_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("max_cm"));
                            }
                            max_cm__ = Some(map_.next_value()?);
                        }
                        GeneratedField::WrapCm => {
                            if wrap_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("wrap_cm"));
                            }
                            wrap_cm__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Fire => {
                            if fire__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fire"));
                            }
                            fire__ = Some(map_.next_value::<FireRule>()? as i32);
                        }
                        GeneratedField::HysteresisCm => {
                            if hysteresis_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hysteresis_cm"));
                            }
                            hysteresis_cm__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Level => {
                            if level__.is_some() {
                                return Err(serde::de::Error::duplicate_field("level"));
                            }
                            level__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Zone {
                    name: name__.unwrap_or_default(),
                    shape: shape__.unwrap_or_default(),
                    axes: axes__.unwrap_or_default(),
                    metric: metric__.unwrap_or_default(),
                    min_cm: min_cm__.unwrap_or_default(),
                    max_cm: max_cm__.unwrap_or_default(),
                    wrap_cm: wrap_cm__,
                    fire: fire__.unwrap_or_default(),
                    hysteresis_cm: hysteresis_cm__,
                    level: level__.unwrap_or_default(),
                    output: output__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.Zone", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZoneBound {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.bound.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ZoneBound", len)?;
        if let Some(v) = self.bound.as_ref() {
            match v {
                zone_bound::Bound::Value(v) => {
                    struct_ser.serialize_field("value", v)?;
                }
                zone_bound::Bound::Reference(v) => {
                    struct_ser.serialize_field("reference", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZoneBound {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "reference",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Reference,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "value" => Ok(GeneratedField::Value),
                            "reference" => Ok(GeneratedField::Reference),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZoneBound;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ZoneBound")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZoneBound, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut bound__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if bound__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            bound__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| zone_bound::Bound::Value(x.0));
                        }
                        GeneratedField::Reference => {
                            if bound__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reference"));
                            }
                            bound__ = map_.next_value::<::std::option::Option<_>>()?.map(zone_bound::Bound::Reference);
                        }
                    }
                }
                Ok(ZoneBound {
                    bound: bound__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ZoneBound", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZoneHitEvent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ZoneHitEvent", len)?;
        if true {
            struct_ser.serialize_field("zone", &self.zone)?;
        }
        if true {
            struct_ser.serialize_field("arm_id", &self.arm_id)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("seq", ToString::to_string(&self.seq).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("host_monotonic_ns", ToString::to_string(&self.host_monotonic_ns).as_str())?;
        }
        if true {
            struct_ser.serialize_field("position_cm", &self.position_cm)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZoneHitEvent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "zone",
            "arm_id",
            "seq",
            "host_monotonic_ns",
            "position_cm",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Zone,
            ArmId,
            Seq,
            HostMonotonicNs,
            PositionCm,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "zone" => Ok(GeneratedField::Zone),
                            "arm_id" => Ok(GeneratedField::ArmId),
                            "seq" => Ok(GeneratedField::Seq),
                            "host_monotonic_ns" => Ok(GeneratedField::HostMonotonicNs),
                            "position_cm" => Ok(GeneratedField::PositionCm),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZoneHitEvent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ZoneHitEvent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZoneHitEvent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut zone__ = None;
                let mut arm_id__ = None;
                let mut seq__ = None;
                let mut host_monotonic_ns__ = None;
                let mut position_cm__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Zone => {
                            if zone__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zone"));
                            }
                            zone__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ArmId => {
                            if arm_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("arm_id"));
                            }
                            arm_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Seq => {
                            if seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("seq"));
                            }
                            seq__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::HostMonotonicNs => {
                            if host_monotonic_ns__.is_some() {
                                return Err(serde::de::Error::duplicate_field("host_monotonic_ns"));
                            }
                            host_monotonic_ns__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PositionCm => {
                            if position_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position_cm"));
                            }
                            position_cm__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ZoneHitEvent {
                    zone: zone__.unwrap_or_default(),
                    arm_id: arm_id__.unwrap_or_default(),
                    seq: seq__.unwrap_or_default(),
                    host_monotonic_ns: host_monotonic_ns__.unwrap_or_default(),
                    position_cm: position_cm__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ZoneHitEvent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZoneMetric {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::ZoneMetricUnspecified => "ZONE_METRIC_UNSPECIFIED",
            Self::ZoneMetricDisplacement => "ZONE_METRIC_DISPLACEMENT",
            Self::ZoneMetricDistance => "ZONE_METRIC_DISTANCE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ZoneMetric {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ZONE_METRIC_UNSPECIFIED",
            "ZONE_METRIC_DISPLACEMENT",
            "ZONE_METRIC_DISTANCE",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = ZoneMetric;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "ZONE_METRIC_UNSPECIFIED" => Ok(ZoneMetric::ZoneMetricUnspecified),
                    "ZONE_METRIC_DISPLACEMENT" => Ok(ZoneMetric::ZoneMetricDisplacement),
                    "ZONE_METRIC_DISTANCE" => Ok(ZoneMetric::ZoneMetricDistance),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ZoneOutput {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ZoneOutput", len)?;
        if true {
            struct_ser.serialize_field("line", &self.line)?;
        }
        if true {
            let v = OutputAction::try_from(self.action)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.action)))?;
            struct_ser.serialize_field("action", &v)?;
        }
        if let Some(v) = self.ms.as_ref() {
            struct_ser.serialize_field("ms", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZoneOutput {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "line",
            "action",
            "ms",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Line,
            Action,
            Ms,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "line" => Ok(GeneratedField::Line),
                            "action" => Ok(GeneratedField::Action),
                            "ms" => Ok(GeneratedField::Ms),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZoneOutput;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ZoneOutput")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZoneOutput, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut line__ = None;
                let mut action__ = None;
                let mut ms__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Line => {
                            if line__.is_some() {
                                return Err(serde::de::Error::duplicate_field("line"));
                            }
                            line__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Action => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("action"));
                            }
                            action__ = Some(map_.next_value::<OutputAction>()? as i32);
                        }
                        GeneratedField::Ms => {
                            if ms__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ms"));
                            }
                            ms__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(ZoneOutput {
                    line: line__.unwrap_or_default(),
                    action: action__.unwrap_or_default(),
                    ms: ms__,
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ZoneOutput", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZoneSet {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ZoneSet", len)?;
        if let Some(v) = self.schema_url.as_ref() {
            struct_ser.serialize_field("schema_url", v)?;
        }
        if true {
            struct_ser.serialize_field("zone_set_version", &self.zone_set_version)?;
        }
        if true {
            struct_ser.serialize_field("zones", &self.zones)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZoneSet {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "schema_url",
            "zone_set_version",
            "zones",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SchemaUrl,
            ZoneSetVersion,
            Zones,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "schema_url" => Ok(GeneratedField::SchemaUrl),
                            "zone_set_version" => Ok(GeneratedField::ZoneSetVersion),
                            "zones" => Ok(GeneratedField::Zones),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZoneSet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ZoneSet")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZoneSet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut schema_url__ = None;
                let mut zone_set_version__ = None;
                let mut zones__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SchemaUrl => {
                            if schema_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schema_url"));
                            }
                            schema_url__ = map_.next_value()?;
                        }
                        GeneratedField::ZoneSetVersion => {
                            if zone_set_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zone_set_version"));
                            }
                            zone_set_version__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Zones => {
                            if zones__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zones"));
                            }
                            zones__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ZoneSet {
                    schema_url: schema_url__,
                    zone_set_version: zone_set_version__.unwrap_or_default(),
                    zones: zones__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ZoneSet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZoneSetName {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ZoneSetName", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZoneSetName {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZoneSetName;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ZoneSetName")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZoneSetName, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ZoneSetName {
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ZoneSetName", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZoneSetNames {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ZoneSetNames", len)?;
        if true {
            struct_ser.serialize_field("zone_sets", &self.zone_sets)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZoneSetNames {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "zone_sets",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ZoneSets,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "zone_sets" => Ok(GeneratedField::ZoneSets),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZoneSetNames;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ZoneSetNames")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZoneSetNames, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut zone_sets__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ZoneSets => {
                            if zone_sets__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zone_sets"));
                            }
                            zone_sets__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ZoneSetNames {
                    zone_sets: zone_sets__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ZoneSetNames", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZoneShape {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::ZoneShapeUnspecified => "ZONE_SHAPE_UNSPECIFIED",
            Self::ZoneShapeRect => "ZONE_SHAPE_RECT",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ZoneShape {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ZONE_SHAPE_UNSPECIFIED",
            "ZONE_SHAPE_RECT",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = ZoneShape;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "ZONE_SHAPE_UNSPECIFIED" => Ok(ZoneShape::ZoneShapeUnspecified),
                    "ZONE_SHAPE_RECT" => Ok(ZoneShape::ZoneShapeRect),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ZoneStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("mousewheeld.v1.ZoneStatus", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            struct_ser.serialize_field("armed", &self.armed)?;
        }
        if true {
            struct_ser.serialize_field("fired", &self.fired)?;
        }
        if true {
            struct_ser.serialize_field("inside", &self.inside)?;
        }
        if let Some(v) = self.fired_at_cm.as_ref() {
            struct_ser.serialize_field("fired_at_cm", v)?;
        }
        if true {
            struct_ser.serialize_field("min_cm", &self.min_cm)?;
        }
        if true {
            struct_ser.serialize_field("max_cm", &self.max_cm)?;
        }
        if let Some(v) = self.wrap_cm.as_ref() {
            struct_ser.serialize_field("wrap_cm", v)?;
        }
        if true {
            let v = ZoneMetric::try_from(self.metric)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.metric)))?;
            struct_ser.serialize_field("metric", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZoneStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "armed",
            "fired",
            "inside",
            "fired_at_cm",
            "min_cm",
            "max_cm",
            "wrap_cm",
            "metric",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Armed,
            Fired,
            Inside,
            FiredAtCm,
            MinCm,
            MaxCm,
            WrapCm,
            Metric,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "armed" => Ok(GeneratedField::Armed),
                            "fired" => Ok(GeneratedField::Fired),
                            "inside" => Ok(GeneratedField::Inside),
                            "fired_at_cm" => Ok(GeneratedField::FiredAtCm),
                            "min_cm" => Ok(GeneratedField::MinCm),
                            "max_cm" => Ok(GeneratedField::MaxCm),
                            "wrap_cm" => Ok(GeneratedField::WrapCm),
                            "metric" => Ok(GeneratedField::Metric),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZoneStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct mousewheeld.v1.ZoneStatus")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZoneStatus, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut armed__ = None;
                let mut fired__ = None;
                let mut inside__ = None;
                let mut fired_at_cm__ = None;
                let mut min_cm__ = None;
                let mut max_cm__ = None;
                let mut wrap_cm__ = None;
                let mut metric__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Armed => {
                            if armed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("armed"));
                            }
                            armed__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Fired => {
                            if fired__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fired"));
                            }
                            fired__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Inside => {
                            if inside__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inside"));
                            }
                            inside__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FiredAtCm => {
                            if fired_at_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fired_at_cm"));
                            }
                            fired_at_cm__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::MinCm => {
                            if min_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("min_cm"));
                            }
                            min_cm__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MaxCm => {
                            if max_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("max_cm"));
                            }
                            max_cm__ = Some(map_.next_value()?);
                        }
                        GeneratedField::WrapCm => {
                            if wrap_cm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("wrap_cm"));
                            }
                            wrap_cm__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Metric => {
                            if metric__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metric"));
                            }
                            metric__ = Some(map_.next_value::<ZoneMetric>()? as i32);
                        }
                    }
                }
                Ok(ZoneStatus {
                    name: name__.unwrap_or_default(),
                    armed: armed__.unwrap_or_default(),
                    fired: fired__.unwrap_or_default(),
                    inside: inside__.unwrap_or_default(),
                    fired_at_cm: fired_at_cm__,
                    min_cm: min_cm__.unwrap_or_default(),
                    max_cm: max_cm__.unwrap_or_default(),
                    wrap_cm: wrap_cm__,
                    metric: metric__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("mousewheeld.v1.ZoneStatus", FIELDS, GeneratedVisitor)
    }
}
