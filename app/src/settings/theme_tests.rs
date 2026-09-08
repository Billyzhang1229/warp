use std::path::PathBuf;

use super::*;
use crate::themes::theme::CustomTheme;
use crate::user_config;

fn custom(path: PathBuf) -> ThemeKind {
    ThemeKind::Custom(CustomTheme::new("Custom".to_string(), path))
}

fn custom_base16(path: PathBuf) -> ThemeKind {
    ThemeKind::CustomBase16(CustomTheme::new("Base16 Custom".to_string(), path))
}

#[test]
fn theme_kind_syncs_custom_theme_under_theme_root() {
    let setting = Theme::new(Some(custom(user_config::themes_dir().join("custom.yml"))));

    assert!(setting.current_value_is_syncable());
}

#[test]
fn theme_kind_does_not_sync_custom_theme_outside_theme_root() {
    let setting = Theme::new(Some(custom(std::env::temp_dir().join("custom.yml"))));

    assert!(!setting.current_value_is_syncable());
}

#[test]
fn theme_kind_syncs_custom_base16_theme_under_theme_root() {
    let setting = Theme::new(Some(custom_base16(
        user_config::themes_dir().join("base16/custom.yml"),
    )));

    assert!(setting.current_value_is_syncable());
}

#[test]
fn selected_system_themes_sync_when_custom_paths_are_under_theme_root() {
    let setting = SystemThemes::new(Some(SelectedSystemThemes {
        light: custom(user_config::themes_dir().join("light.yml")),
        dark: custom_base16(user_config::themes_dir().join("dark.yml")),
    }));

    assert!(setting.current_value_is_syncable());
}

#[test]
fn selected_system_themes_do_not_sync_when_any_custom_path_is_outside_theme_root() {
    let setting = SystemThemes::new(Some(SelectedSystemThemes {
        light: custom(user_config::themes_dir().join("light.yml")),
        dark: custom(std::env::temp_dir().join("dark.yml")),
    }));

    assert!(!setting.current_value_is_syncable());
}

#[test]
fn built_in_theme_settings_remain_syncable() {
    let theme = Theme::new(Some(ThemeKind::Dark));
    let system_themes = SystemThemes::new(Some(SelectedSystemThemes {
        light: ThemeKind::Light,
        dark: ThemeKind::Dark,
    }));

    assert!(theme.current_value_is_syncable());
    assert!(system_themes.current_value_is_syncable());
}

fn dither_settings(
    size: Option<u8>,
    strength: Option<u8>,
    animated: Option<bool>,
) -> ThemeSettings {
    ThemeSettings {
        theme_kind: Theme::new(None),
        use_system_theme: UseSystemTheme::new(None),
        selected_system_themes: SystemThemes::new(None),
        dither_enabled: DitherEnabled::new(None),
        dither_pixel_size: DitherPixelSize::new(Some(size)),
        dither_strength: DitherStrength::new(Some(strength)),
        dither_animated: DitherAnimated::new(Some(animated)),
    }
}

#[test]
fn dither_overrides_preserve_theme_defaults_until_changed() {
    let theme = DitherConfig {
        pixel_size: 7,
        strength: 42,
        animated: false,
    };
    assert_eq!(
        dither_settings(None, None, None).dither_config(theme),
        theme
    );
    assert_eq!(
        dither_settings(Some(12), Some(80), Some(true)).dither_config(theme),
        DitherConfig {
            pixel_size: 12,
            strength: 80,
            animated: true
        }
    );
}

#[test]
fn dither_overrides_clamp_config_file_values() {
    let config =
        dither_settings(Some(0), Some(255), Some(false)).dither_config(DitherConfig::default());
    assert_eq!(config.pixel_size, 1);
    assert_eq!(config.strength, 100);
    assert!(!config.should_animate(true));
    let config =
        dither_settings(Some(255), Some(0), Some(true)).dither_config(DitherConfig::default());
    assert_eq!(config.pixel_size, 32);
    assert!(!config.should_animate(true));
}

#[test]
fn dither_defaults_to_off_for_plain_images_and_on_for_legacy_shaders() {
    let settings = dither_settings(None, None, None);
    assert!(!settings.resolve_dither(None).enabled);
    assert!(
        settings
            .resolve_dither(Some(BackgroundShader::Dither(DitherConfig::default())))
            .enabled
    );
}

#[test]
fn dither_saved_boolean_overrides_apply_to_any_image() {
    let mut settings = dither_settings(Some(12), Some(65), Some(false));
    let legacy = Some(BackgroundShader::Dither(DitherConfig::default()));
    let saved_enabled = serde_json::to_string(&true).unwrap();
    settings.dither_enabled =
        DitherEnabled::new(Some(serde_json::from_str(&saved_enabled).unwrap()));
    let enabled = settings.resolve_dither(None);
    assert!(enabled.enabled);
    assert_eq!(enabled.config.pixel_size, 12);
    assert_eq!(settings.resolve_dither(legacy), enabled);

    settings.dither_enabled = DitherEnabled::new(Some(serde_json::from_str("false").unwrap()));
    let disabled = settings.resolve_dither(legacy);
    assert!(!disabled.enabled);
    assert_eq!(disabled.config, enabled.config);
    assert_eq!(
        serde_json::to_string(settings.dither_enabled.value()).unwrap(),
        "false"
    );
}

#[test]
fn dither_is_unavailable_without_a_background_image() {
    let mut settings = dither_settings(None, None, None);
    settings.dither_enabled = DitherEnabled::new(Some(Some(true)));
    assert!(
        settings
            .background_dither(crate::appearance::Appearance::mock().theme())
            .is_none()
    );
}
