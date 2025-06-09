use crate::drawing::primitives::{draw_rect, draw_rect_outline};
use crate::drawing::text::{compute_text_bbox, draw_str, get_font};
use crate::uitk::{ContentId, UiContext};
use crate::{Color, FbView, FbViewMut, Framebuffer, OwnedPixels, Rect, StyleSheet};
use alloc::borrow::ToOwned;
use alloc::string::String;

impl<'a, F: FbViewMut> UiContext<'a, F> {
    pub fn button(&mut self, config: &ButtonConfig) -> bool {
        let mut active = false;
        self.button_inner(config, &mut active, false);
        active
    }

    pub fn button_toggle(&mut self, config: &ButtonConfig, active: &mut bool) {
        self.button_inner(config, active, false);
    }

    pub fn button_toggle_once(&mut self, config: &ButtonConfig, active: &mut bool) {
        self.button_inner(config, active, true);
    }

    fn button_inner(&mut self, config: &ButtonConfig, active: &mut bool, toggle_once: bool) {
        let UiContext {
            fb,
            input_state,
            stylesheet,
            tile_cache,
            ..
        } = self;

        let ps = &input_state.pointer;
        let hovered = config.rect.check_contains_point(ps.x, ps.y) && !(*active && toggle_once);
        let clicked = hovered && ps.left_click_trigger;

        let state = {
            if hovered && !clicked {
                ButtonState::Hover
            } else {
                if hovered && clicked {
                    *active = !(*active);
                }
                match *active {
                    true => ButtonState::Clicked,
                    false => ButtonState::Idle,
                }
            }
        };

        let content_id = ContentId::from_hash(&(
            state,
            &config.rect,
            &config.text,
            *active,
            config.icon.as_ref().map(|(name, _)| name),
        ));

        let button_fb = tile_cache.fetch_or_create(content_id, self.time, || {
            render_button(stylesheet, config, state, *active)
        });

        let Rect { x0, y0, .. } = config.rect;
        fb.copy_from_fb(button_fb, (x0, y0), false);
    }
}

fn render_button(
    stylesheet: &StyleSheet,
    config: &ButtonConfig,
    state: ButtonState,
    active: bool,
) -> Framebuffer<OwnedPixels> {
    let rect = config.rect.zero_origin();

    let Rect { w, h, .. } = rect;
    let mut button_fb = Framebuffer::new_owned(w, h);

    let colorsheet = &stylesheet.colors;

    let button_rect = rect;

    draw_rect(&mut button_fb, &button_rect, colorsheet.element, false);

    let (mut x, gap) = match config.indicator_mode {
        ButtonIndicatorMode::Light => {
            let indicator_h = 3 * button_rect.h / 4;
            let indicator_w = 10;
            let gap = i64::max(0, button_rect.h as i64 - indicator_h as i64) / 2;

            let indicator_rect = Rect {
                x0: button_rect.x0 + gap,
                y0: button_rect.y0 + gap,
                w: indicator_w,
                h: indicator_h,
            };
            let color = match active {
                true => colorsheet.accent,
                false => colorsheet.background,
            };
            draw_rect(&mut button_fb, &indicator_rect, color, false);

            let x = button_rect.x0 + gap + indicator_w as i64;
            (x, gap)
        }

        _ => match &config.icon {
            Some((_, icon_fb)) => {
                let (_icon_w, icon_h) = icon_fb.shape();
                let gap = i64::max(0, button_rect.h as i64 - icon_h as i64) / 2;
                (button_rect.x0, gap)
            }

            None => (button_rect.x0, 0),
        },
    };

    if config.indicator_mode == ButtonIndicatorMode::Border && active {
        draw_rect_outline(
            &mut button_fb,
            &button_rect,
            Color::WHITE,
            false,
            stylesheet.margin,
        );
    }

    if let Some(icon) = &config.icon {
        let (_, icon_fb) = icon;
        let (icon_w, icon_h) = icon_fb.shape();

        let mut icon_rect = Rect {
            x0: 0,
            y0: 0,
            w: icon_w,
            h: icon_h,
        }
        .align_to_rect_vert(&button_rect);

        if config.text.is_empty() {
            let content_rect = {
                let [_, y0, x1, y1] = button_rect.as_xyxy();
                Rect::from_xyxy([x, y0, x1, y1])
            };
            icon_rect = icon_rect.align_to_rect_horiz(&content_rect);
        } else {
            icon_rect.x0 = x + gap;
        }

        button_fb.copy_from_fb(*icon_fb, (icon_rect.x0, icon_rect.y0), true);

        let [_, _, x1, _] = icon_rect.as_xyxy();
        x = x1;
    }

    if !config.text.is_empty() {
        let font = get_font(&stylesheet.text.font_family(), stylesheet.text.sizes.medium);

        let (text_w, text_h) = compute_text_bbox(&config.text, font);

        let mut text_rect = Rect {
            x0: 0,
            y0: 0,
            w: text_w,
            h: text_h,
        }
        .align_to_rect_vert(&button_rect);

        if config.icon.is_none() && config.indicator_mode != ButtonIndicatorMode::Light {
            let content_rect = {
                let [_, y0, x1, y1] = button_rect.as_xyxy();
                Rect::from_xyxy([x, y0, x1, y1])
            };
            text_rect = text_rect.align_to_rect_horiz(&content_rect);
        } else {
            text_rect.x0 = x + gap;
        }

        draw_str(
            &mut button_fb,
            &config.text,
            text_rect.x0,
            text_rect.y0,
            font,
            colorsheet.text,
            None,
        );
    }

    if state == ButtonState::Hover {
        draw_rect(&mut button_fb, &button_rect, colorsheet.hover_overlay, true);
    }

    button_fb
}

#[derive(PartialEq, Hash, Clone, Copy)]
enum ButtonState {
    Idle,
    Hover,
    Clicked,
}

#[derive(PartialEq, Hash, Clone, Copy)]
pub enum ButtonIndicatorMode {
    Off,
    Light,
    Border,
}

#[derive(Clone)]
pub struct ButtonConfig {
    pub rect: Rect,
    pub text: String,
    pub icon: Option<(String, &'static Framebuffer<OwnedPixels>)>,
    pub untoggle: bool,
    pub indicator_mode: ButtonIndicatorMode,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        ButtonConfig {
            rect: Rect {
                x0: 0,
                y0: 0,
                w: 100,
                h: 25,
            },
            text: "".to_owned(),
            icon: None,
            untoggle: true,
            indicator_mode: ButtonIndicatorMode::Off,
        }
    }
}
