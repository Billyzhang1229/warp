use super::*;

#[test]
fn clamps_extreme_config_and_aligns_grain_to_device_pixels() {
    let config = DitherConfig {
        pixel_size: 0,
        strength: 255,
        animated: true,
    };
    assert_eq!(
        config.parameters(Duration::from_secs(2), 1.5),
        [2.0, 1.0, 2.0, 1.0]
    );
    let config = DitherConfig {
        pixel_size: 255,
        ..config
    };
    assert_eq!(
        config.parameters(Duration::ZERO, 2.0),
        [0.0, 1.0, 64.0, 1.0]
    );
}

#[test]
fn static_effect_has_no_time_or_motion() {
    let config = DitherConfig {
        animated: false,
        ..Default::default()
    };
    assert_eq!(
        config.parameters(Duration::from_secs(500), 2.0),
        [0.0, 1.0, 8.0, 0.0]
    );
    assert!(!config.should_animate(true));
}

#[test]
fn inactive_window_does_not_schedule_animation() {
    assert!(!DitherConfig::default().should_animate(false));
    assert!(DitherConfig::default().should_animate(true));
}

#[test]
fn disabled_effect_does_not_schedule_animation() {
    let config = DitherConfig {
        strength: 0,
        ..Default::default()
    };
    assert!(!config.should_animate(true));
}
