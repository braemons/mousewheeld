//! Publishing what the wheel did, to whoever is reading.
//!
//! **The shared-memory segment is the fast path, and it is written first.**
//! vstimd reads it every frame, inside the frame, and cannot wait for a socket
//! send, a ring insert or a file write. Everything else this daemon does with a
//! sample happens after these few atomic stores.
//!
//! What goes in it is **centimetres**, and the axis descriptor says `scale =
//! 1.0`. The calibration belongs here — the daemon that owns the encoder — so
//! vstimd's rig config names a segment and restates nothing about the wheel.
//!
//! The layout comes from vstimd's own `vinput` crate rather than a copy of it.
//! That is the one exception to "no shared code" in this family, and barely
//! one: a memory layout is a contract between a writer and a reader, and the
//! reader's own definition is the only copy that cannot drift.

use std::sync::Mutex;

use vinput::{AxisDesc, Semantic, VinputOwner};

/// The segment, and the axes in it.
pub struct SegmentPublisher {
    name: String,
    owner: Mutex<VinputOwner>,
}

impl SegmentPublisher {
    /// Create the segment, or say why not and carry on without it.
    ///
    /// A rig with no display attached is a normal thing — a calibration
    /// session, a bench — so a segment that cannot be created is a warning and
    /// not a reason to refuse to start. Everything else this daemon does still
    /// works, and the console says the segment is not open.
    pub fn create(shm_name: &str, axes: &[String]) -> Option<Self> {
        if axes.is_empty() {
            log::warn!("no axes to publish; {shm_name} not created");
            return None;
        }
        let descriptors: Vec<AxisDesc> = axes
            .iter()
            // Cumulative, and never a delta: the consumer differences
            // successive totals, so nothing is lost or double-counted however
            // the write and read rates interleave. `1.0` because what is in
            // here is already the unit — centimetres.
            .map(|name| AxisDesc::new(name, Semantic::Cumulative, 1.0))
            .collect();
        match VinputOwner::create(shm_name, &descriptors) {
            Ok(owner) => {
                log::info!("publishing {} axes to {shm_name}, in centimetres", descriptors.len());
                Some(Self {
                    name: shm_name.to_string(),
                    owner: Mutex::new(owner),
                })
            }
            Err(problem) => {
                log::warn!("cannot publish to {shm_name}: {problem}");
                None
            }
        }
    }

    /// One sample, in centimetres. Called from the link's reader thread, first,
    /// before anything else is done with the sample.
    pub fn publish(&self, centimetres: &[f64]) {
        if let Ok(mut owner) = self.owner.lock() {
            owner.write(centimetres);
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Completed writes since the segment was created — what a reader's own
    /// `write_count` is compared against when somebody asks why a corridor is
    /// not moving.
    pub fn write_count(&self) -> u64 {
        self.owner
            .lock()
            .map(|owner| owner.write_count())
            .unwrap_or(0)
    }
}
