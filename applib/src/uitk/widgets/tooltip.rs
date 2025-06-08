use crate::drawing::primitives::draw_rect;
use crate::drawing::text::{compute_text_bbox, draw_line_in_rect, get_font, TextJustification};
use crate::uitk::{UiContext};
use crate::{FbViewMut, Rect};

impl<'a, F: FbViewMut> UiContext<'a, F> {

    pub fn tooltip(&mut self, trigger: &Rect, offset: (i64, i64), text: &str) {

        const MARGIN: u32 = 10;

        let px = self.input_state.pointer.x;
        let py = self.input_state.pointer.y;

        if trigger.check_contains_point(px, py) {

            let font = get_font(
                &self.stylesheet.text.font_family(),
                self.stylesheet.text.sizes.medium,
            );

            let color = self.stylesheet.colors.text;

            let (dx, dy) = offset;
            let (cx, cy) = trigger.center();

            let (text_w, text_h) = compute_text_bbox(text, font);

            let rect = Rect::from_center(cx + dx, cy + dy, text_w + MARGIN, text_h + MARGIN);
    
            draw_rect(self.fb, &rect, self.stylesheet.colors.element, false);
            draw_line_in_rect(self.fb, text, &rect, font, color, TextJustification::Center);
        }
    }
}
