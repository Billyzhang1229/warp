use settings::macros::define_settings_group;
use settings::{RespectUserSyncSetting, Setting, SupportedPlatforms, SyncToCloud};
use warp_core::ui::theme::WarpTheme;
use warpui::ModelContext;
use warpui::rendering::background_image::BackgroundImageEffects;

// Separate from ThemeSettings: adjusting an image must not reload its theme or asset.
define_settings_group!(BackgroundImageSettings, settings: [
    opacity: BackgroundImageOpacity {
        type: Option<u8>, default: None,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI, private: false,
        toml_path: "appearance.background_image.opacity",
        description: "Image opacity from 0 to 100. Omit to use the theme opacity.",
    },
    brightness: BackgroundImageBrightness {
        type: u16, default: 100,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI, private: false,
        toml_path: "appearance.background_image.brightness",
        description: "Background image brightness from 0 to 200 percent.",
    },
    contrast: BackgroundImageContrast {
        type: u16, default: 100,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI, private: false,
        toml_path: "appearance.background_image.contrast",
        description: "Background image contrast from 0 to 200 percent.",
    },
    gradient_enabled: BackgroundImageGradientEnabled {
        type: bool, default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI, private: false,
        toml_path: "appearance.background_image.gradient_enabled",
        description: "Gradually darken the background image toward the bottom.",
    },
    gradient_strength: BackgroundImageGradientStrength {
        type: u8, default: 60,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI, private: false,
        toml_path: "appearance.background_image.gradient_strength",
        description: "Bottom darkening strength from 0 to 100 percent.",
    },
    gradient_start: BackgroundImageGradientStart {
        type: u8, default: 50,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI, private: false,
        toml_path: "appearance.background_image.gradient_start",
        description: "Gradient start measured from the window top, from 0 to 100 percent.",
    },
    vignette_enabled: BackgroundImageVignetteEnabled {
        type: bool, default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI, private: false,
        toml_path: "appearance.background_image.vignette_enabled",
        description: "Darken the background image edges with a soft elliptical vignette.",
    },
    vignette_strength: BackgroundImageVignetteStrength {
        type: u8, default: 40,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI, private: false,
        toml_path: "appearance.background_image.vignette_strength",
        description: "Background image vignette strength from 0 to 100 percent.",
    },
]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundImageState {
    pub opacity: u8,
    pub effects: BackgroundImageEffects,
    pub gradient_enabled: bool,
    pub vignette_enabled: bool,
}

impl BackgroundImageSettings {
    pub fn resolve(&self, theme: &WarpTheme) -> Option<BackgroundImageState> {
        if !warpui::SUPPORTS_BACKGROUND_SHADERS {
            return None;
        }
        theme.background_image().map(|image| BackgroundImageState {
            opacity: self.opacity.value().unwrap_or(image.opacity).min(100),
            effects: BackgroundImageEffects {
                brightness: (*self.brightness).min(200),
                contrast: (*self.contrast).min(200),
                gradient_strength: if *self.gradient_enabled {
                    (*self.gradient_strength).min(100)
                } else {
                    0
                },
                gradient_start: (*self.gradient_start).min(100),
                vignette_strength: if *self.vignette_enabled {
                    (*self.vignette_strength).min(100)
                } else {
                    0
                },
            },
            gradient_enabled: *self.gradient_enabled,
            vignette_enabled: *self.vignette_enabled,
        })
    }

    pub fn reset(&mut self, ctx: &mut ModelContext<Self>) -> anyhow::Result<()> {
        self.opacity.clear_value(ctx)?;
        self.brightness.clear_value(ctx)?;
        self.contrast.clear_value(ctx)?;
        self.gradient_enabled.clear_value(ctx)?;
        self.gradient_strength.clear_value(ctx)?;
        self.gradient_start.clear_value(ctx)?;
        self.vignette_enabled.clear_value(ctx)?;
        self.vignette_strength.clear_value(ctx)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "background_image_tests.rs"]
mod tests;
