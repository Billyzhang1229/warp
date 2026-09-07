use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Parameters for the dot-and-cross background shader. Sizes are in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DitherConfig {
    pub pixel_size: u8,
    pub strength: u8,
    pub animated: bool,
}

impl Default for DitherConfig {
    fn default() -> Self {
        Self {
            pixel_size: 4,
            strength: 100,
            animated: true,
        }
    }
}

impl DitherConfig {
    pub const FRAME_INTERVAL: Duration = Duration::from_nanos(33_333_334);

    pub fn parameters(self, elapsed: Duration, scale_factor: f32) -> [f32; 4] {
        [
            if self.animated {
                elapsed.as_secs_f32()
            } else {
                0.0
            },
            f32::from(self.strength.min(100)) / 100.0,
            (f32::from(self.pixel_size.clamp(1, 32)) * scale_factor)
                .round()
                .max(1.0),
            if self.animated { 1.0 } else { 0.0 },
        ]
    }

    pub fn should_animate(self, window_active: bool) -> bool {
        self.animated && self.strength > 0 && window_active
    }
}

#[cfg(test)]
#[path = "dither_tests.rs"]
mod tests;
