//! Black-box tests for `adico_primitives::slider`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/slider.rs`.

use adico_primitives::slider::{
    RangeSlider, Slider, SliderRange, SliderThumb, SliderTrack, clamp_to_step_bounds,
    closest_thumb_for,
};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn closest_thumb_uses_raw_collision_position() {
    let collided = [80.0, 80.0];

    assert_eq!(closest_thumb_for(79.6, &collided), 0);
    assert_eq!(closest_thumb_for(80.0, &collided), 1);
    assert_eq!(closest_thumb_for(80.4, &collided), 1);
}

#[test]
fn clamp_to_step_bounds_keeps_fallbacks_in_range() {
    assert_eq!(clamp_to_step_bounds(8.0, 5.0, 8.0, 10.0), 5.0);
    assert_eq!(clamp_to_step_bounds(5.0, 5.0, 8.0, 10.0), 5.0);
    assert_eq!(clamp_to_step_bounds(93.0, 93.0, 95.0, 10.0), 95.0);
}

#[test]
fn clamp_to_step_bounds_preserves_available_step_ticks() {
    assert_eq!(clamp_to_step_bounds(85.0, 72.0, 85.0, 10.0), 80.0);
    assert_eq!(clamp_to_step_bounds(84.0, 78.0, 89.0, 10.0), 80.0);
}

#[component]
fn HalfwaySlider() -> Element {
    rsx! {
        Slider { label: "Demo Slider", default_value: 50.0,
            SliderTrack {
                SliderRange {}
                SliderThumb {}
            }
        }
    }
}

#[test]
fn a_single_thumb_slider_reports_its_value_and_range() {
    let html = render(HalfwaySlider);
    assert!(html.contains(r#"role="slider""#), "{html}");
    assert!(html.contains("aria-valuemin=0"), "{html}");
    assert!(html.contains("aria-valuemax=100"), "{html}");
    assert!(html.contains("aria-valuenow=50"), "{html}");
    assert!(html.contains(r#"aria-label="Demo Slider""#), "{html}");
    assert!(html.contains(r#"data-orientation="horizontal""#), "{html}");
    assert!(html.contains("left: 50%"), "{html}");
}

#[component]
fn VerticalDisabledSlider() -> Element {
    rsx! {
        Slider { label: "Vertical", default_value: 25.0, horizontal: false, disabled: true,
            SliderTrack {
                SliderRange {}
                SliderThumb {}
            }
        }
    }
}

#[test]
fn a_disabled_vertical_slider_marks_data_disabled_and_vertical_orientation() {
    let html = render(VerticalDisabledSlider);
    assert!(html.contains(r#"data-orientation="vertical""#), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
    assert!(html.contains("bottom: 25%"), "{html}");
}

#[component]
fn TwentyToEightyRangeSlider() -> Element {
    rsx! {
        RangeSlider { label: "Range Slider", default_value: 20.0f64..80.0f64,
            SliderTrack {
                SliderRange {}
                SliderThumb { index: 0usize }
                SliderThumb { index: 1usize }
            }
        }
    }
}

#[test]
fn a_range_slider_renders_two_independently_bounded_thumbs() {
    let html = render(TwentyToEightyRangeSlider);
    assert!(html.contains("data-index=0"), "{html}");
    assert!(html.contains("data-index=1"), "{html}");
    assert!(html.contains("left: 20%"), "{html}");
    assert!(html.contains("left: 80%"), "{html}");
    assert!(html.contains("left: 20%; right: 20%"), "{html}");
}
