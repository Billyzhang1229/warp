use std::cell::Cell;

use settings::Setting;
use warpui::elements::{Element, MouseStateHandle};
use warpui::ui_components::components::{UiComponent, UiComponentStyles};
use warpui::ui_components::slider::SliderStateHandle;
use warpui::ui_components::switch::SwitchStateHandle;
use warpui::{AppContext, ModelContext, SingletonEntity};

use super::{AppearancePageAction, AppearanceSettingsPageView, OPACITY_SLIDER_WIDTH};
use crate::appearance::Appearance;
use crate::settings::BackgroundImageSettings;
use crate::settings_view::settings_page::{
    build_reset_button, render_body_item, LocalOnlyIconState, SettingsWidget, ToggleState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundImageAdjustment {
    Opacity,
    Brightness,
    Contrast,
    GradientStrength,
    GradientStart,
    VignetteStrength,
}

impl BackgroundImageAdjustment {
    fn label(self) -> &'static str {
        match self {
            Self::Opacity => "Image opacity",
            Self::Brightness => "Brightness",
            Self::Contrast => "Contrast",
            Self::GradientStrength => "Bottom darkening strength",
            Self::GradientStart => "Gradient start from top",
            Self::VignetteStrength => "Vignette strength",
        }
    }

    fn search_terms(self) -> &'static str {
        match self {
            Self::Opacity => "background image opacity transparency mix",
            Self::Brightness => "background image brightness color exposure",
            Self::Contrast => "background image contrast color",
            Self::GradientStrength => {
                "background image bottom darkening gradient strength intensity"
            }
            Self::GradientStart => "background image gradient start position top",
            Self::VignetteStrength => "background image vignette strength intensity edges",
        }
    }

    fn maximum(self) -> u16 {
        match self {
            Self::Brightness | Self::Contrast => 200,
            Self::Opacity
            | Self::GradientStrength
            | Self::GradientStart
            | Self::VignetteStrength => 100,
        }
    }

    fn value(self, app: &AppContext) -> u16 {
        let settings = BackgroundImageSettings::as_ref(app);
        match self {
            Self::Opacity => settings
                .resolve(Appearance::as_ref(app).theme())
                .map_or(100, |state| u16::from(state.opacity)),
            Self::Brightness => *settings.brightness,
            Self::Contrast => *settings.contrast,
            Self::GradientStrength => u16::from(*settings.gradient_strength),
            Self::GradientStart => u16::from(*settings.gradient_start),
            Self::VignetteStrength => u16::from(*settings.vignette_strength),
        }
        .min(self.maximum())
    }

    pub(super) fn set(
        self,
        value: f32,
        settings: &mut BackgroundImageSettings,
        ctx: &mut ModelContext<BackgroundImageSettings>,
    ) -> anyhow::Result<()> {
        if !value.is_finite() {
            return Ok(());
        }
        let value = value.round().clamp(0., f32::from(self.maximum())) as u16;
        match self {
            Self::Opacity => settings.opacity.set_value(Some(value as u8), ctx),
            Self::Brightness => settings.brightness.set_value(value, ctx),
            Self::Contrast => settings.contrast.set_value(value, ctx),
            Self::GradientStrength => settings.gradient_strength.set_value(value as u8, ctx),
            Self::GradientStart => settings.gradient_start.set_value(value as u8, ctx),
            Self::VignetteStrength => settings.vignette_strength.set_value(value as u8, ctx),
        }
    }
}

pub(super) fn has_background_image(app: &AppContext) -> bool {
    BackgroundImageSettings::as_ref(app)
        .resolve(Appearance::as_ref(app).theme())
        .is_some()
}

pub(super) fn widgets() -> Vec<Box<dyn SettingsWidget<View = AppearanceSettingsPageView>>> {
    use BackgroundImageAdjustment::*;
    vec![
        Box::new(ImageSlider::new(Opacity)),
        Box::new(ImageSlider::new(Brightness)),
        Box::new(ImageSlider::new(Contrast)),
        Box::new(ImageToggle::new(ImageToggleKind::Gradient)),
        Box::new(ImageSlider::new(GradientStrength)),
        Box::new(ImageSlider::new(GradientStart)),
        Box::new(ImageToggle::new(ImageToggleKind::Vignette)),
        Box::new(ImageSlider::new(VignetteStrength)),
        Box::new(ImageReset::default()),
    ]
}

struct ImageSlider {
    adjustment: BackgroundImageAdjustment,
    state: SliderStateHandle,
    last_value: Cell<Option<u16>>,
}

impl ImageSlider {
    fn new(adjustment: BackgroundImageAdjustment) -> Self {
        Self {
            adjustment,
            state: SliderStateHandle::default(),
            last_value: Cell::new(None),
        }
    }
}

impl SettingsWidget for ImageSlider {
    type View = AppearanceSettingsPageView;

    fn widget_id(&self) -> &'static str {
        self.adjustment.search_terms()
    }
    fn search_terms(&self) -> &str {
        self.adjustment.search_terms()
    }
    fn should_render(&self, app: &AppContext) -> bool {
        has_background_image(app)
    }

    fn render(
        &self,
        _: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let adjustment = self.adjustment;
        let value = adjustment.value(app);
        if self.last_value.replace(Some(value)) != Some(value) {
            self.state.reset_offset();
        }
        render_body_item::<AppearancePageAction>(
            format!("{}: {value}%", adjustment.label()),
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
            appearance
                .ui_builder()
                .slider(self.state.clone())
                .with_range(0.0..f32::from(adjustment.maximum()))
                .with_step(1.0)
                .with_default_value(f32::from(value))
                .with_style(UiComponentStyles {
                    width: Some(OPACITY_SLIDER_WIDTH),
                    ..Default::default()
                })
                .on_drag(move |ctx, _, value| {
                    ctx.dispatch_typed_action(AppearancePageAction::SetBackgroundImageAdjustment(
                        adjustment, value,
                    ))
                })
                .on_change(move |ctx, _, value| {
                    ctx.dispatch_typed_action(AppearancePageAction::SetBackgroundImageAdjustment(
                        adjustment, value,
                    ))
                })
                .build()
                .finish(),
            None,
        )
    }
}

#[derive(Clone, Copy)]
enum ImageToggleKind {
    Gradient,
    Vignette,
}

struct ImageToggle {
    kind: ImageToggleKind,
    state: SwitchStateHandle,
}

impl ImageToggle {
    fn new(kind: ImageToggleKind) -> Self {
        Self {
            kind,
            state: SwitchStateHandle::default(),
        }
    }
}

impl SettingsWidget for ImageToggle {
    type View = AppearanceSettingsPageView;

    fn widget_id(&self) -> &'static str {
        match self.kind {
            ImageToggleKind::Gradient => "BackgroundImageGradientToggle",
            ImageToggleKind::Vignette => "BackgroundImageVignetteToggle",
        }
    }
    fn search_terms(&self) -> &str {
        match self.kind {
            ImageToggleKind::Gradient => {
                "background image bottom darkening gradient enable disable on off"
            }
            ImageToggleKind::Vignette => "background image vignette enable disable on off",
        }
    }
    fn should_render(&self, app: &AppContext) -> bool {
        has_background_image(app)
    }
    fn render(
        &self,
        _: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let state = BackgroundImageSettings::as_ref(app).resolve(appearance.theme());
        let (label, enabled, action) = match self.kind {
            ImageToggleKind::Gradient => (
                "Bottom darkening",
                state.is_some_and(|state| state.gradient_enabled),
                AppearancePageAction::ToggleBackgroundGradient,
            ),
            ImageToggleKind::Vignette => (
                "Vignette",
                state.is_some_and(|state| state.vignette_enabled),
                AppearancePageAction::ToggleBackgroundVignette,
            ),
        };
        render_body_item::<AppearancePageAction>(
            label.into(),
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
            appearance
                .ui_builder()
                .switch(self.state.clone())
                .check(enabled)
                .build()
                .on_click(move |ctx, _, _| ctx.dispatch_typed_action(action.clone()))
                .finish(),
            None,
        )
    }
}

#[derive(Default)]
struct ImageReset {
    state: MouseStateHandle,
}

impl SettingsWidget for ImageReset {
    type View = AppearanceSettingsPageView;
    fn search_terms(&self) -> &str {
        "background image restore reset defaults"
    }
    fn should_render(&self, app: &AppContext) -> bool {
        has_background_image(app)
    }
    fn render(&self, _: &Self::View, appearance: &Appearance, _: &AppContext) -> Box<dyn Element> {
        render_body_item::<AppearancePageAction>(
            "Restore image defaults".into(),
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
            build_reset_button(appearance, self.state.clone(), true)
                .with_text_label("Restore defaults".into())
                .build()
                .on_click(|ctx, _, _| {
                    ctx.dispatch_typed_action(AppearancePageAction::ResetBackgroundImage)
                })
                .finish(),
            None,
        )
    }
}
