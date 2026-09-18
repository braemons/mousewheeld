// SPDX-License-Identifier: AGPL-3.0-or-later
//! The board, and the wire to it.

use crate::model::device as m;
use crate::model::version as v;
use crate::wire;

pub fn version_report_to_wire(report: v::VersionReport) -> wire::VersionReport {
    wire::VersionReport {
        daemon: report.daemon,
        api: report.api,
        device_protocol: Some(wire::DeviceProtocol {
            speaks: report.device_protocol.speaks,
            floor: report.device_protocol.floor,
            board: report.device_protocol.board,
        }),
        vinput_layout: report.vinput_layout,
    }
}

pub fn device_info_to_wire(info: m::DeviceInfo) -> wire::DeviceInfo {
    wire::DeviceInfo {
        connected: info.connected,
        port: info.port,
        board: info.board,
        firmware_version: info.firmware_version,
        protocol_version: info.protocol_version,
        uptime_device_us: info.uptime_device_us,
        capacities: Some(wire::Capacities {
            // The firmware's limits are u8 and u16; protobuf's smallest integer
            // is 32 bits. Widening here rather than inventing a narrower wire
            // type keeps one bound in one place — the board reports it.
            n_axes: u32::from(info.capacities.n_axes),
            max_zones: u32::from(info.capacities.max_zones),
            max_lines: u32::from(info.capacities.max_lines),
            scan_hz: info.capacities.scan_hz,
        }),
        flashed_zone_set: info.flashed_zone_set.map(|set| wire::FlashedZoneSet {
            name: set.name,
            version: set.version,
        }),
        link: Some(wire::LinkStats {
            connection_count: info.link.connection_count,
            last_error: info.link.last_error,
        }),
    }
}

pub fn firmware_versions_to_wire(versions: m::FirmwareVersions) -> wire::FirmwareVersions {
    wire::FirmwareVersions {
        running: versions.running,
        available: versions.available,
    }
}

pub fn wire_log_to_wire(log: m::WireLog) -> wire::WireLog {
    wire::WireLog {
        lines: log.lines.into_iter().map(wire_line_to_wire).collect(),
    }
}

pub fn wire_line_to_wire(line: m::WireLine) -> wire::WireLine {
    wire::WireLine {
        host_monotonic_ns: line.host_monotonic_ns,
        direction: match line.direction {
            m::WireDirection::Out => wire::WireDirection::Out,
            m::WireDirection::In => wire::WireDirection::In,
        } as i32,
        text: line.text,
        level: match line.level {
            m::WireLevel::Info => wire::WireLevel::Info,
            m::WireLevel::Error => wire::WireLevel::Error,
        } as i32,
    }
}
