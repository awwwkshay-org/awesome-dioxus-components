//! Black-box tests for `adico_primitives::color_picker`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/color_picker.rs`. Also carries the module's 2 previously-inline
//! `#[cfg(test)] mod tests` tests, moved here verbatim.

use adico_primitives::color_picker::{
    AreaThumb, AreaThumbSaturationInput, AreaThumbValueInput, AreaTrack, Color, ColorArea,
    ColorPicker, color_name,
};
use dioxus::prelude::*;
use palette::{Hsv, RgbHue, encoding};

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn ColorAreaWithZeroChildThumb() -> Element {
    rsx! {
        ColorPicker {
            color: Hsv::<encoding::Srgb, f64>::new(RgbHue::new(155.0), 0.5, 0.75),
            ColorArea {
                AreaTrack {
                    AreaThumb {}
                }
            }
        }
    }
}

#[component]
fn ColorAreaWithAccessibleThumbInputs() -> Element {
    rsx! {
        ColorPicker {
            color: Hsv::<encoding::Srgb, f64>::new(RgbHue::new(155.0), 0.5, 0.75),
            ColorArea {
                AreaTrack {
                    AreaThumb {
                        AreaThumbSaturationInput { class: "custom-saturation" }
                        AreaThumbValueInput { class: "custom-value" }
                    }
                }
            }
        }
    }
}

#[test]
fn area_thumb_allows_zero_children() {
    let html = render(ColorAreaWithZeroChildThumb);
    assert_eq!(html.matches("type=\"range\"").count(), 0);
}

#[test]
fn area_thumb_preserves_explicit_axis_input_slots() {
    let html = render(ColorAreaWithAccessibleThumbInputs);
    assert_eq!(html.matches("type=\"range\"").count(), 2);
    assert!(html.contains("custom-saturation"));
    assert!(html.contains("custom-value"));
}

#[test]
fn color_name_classifies_pure_black_and_white_without_a_hue() {
    assert_eq!(color_name(Color::new(0, 0, 0)), "black");
    assert_eq!(color_name(Color::new(255, 255, 255)), "white");
}

#[test]
fn color_name_classifies_a_neutral_gray_as_gray() {
    let name = color_name(Color::new(128, 128, 128));
    assert!(name.contains("gray"), "{name}");
}

#[test]
fn color_name_classifies_a_saturated_red_as_red() {
    let name = color_name(Color::new(220, 20, 20));
    assert!(name.contains("red"), "{name}");
}

#[test]
fn color_name_classifies_a_saturated_blue_as_blue() {
    let name = color_name(Color::new(20, 20, 220));
    assert!(name.contains("blue"), "{name}");
}

#[test]
fn the_color_picker_root_reports_the_group_role_and_label() {
    let html = render(ColorAreaWithZeroChildThumb);
    assert!(html.contains(r#"role="group""#), "{html}");
    assert!(html.contains(r#"aria-label="Color picker""#), "{html}");
    assert!(html.contains("data-disabled=false"), "{html}");
}

#[test]
fn the_area_track_carries_a_css_custom_property_for_the_hue_backdrop() {
    let html = render(ColorAreaWithZeroChildThumb);
    assert!(html.contains("--area-color:"), "{html}");
}

#[test]
fn the_area_thumb_reports_a_label_and_dragging_and_position_style() {
    let html = render(ColorAreaWithZeroChildThumb);
    assert!(html.contains(r#"aria-label="Color area""#), "{html}");
    assert!(html.contains("data-dragging=false"), "{html}");
    assert!(html.contains("left:"), "{html}");
    assert!(html.contains("top:"), "{html}");
}

#[test]
fn the_saturation_and_value_inputs_are_native_range_sliders_with_descriptive_valuetext() {
    let html = render(ColorAreaWithAccessibleThumbInputs);
    assert!(html.contains(r#"aria-label="Saturation""#), "{html}");
    assert!(html.contains(r#"aria-label="Value""#), "{html}");
    assert_eq!(
        html.matches(r#"aria-roledescription="2D Slider""#).count(),
        2,
        "{html}"
    );
    assert!(html.contains(r#"aria-orientation="horizontal""#), "{html}");
    assert!(html.contains(r#"aria-orientation="vertical""#), "{html}");
    assert!(html.contains("Saturation 50%"), "{html}");
    assert!(html.contains("Value 75%"), "{html}");
}
