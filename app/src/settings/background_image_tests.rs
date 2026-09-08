use settings::{PrivatePreferences, PublicPreferences, SettingsManager};
use warp_core::ui::theme::{BackgroundShader, Image as ThemeImage, mock_terminal_colors};
use warpui::assets::asset_cache::AssetSource;
use warpui::color::ColorU;
use warpui::rendering::dither::DitherConfig;
use warpui::{App, SingletonEntity};
use warpui_extras::user_preferences;

use super::*;
use crate::settings::ThemeSettings;

fn theme(opacity: u8, shader: Option<BackgroundShader>) -> WarpTheme {
    WarpTheme::new(
        ColorU::black().into(),
        ColorU::white(),
        ColorU::white().into(),
        None,
        None,
        mock_terminal_colors(),
        Some(ThemeImage {
            source: AssetSource::Raw {
                id: "background-controls-test".into(),
            },
            opacity,
            shader,
        }),
        None,
    )
}

#[test]
fn image_adjustments_inherit_opacity_and_are_independent_of_legacy_dither() {
    App::test((), |mut app| async move {
        app.update(|ctx| {
            let image = ctx.add_singleton_model(BackgroundImageSettings::new_with_defaults);
            ctx.add_singleton_model(ThemeSettings::new_with_defaults);
            let plain = theme(30, None);
            let legacy = theme(65, Some(BackgroundShader::Dither(DitherConfig::default())));
            let settings = image.as_ref(ctx);
            assert!(
                settings
                    .resolve(crate::appearance::Appearance::mock().theme())
                    .is_none()
            );
            if !warpui::SUPPORTS_BACKGROUND_SHADERS {
                return;
            }
            assert_eq!(settings.resolve(&plain).unwrap().opacity, 30);
            assert_eq!(settings.resolve(&legacy).unwrap().opacity, 65);
            assert!(settings.resolve(&legacy).unwrap().effects.is_identity());
            assert!(
                !ThemeSettings::as_ref(ctx)
                    .background_dither(&plain)
                    .unwrap()
                    .enabled
            );
            assert!(
                ThemeSettings::as_ref(ctx)
                    .background_dither(&legacy)
                    .unwrap()
                    .enabled
            );
            image.update(ctx, |settings, _| {
                settings.opacity = BackgroundImageOpacity::new(Some(Some(255)));
                settings.brightness = BackgroundImageBrightness::new(Some(u16::MAX));
                settings.contrast = BackgroundImageContrast::new(Some(0));
                settings.gradient_enabled = BackgroundImageGradientEnabled::new(Some(true));
                settings.gradient_strength = BackgroundImageGradientStrength::new(Some(255));
                settings.gradient_start = BackgroundImageGradientStart::new(Some(255));
                settings.vignette_enabled = BackgroundImageVignetteEnabled::new(Some(true));
                settings.vignette_strength = BackgroundImageVignetteStrength::new(Some(255));
            });
            let plain_state = image.as_ref(ctx).resolve(&plain).unwrap();
            assert_eq!(plain_state, image.as_ref(ctx).resolve(&legacy).unwrap());
            assert_eq!(plain_state.opacity, 100);
            assert_eq!(
                plain_state.effects,
                BackgroundImageEffects {
                    brightness: 200,
                    contrast: 0,
                    gradient_strength: 100,
                    gradient_start: 100,
                    vignette_strength: 100,
                }
            );
        });
    });
}

#[test]
fn image_adjustments_persist_and_reset_without_erasing_dither() {
    App::test((), |mut app| async move {
        app.update(|ctx| {
            ctx.add_singleton_model(|_| {
                PublicPreferences::new(
                    Box::<user_preferences::in_memory::InMemoryPreferences>::default(),
                )
            });
            ctx.add_singleton_model(|_| {
                PrivatePreferences::new(
                    Box::<user_preferences::in_memory::InMemoryPreferences>::default(),
                )
            });
            ctx.add_singleton_model(|_| SettingsManager::default());
            BackgroundImageSettings::register(ctx);
            ThemeSettings::register(ctx);
            ThemeSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings.dither_enabled.set_value(Some(true), ctx).unwrap();
                settings.dither_strength.set_value(Some(65), ctx).unwrap();
            });
            BackgroundImageSettings::handle(ctx).update(ctx, |settings, ctx| {
                settings.opacity.set_value(Some(83), ctx).unwrap();
                settings.brightness.set_value(120, ctx).unwrap();
                settings.contrast.set_value(140, ctx).unwrap();
                settings.gradient_strength.set_value(77, ctx).unwrap();
                settings.gradient_enabled.set_value(true, ctx).unwrap();
                settings.vignette_strength.set_value(62, ctx).unwrap();
                settings.vignette_enabled.set_value(true, ctx).unwrap();
                settings.gradient_enabled.set_value(false, ctx).unwrap();
                settings.vignette_enabled.set_value(false, ctx).unwrap();
            });
            let reloaded = ctx.add_model(BackgroundImageSettings::new_from_storage);
            let settings = reloaded.as_ref(ctx);
            assert_eq!(*settings.opacity, Some(83));
            assert_eq!(*settings.brightness, 120);
            assert_eq!(*settings.contrast, 140);
            assert_eq!(*settings.gradient_strength, 77);
            assert_eq!(*settings.vignette_strength, 62);
            if warpui::SUPPORTS_BACKGROUND_SHADERS {
                let state = settings.resolve(&theme(30, None)).unwrap();
                assert!(!state.gradient_enabled && !state.vignette_enabled);
                assert_eq!(state.effects.gradient_strength, 0);
                assert_eq!(state.effects.vignette_strength, 0);
            }
            BackgroundImageSettings::handle(ctx)
                .update(ctx, |settings, ctx| settings.reset(ctx).unwrap());
            let reloaded = ctx.add_model(BackgroundImageSettings::new_from_storage);
            let settings = reloaded.as_ref(ctx);
            assert_eq!(*settings.opacity, None);
            assert_eq!(*settings.brightness, 100);
            assert_eq!(*settings.contrast, 100);
            assert_eq!(*settings.gradient_strength, 60);
            assert_eq!(*settings.gradient_start, 50);
            assert_eq!(*settings.vignette_strength, 40);
            assert_eq!(*ThemeSettings::as_ref(ctx).dither_enabled, Some(true));
            assert_eq!(*ThemeSettings::as_ref(ctx).dither_strength, Some(65));
        });
    });
}
