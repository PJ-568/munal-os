use crate::drawing::primitives::draw_rect;
use crate::drawing::text::{draw_line_in_rect, TextJustification};
use crate::uitk::{UiContext};
use crate::{FbViewMut, Rect};

impl<'a, F: FbViewMut> UiContext<'a, F> {

    pub fn tooltip(&mut self, trigger: &Rect, offset: (i64, i64), text: &str) {

        let px = self.input_state.pointer.x;
        let py = self.input_state.pointer.y;

        if trigger.check_contains_point(px, py) {

            let font = self.font_family.get_size(self.stylesheet.text_sizes.medium);
            let color = self.stylesheet.colors.text;

            let (dx, dy) = offset;
            let rect = Rect { 
                x0: trigger.x0 + dx, y0: trigger.y0 + dy,
                w: trigger.w, h: trigger.h,
            };
    
            draw_rect(self.fb, &rect, self.stylesheet.colors.element, false);
            draw_line_in_rect(self.fb, text, &rect, font, color, TextJustification::Center);
        }
    }
}
