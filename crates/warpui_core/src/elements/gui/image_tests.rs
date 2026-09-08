use super::*;

struct TestElement;

impl Element for TestElement {
    fn layout(
        &mut self,
        constraint: SizeConstraint,
        _: &mut LayoutContext,
        _: &AppContext,
    ) -> Vector2F {
        constraint.max
    }

    fn after_layout(&mut self, _: &mut AfterLayoutContext, _: &AppContext) {}

    fn paint(&mut self, _: Vector2F, _: &mut PaintContext, _: &AppContext) {}

    fn size(&self) -> Option<Vector2F> {
        None
    }

    fn origin(&self) -> Option<Point> {
        None
    }

    fn dispatch_event(
        &mut self,
        _: &DispatchedEvent,
        _: &mut EventContext,
        _: &AppContext,
    ) -> bool {
        false
    }
}

fn test_element() -> Box<dyn Element> {
    Box::new(TestElement)
}

fn test_image() -> Image {
    Image::new(
        AssetSource::Raw {
            id: "test".to_string(),
        },
        CacheOption::BySize,
    )
}

#[test]
fn image_rect_returns_none_for_nan_origin() {
    assert!(
        image_rect(
            vec2f(164.0, 164.0),
            vec2f(f32::NAN, 874.725),
            vec2f(163.75, 163.75),
            false,
            false,
        )
        .is_none()
    );
}

#[test]
fn dither_preserves_cover_geometry_for_landscape_and_portrait_images() {
    let plain = test_image().cover();
    let effect =
        test_image()
            .cover()
            .with_dither(DitherConfig::default(), Instant::now(), WindowId::new());
    let viewport = vec2f(800., 600.);
    assert_eq!(
        dimensions(vec2f(1200., 400.), viewport, plain.fit_type),
        dimensions(vec2f(1200., 400.), viewport, effect.fit_type)
    );
    assert_eq!(
        dimensions(vec2f(400., 1200.), viewport, plain.fit_type),
        dimensions(vec2f(400., 1200.), viewport, effect.fit_type)
    );
}

#[test]
fn dither_zero_strength_uses_the_original_image_path() {
    let image = test_image().cover().with_dither(
        DitherConfig {
            strength: 0,
            ..DitherConfig::default()
        },
        Instant::now(),
        WindowId::new(),
    );
    assert!(image.dither.is_none());
    assert_eq!(
        dimensions(vec2f(1200., 400.), vec2f(800., 600.), image.fit_type),
        vec2f(1800., 600.)
    );
}

#[test]
fn image_adjustments_preserve_geometry_and_survive_zero_dither() {
    let effects = BackgroundImageEffects {
        brightness: 120,
        gradient_strength: 60,
        vignette_strength: 40,
        ..Default::default()
    };
    let image = test_image()
        .cover()
        .with_background_effects(effects)
        .with_dither(
            DitherConfig {
                strength: 0,
                ..Default::default()
            },
            Instant::now(),
            WindowId::new(),
        );
    assert!(image.dither.is_none());
    assert_eq!(image.background_effects, Some(effects));
    for source in [vec2f(1200., 400.), vec2f(400., 1200.)] {
        assert_eq!(
            dimensions(source, vec2f(800., 600.), image.fit_type),
            dimensions(source, vec2f(800., 600.), test_image().cover().fit_type)
        );
    }
    assert!(
        image
            .with_background_effects(BackgroundImageEffects::default())
            .background_effects
            .is_none()
    );
}

#[test]
fn failed_to_load_prefers_failure_element_when_provided() {
    let image = test_image()
        .before_load(test_element())
        .on_load_failure(test_element());

    assert_eq!(
        image.failed_to_load_backup_element_kind(),
        Some(BackupElementKind::FailedToLoad)
    );
}

#[test]
fn failed_to_load_falls_back_to_before_load_element() {
    let image = test_image().before_load(test_element());

    assert_eq!(
        image.failed_to_load_backup_element_kind(),
        Some(BackupElementKind::BeforeLoad)
    );
}

#[test]
fn loading_image_switches_to_timeout_element_after_timeout() {
    let mut image = test_image()
        .before_load(test_element())
        .on_load_timeout(Duration::from_secs(10), test_element());
    image.clear_load_timeout_started_at();
    let now = Instant::now();

    let (initial_kind, initial_repaint_after) = image.loading_backup_element_kind(now);
    assert_eq!(initial_kind, Some(BackupElementKind::BeforeLoad));
    assert_eq!(initial_repaint_after, Some(Duration::from_secs(10)));

    let (timed_out_kind, timed_out_repaint_after) =
        image.loading_backup_element_kind(now + Duration::from_secs(11));
    assert_eq!(timed_out_kind, Some(BackupElementKind::LoadTimeout));
    assert_eq!(timed_out_repaint_after, None);
}

#[test]
fn loading_timeout_survives_image_rebuild_for_same_source() {
    let mut image = test_image()
        .before_load(test_element())
        .on_load_timeout(Duration::from_secs(10), test_element());
    image.clear_load_timeout_started_at();
    let now = Instant::now();

    let (initial_kind, _initial_repaint_after) = image.loading_backup_element_kind(now);
    assert_eq!(initial_kind, Some(BackupElementKind::BeforeLoad));

    let mut rebuilt_image = test_image()
        .before_load(test_element())
        .on_load_timeout(Duration::from_secs(10), test_element());
    let (timed_out_kind, timed_out_repaint_after) =
        rebuilt_image.loading_backup_element_kind(now + Duration::from_secs(11));
    assert_eq!(timed_out_kind, Some(BackupElementKind::LoadTimeout));
    assert_eq!(timed_out_repaint_after, None);
}
