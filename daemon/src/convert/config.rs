// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rates, and the line map.

use crate::model::config as m;
use crate::model::line_map as l;
use crate::wire;

pub fn config_view_to_wire(view: m::ConfigView) -> wire::ConfigView {
    wire::ConfigView {
        rate_hz: view.rate_hz,
        display_hz: view.display_hz,
        ring_minutes: view.ring_minutes,
        shm_name: view.shm_name,
        shm_open: view.shm_open,
        shm_writes: view.shm_writes,
        event_port: u32::from(view.event_port),
        port: view.port,
        starves_the_display: view.starves_the_display,
    }
}

pub fn config_patch_from_wire(patch: wire::ConfigPatch) -> m::ConfigPatch {
    m::ConfigPatch {
        rate_hz: patch.rate_hz,
        display_hz: patch.display_hz,
        ring_minutes: patch.ring_minutes,
    }
}

pub fn line_map_to_wire(map: l::LineMap) -> wire::LineMap {
    wire::LineMap {
        lines: map
            .lines
            .into_iter()
            .map(|line| wire::OutputLine {
                name: line.name,
                index: u32::from(line.index),
                pin: u32::from(line.pin),
                safe_high: line.safe_high,
            })
            .collect(),
    }
}
