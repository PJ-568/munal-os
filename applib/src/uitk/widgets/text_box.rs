use alloc::string::String;

use crate::content::{ContentId, TrackedContent};
use crate::drawing::primitives::draw_rect;
use crate::drawing::text::{
    draw_rich_slice, format_rich_lines, get_font, Font, FormattedRichText, RichText,
    TextJustification,
};
use crate::Color;
use crate::Rect;
use crate::{FbView, FbViewMut};

use crate::uitk::{TileRenderer, UiContext};

use crate::uitk::text::{string_input, EditableText};
use crate::uitk::UuidProvider;

const CURSOR_BLINK_PERIOD: u64 = 1;

impl<'a, F: FbViewMut> UiContext<'a, F> {
    pub fn text_box<T: FormattableText>(
        &mut self,
        dst_rect: &Rect,
        text: &T,
        state: &mut TextBoxState,
        autoscroll: bool,
    ) {
        let prelude: Option<&T> = None;
        let bg_color = self.stylesheet.colors.element;
        self.text_box_inner(
            dst_rect, text, bg_color, state, autoscroll, false, prelude, false,
        );
    }

    pub fn editable_text_box<T: FormattableText + EditableText, U: FormattableText>(
        &mut self,
        dst_rect: &Rect,
        text: &mut T,
        state: &mut TextBoxState,
        autoscroll: bool,
        allow_newline: bool,
        prelude: Option<&U>,
    ) {
        let UiContext {
            input_state,
            uuid_provider,
            ..
        } = self;

        let bg_color = self.stylesheet.colors.editable;

        let old_cursor = state.cursor;

        string_input(
            text,
            input_state,
            allow_newline,
            &mut state.cursor,
            *uuid_provider,
        );

        let cursor_changed = state.cursor != old_cursor;

        self.text_box_inner(
            dst_rect,
            text,
            bg_color,
            state,
            cursor_changed,
            autoscroll,
            prelude,
            true,
        );
    }

    fn text_box_inner<T: FormattableText, U: FormattableText>(
        &mut self,
        dst_rect: &Rect,
        text: &T,
        bg_color: Color,
        state: &mut TextBoxState,
        mut cursor_changed: bool,
        autoscroll: bool,
        prelude: Option<&U>,
        cursor_enabled: bool,
    ) {
        let time_sec = (self.time as u64) / 1000;

        // Only used if text is not already a RichText
        let font = get_font(
            &self.stylesheet.text.font_family(),
            self.stylesheet.text.sizes.medium,
        );

        let color = self.stylesheet.colors.text;

        let (rich_text, prelude_len) = match prelude {
            None => {
                let rich = text.to_rich_text(color, font);
                (rich, 0)
            }
            Some(prelude) => {
                let (mut rich_1, cid_1) = prelude.to_rich_text(color, font).to_inner();
                let (rich_2, cid_2) = text.to_rich_text(color, font).to_inner();

                let prelude_len = rich_1.len();

                rich_1.concat(rich_2);
                let cid = ContentId::from_hash(&(cid_1, cid_2));
                let rich = TrackedContent::new_with_id(rich_1, cid);

                (rich, prelude_len)
            }
        };

        let formatted = {
            let formatted =
                format_rich_lines(rich_text.as_ref(), dst_rect.w - CURSOR_W, state.justif);
            let content_id = ContentId::from_hash(&(rich_text.get_id(), dst_rect.w, state.justif));
            TrackedContent::new_with_id(formatted, content_id)
        };

        let p = &self.input_state.pointer;
        let vr = dst_rect;

        let mut shadow_cursor = None;

        if dst_rect.check_contains_point(p.x, p.y) {
            let (ox, oy) = state.scroll_offsets;
            let (x_text, y_text) = (p.x - vr.x0 + ox, p.y - vr.y0 + oy);
            if let Some(index) = formatted.as_ref().xy_to_index((x_text, y_text)) {
                if p.left_click_trigger {
                    state.cursor = index - prelude_len;
                    cursor_changed = true;
                } else {
                    shadow_cursor = Some(index - prelude_len);
                }
            }
        }

        if !cursor_enabled {
            state.cursor_visible = false;
        } else if cursor_changed {
            state.last_blink_t = time_sec;
            state.cursor_visible = true;
        } else if time_sec - state.last_blink_t > CURSOR_BLINK_PERIOD {
            state.last_blink_t = time_sec;
            state.cursor_visible = !state.cursor_visible;
        }

        let formatted_content_id = formatted.get_id();

        let renderer = TextRenderer {
            formatted,
            bg_color,
            cursor: state.cursor,
            cursor_visible: state.cursor_visible,
            shadow_cursor,
            prelude_len,
        };

        if autoscroll {
            let TextBoxState {
                content_id,
                scroll_offsets,
                ..
            } = state;
            match content_id {
                Some(content_id) if *content_id == formatted_content_id => (),
                _ => {
                    let (_, scroll_y0) = scroll_offsets;
                    let (_, max_h) = renderer.shape();
                    *scroll_y0 = (max_h - dst_rect.h - 1).into();
                    *content_id = Some(formatted_content_id);
                }
            }
        }

        self.dynamic_canvas(
            dst_rect,
            &renderer,
            &mut state.scroll_offsets,
            &mut state.scroll_dragging,
        );
    }
}

pub struct TextBoxState {
    pub content_id: Option<ContentId>,
    pub scroll_offsets: (i64, i64),
    pub scroll_dragging: (bool, bool),
    pub cursor: usize,
    pub justif: TextJustification,

    cursor_visible: bool,
    last_blink_t: u64,
}

impl TextBoxState {
    pub fn new() -> Self {
        Self {
            content_id: None,
            scroll_offsets: (0, 0),
            scroll_dragging: (false, false),
            cursor: 0,
            justif: TextJustification::Left,
            cursor_visible: true,
            last_blink_t: 0,
        }
    }
}

pub trait FormattableText {
    fn to_rich_text(&self, color: Color, font: &'static Font) -> TrackedContent<RichText>;
}

impl FormattableText for TrackedContent<String> {
    fn to_rich_text(&self, color: Color, font: &'static Font) -> TrackedContent<RichText> {
        let rich_text = RichText::from_str(self.as_ref(), color, font, None);
        let new_id = ContentId::from_hash(&(self.get_id(), color, font.name.as_str()));
        TrackedContent::new_with_id(rich_text, new_id)
    }
}

impl FormattableText for TrackedContent<RichText> {
    fn to_rich_text(&self, _color: Color, _font: &'static Font) -> TrackedContent<RichText> {
        let content_id = self.get_id();
        let rich_text = self.as_ref().clone();
        TrackedContent::new_with_id(rich_text, content_id)
    }
}

pub struct EditableRichText<'a> {
    pub font: &'static Font,
    pub color: Color,
    pub rich_text: &'a mut TrackedContent<RichText>,
}

impl<'a> EditableText for EditableRichText<'a> {
    fn len(&self) -> usize {
        self.rich_text.as_ref().len()
    }

    fn insert(&mut self, uuid_provider: &mut UuidProvider, pos: usize, c: char) {
        self.rich_text
            .mutate(uuid_provider)
            .insert(pos, c, self.color, self.font);
    }

    fn remove(&mut self, uuid_provider: &mut UuidProvider, pos: usize) {
        self.rich_text.mutate(uuid_provider).remove(pos);
    }
}

impl<'a> FormattableText for EditableRichText<'a> {
    fn to_rich_text(&self, _color: Color, _font: &'static Font) -> TrackedContent<RichText> {
        self.rich_text.clone()
    }
}

struct TextRenderer {
    formatted: TrackedContent<FormattedRichText>,
    bg_color: Color,
    cursor: usize,
    shadow_cursor: Option<usize>,
    prelude_len: usize,
    cursor_visible: bool,
}

const CURSOR_W: u32 = 2;
const MIN_TILE_W: u32 = 200;
const TILE_H: u32 = 200;

impl TileRenderer for TextRenderer {
    fn shape(&self) -> (u32, u32) {
        let FormattedRichText { w, h, .. } = *self.formatted.as_ref();
        (w + CURSOR_W, h)
    }

    fn tile_shape(&self) -> (u32, u32) {
        let FormattedRichText { w, .. } = *self.formatted.as_ref();
        (u32::max(w + CURSOR_W, MIN_TILE_W), TILE_H)
    }

    fn content_id(&self, tile_rect: &Rect) -> ContentId {
        let formatted = self.formatted.as_ref();
        let FormattedRichText { w, h, .. } = *formatted;
        let cursor_h = formatted
            .lines
            .last()
            .map(|l| l.chars.last())
            .flatten()
            .map(|c| c.font.char_h as u32)
            .unwrap_or(0);
        let text_rect = Rect {
            x0: 0,
            y0: 0,
            w,
            h: h + cursor_h,
        };

        if tile_rect.intersection(&text_rect).is_none() {
            ContentId::from_hash(&(tile_rect.w, tile_rect.h, self.bg_color))
        } else {
            ContentId::from_hash(&(
                tile_rect,
                self.formatted.get_id(),
                self.cursor,
                self.cursor_visible,
                self.shadow_cursor,
                self.bg_color,
            ))
        }
    }

    fn render<F: FbViewMut>(&self, dst_fb: &mut F, tile_rect: &Rect) {
        let Rect { x0: ox, y0: oy, .. } = *tile_rect;

        dst_fb.fill(self.bg_color);

        if ox != 0 {
            return;
        }

        let mut y = 0;
        for line in self.formatted.as_ref().lines.iter() {
            let line_x0 = line.x_offset as i64;

            // Bounding box of line in source
            let line_rect = Rect {
                x0: line_x0,
                y0: y,
                w: line.w,
                h: line.h,
            };

            if tile_rect.intersection(&line_rect).is_some() {
                draw_rich_slice(dst_fb, &line.chars, line_x0, y - oy);
            }

            y += line.h as i64;
        }

        //
        // Draw blinking cursor

        let mut draw_cursor = |cursor: usize| {
            let index = self.prelude_len + cursor;
            let (x, y, h) = self.formatted.as_ref().index_to_xy(index);
            let cursor_rect = Rect {
                x0: x - ox,
                y0: y - oy,
                w: CURSOR_W,
                h,
            };
            let cursor_color = self.bg_color.invert();
            draw_rect(dst_fb, &cursor_rect, cursor_color, false);
        };

        if self.cursor_visible {
            draw_cursor(self.cursor);
        }

        if let Some(shadow_cursor) = self.shadow_cursor {
            draw_cursor(shadow_cursor);
        }
    }
}
