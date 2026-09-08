use warp_core::ui::theme::{mock_terminal_colors, Image as ThemeImage, WarpTheme};
use warpui::assets::asset_cache::AssetSource;
use warpui::color::ColorU;
use warpui::App;

use super::*;
use crate::settings_view::settings_page::{search_terms_match, FilteredPageType};

#[test]
fn dither_controls_follow_plain_image_availability_and_search() {
    App::test((), |mut app| async move {
        app.update(|ctx| {
            ctx.add_singleton_model(ThemeSettings::new_with_defaults);
            ctx.add_singleton_model(BackgroundImageSettings::new_with_defaults);
            let appearance = ctx.add_singleton_model(|_| Appearance::mock());
            let mut page = PageType::new_uncategorized(
                vec![
                    Box::new(DitherEnabledWidget::default()),
                    Box::new(DitherSizeWidget::default()),
                    Box::new(DitherStrengthWidget::default()),
                    Box::new(DitherAnimationWidget::default()),
                ],
                None,
            );
            page.update_filter("", ctx);
            assert_eq!(visible_dither_widgets(&page), 0);

            let theme = WarpTheme::new(
                ColorU::black().into(),
                ColorU::white(),
                ColorU::white().into(),
                None,
                None,
                mock_terminal_colors(),
                Some(ThemeImage {
                    source: AssetSource::Raw {
                        id: "plain-background-test".into(),
                    },
                    opacity: 100,
                    shader: None,
                }),
                Some("Plain background".into()),
            );
            appearance.update(ctx, |appearance, ctx| appearance.set_theme(theme, ctx));
            let mut image_page =
                PageType::new_uncategorized(background_image_controls::widgets(), None);
            image_page.update_filter("", ctx);
            assert_eq!(
                visible_dither_widgets(&image_page),
                if warpui::SUPPORTS_BACKGROUND_SHADERS {
                    9
                } else {
                    0
                }
            );
            page.update_filter("", ctx);
            if warpui::SUPPORTS_BACKGROUND_SHADERS {
                assert_eq!(visible_dither_widgets(&page), 4);
                assert!(!current_background_dither(ctx).unwrap().enabled);
                page.update_filter("grain size", ctx);
                assert_eq!(visible_dither_widgets(&page), 1);
                page.update_filter("", ctx);
                assert_eq!(visible_dither_widgets(&page), 4);
            } else {
                assert_eq!(visible_dither_widgets(&page), 0);
            }

            appearance.update(ctx, |appearance, ctx| {
                appearance.set_theme(Appearance::mock().theme().clone(), ctx)
            });
            page.update_filter("dither", ctx);
            assert_eq!(visible_dither_widgets(&page), 0);
            image_page.update_filter("background image", ctx);
            assert_eq!(visible_dither_widgets(&image_page), 0);
        });
    });
}

fn visible_dither_widgets(page: &PageType<AppearanceSettingsPageView>) -> usize {
    let FilteredPageType::Uncategorized { widgets, .. } = page.get_filtered() else {
        panic!("expected widget list");
    };
    widgets.len()
}

#[test]
fn dither_search_is_scoped_to_individual_controls() {
    let widgets: Vec<Box<dyn SettingsWidget<View = AppearanceSettingsPageView>>> = vec![
        Box::new(DitherEnabledWidget::default()),
        Box::new(DitherSizeWidget::default()),
        Box::new(DitherStrengthWidget::default()),
        Box::new(DitherAnimationWidget::default()),
    ];
    for query in ["grain size", "dither strength", "animation", "dither off"] {
        assert_eq!(
            widgets
                .iter()
                .filter(|widget| search_terms_match(widget.search_terms(), query))
                .count(),
            1,
            "{query}"
        );
    }
    for query in ["dither", ""] {
        assert_eq!(
            widgets
                .iter()
                .filter(|widget| search_terms_match(widget.search_terms(), query))
                .count(),
            4
        );
    }
}

#[test]
fn background_image_search_and_ids_are_independent() {
    let widgets = background_image_controls::widgets();
    let ids: std::collections::HashSet<_> =
        widgets.iter().map(|widget| widget.widget_id()).collect();
    assert_eq!(ids.len(), 9);
    for query in [
        "image opacity",
        "brightness",
        "contrast",
        "gradient off",
        "gradient strength",
        "gradient start",
        "vignette off",
        "vignette strength",
        "restore defaults",
    ] {
        assert_eq!(
            widgets
                .iter()
                .filter(|widget| search_terms_match(widget.search_terms(), query))
                .count(),
            1,
            "{query}"
        );
    }
    assert_eq!(
        widgets
            .iter()
            .filter(|widget| search_terms_match(widget.search_terms(), "background image"))
            .count(),
        9
    );
}
