//! The output lines, by name.
//!
//! A zone set says `"line": "zone_goal"`. The pin behind that name is the
//! rig's, in its config, uploaded to the board at connect and **not writable
//! over the API** — wiring is changed where wiring is described. This is the
//! same split statemachined draws between its graphs and its line map, and it
//! is what lets one zone set run on every rig that has a `zone_goal`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputLine {
    /// What a zone set calls it.
    pub name: String,
    /// The board's own index for this line — what the wire carries, since the
    /// protocol is indices and not names.
    pub index: u8,
    /// The pin it is wired to, on this board.
    pub pin: u8,
    /// The level the firmware drives when nothing else says otherwise. Outputs
    /// reach this before anything else runs.
    #[serde(default)]
    pub safe_high: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LineMap {
    pub lines: Vec<OutputLine>,
}
