/// Static adjustments applied only to workspace background images.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundImageEffects {
    pub brightness: u16,
    pub contrast: u16,
    pub gradient_strength: u8,
    pub gradient_start: u8,
    pub vignette_strength: u8,
}

impl Default for BackgroundImageEffects {
    fn default() -> Self {
        Self {
            brightness: 100,
            contrast: 100,
            gradient_strength: 0,
            gradient_start: 50,
            vignette_strength: 0,
        }
    }
}

impl BackgroundImageEffects {
    pub fn is_identity(self) -> bool {
        self.brightness == 100
            && self.contrast == 100
            && (self.gradient_strength == 0 || self.gradient_start >= 100)
            && self.vignette_strength == 0
    }

    pub fn parameters(self) -> [[f32; 4]; 2] {
        if self.is_identity() {
            return [[0.; 4]; 2];
        }
        [
            [
                f32::from(self.brightness.min(200)) / 100.,
                f32::from(self.contrast.min(200)) / 100.,
                f32::from(self.gradient_strength.min(100)) / 100.,
                f32::from(self.gradient_start.min(100)) / 100.,
            ],
            [
                f32::from(self.vignette_strength.min(100)) / 100.,
                1.,
                0.,
                0.,
            ],
        ]
    }
}

#[cfg(test)]
#[path = "background_image_tests.rs"]
mod tests;
