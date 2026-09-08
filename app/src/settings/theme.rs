use settings::macros::define_settings_group;
use settings::{RespectUserSyncSetting, Setting, SupportedPlatforms, SyncToCloud};
use warp_core::ui::theme::{BackgroundShader, WarpTheme};
use warpui::platform::SystemTheme;
use warpui::rendering::dither::DitherConfig;
use warpui::AppContext;

use crate::themes::theme::{RespectSystemTheme, SelectedSystemThemes, ThemeKind};

// Theme selection and global background effect overrides.
define_settings_group!(ThemeSettings, settings: [
    theme_kind: Theme {
        type: ThemeKind,
        // Note that for new users, we now override this default value in SettingsInitializer
        // to set the default theme to Phenomenon.
        default: ThemeKind::default(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.themes.theme",
        max_table_depth: 0,
        description: "The color theme.",
    },
    use_system_theme: UseSystemTheme {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        storage_key: "SystemTheme",
        toml_path: "appearance.themes.system_theme",
        description: "Whether to match the system light/dark theme.",
    },
    selected_system_themes: SystemThemes {
        type: SelectedSystemThemes,
        default: SelectedSystemThemes::default(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        storage_key: "SelectedSystemThemes",
        toml_path: "appearance.themes.selected_system_themes",
        max_table_depth: 0,
        description: "The themes to use for system light and dark modes.",
    },
    dither_enabled: DitherEnabled {
        type: Option<bool>,
        default: None,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.dither.enabled",
        description: "Enable dither on background images. Omit to use the theme default.",
    },
    dither_pixel_size: DitherPixelSize {
        type: Option<u8>,
        default: None,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.dither.pixel_size",
        description: "Dither grain size in logical pixels, from 1 to 32. Omit to use the theme default.",
    },
    dither_strength: DitherStrength {
        type: Option<u8>,
        default: None,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.dither.strength",
        description: "Dither strength from 0 to 100. Omit to use the theme default.",
    },
    dither_animated: DitherAnimated {
        type: Option<bool>,
        default: None,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.dither.animated",
        description: "Animate the dither effect. Omit to use the theme default.",
    },
]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundDitherState {
    pub enabled: bool,
    pub config: DitherConfig,
}

impl ThemeSettings {
    pub fn background_dither(&self, theme: &WarpTheme) -> Option<BackgroundDitherState> {
        if !warpui::SUPPORTS_BACKGROUND_SHADERS {
            return None;
        }
        theme
            .background_image()
            .map(|image| self.resolve_dither(image.shader))
    }

    fn resolve_dither(&self, shader: Option<BackgroundShader>) -> BackgroundDitherState {
        let defaults = shader
            .map(|shader| match shader {
                BackgroundShader::Dither(config) => config,
            })
            .unwrap_or_default();
        BackgroundDitherState {
            enabled: self.dither_enabled.value().unwrap_or(shader.is_some()),
            config: self.dither_config(defaults),
        }
    }

    /// Resolve optional user overrides against the active theme's shader defaults.
    pub fn dither_config(&self, theme: DitherConfig) -> DitherConfig {
        DitherConfig {
            pixel_size: self
                .dither_pixel_size
                .value()
                .unwrap_or(theme.pixel_size)
                .clamp(1, 32),
            strength: self
                .dither_strength
                .value()
                .unwrap_or(theme.strength)
                .min(100),
            animated: self.dither_animated.value().unwrap_or(theme.animated),
        }
    }
}

impl Theme {
    fn current_value_is_syncable(&self) -> bool {
        self.value().is_custom_theme_reference_syncable()
    }
}

impl SystemThemes {
    fn current_value_is_syncable(&self) -> bool {
        let selected = self.value();
        selected.light.is_custom_theme_reference_syncable()
            && selected.dark.is_custom_theme_reference_syncable()
    }
}

/// Returns a derived value for whether to respect the system theme based on
/// the current theme settings.
pub fn respect_system_theme(theme_settings: &ThemeSettings) -> RespectSystemTheme {
    if *theme_settings.use_system_theme.value() {
        RespectSystemTheme::On(theme_settings.selected_system_themes.value().clone())
    } else {
        RespectSystemTheme::Off
    }
}

/// Returns the current theme kind based on the theme settings and the system theme.
pub fn derived_theme_kind(theme_settings: &ThemeSettings, system_theme: SystemTheme) -> ThemeKind {
    let respect_system_theme = respect_system_theme(theme_settings);
    match respect_system_theme {
        RespectSystemTheme::On(selected_system_themes) => match system_theme {
            SystemTheme::Light => selected_system_themes.light.clone(),
            SystemTheme::Dark => selected_system_themes.dark.clone(),
        },
        RespectSystemTheme::Off => theme_settings.theme_kind.value().clone(),
    }
}

/// Return the current theme kind based on the theme settings and active app context.
pub fn active_theme_kind(theme_settings: &ThemeSettings, app: &AppContext) -> ThemeKind {
    derived_theme_kind(theme_settings, app.system_theme())
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;
