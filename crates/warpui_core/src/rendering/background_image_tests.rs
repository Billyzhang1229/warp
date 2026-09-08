use super::*;

#[test]
fn background_image_neutral_and_end_of_window_gradient_bypass_adjustments() {
    assert_eq!(BackgroundImageEffects::default().parameters(), [[0.; 4]; 2]);
    assert!(BackgroundImageEffects {
        gradient_start: 100,
        gradient_strength: 80,
        ..Default::default()
    }
    .is_identity());
    assert!(!BackgroundImageEffects {
        brightness: 120,
        gradient_start: 100,
        gradient_strength: 80,
        ..Default::default()
    }
    .is_identity());
}

#[test]
fn background_image_gpu_parameters_clamp_extreme_values() {
    let parameters = BackgroundImageEffects {
        brightness: u16::MAX,
        contrast: 0,
        gradient_strength: u8::MAX,
        gradient_start: u8::MAX,
        vignette_strength: u8::MAX,
    }
    .parameters();
    assert_eq!(parameters, [[2., 0., 1., 1.], [1., 1., 0., 0.]]);
}
