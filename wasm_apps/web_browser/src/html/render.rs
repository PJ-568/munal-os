use applib::drawing::primitives::draw_rect;
use applib::uitk::render_rich_text;
use applib::{FbViewMut, Rect};

use super::render_list::RenderItem;

pub fn render_html<F: FbViewMut>(dst_fb: &mut F, render_list: &[RenderItem], src_rect: &Rect) {
    // Assuming tiles are horizontal slices
    if src_rect.x0 != 0 {
        return;
    }

    for render_item in render_list.iter().rev() {
        match render_item {
            RenderItem::Text { formatted, origin } => {
                let (x0, y0) = *origin;
                let draw_rect = Rect {
                    x0,
                    y0,
                    w: formatted.w,
                    h: formatted.h,
                };

                // DEBUG
                //draw_rect_outline(dst_fb, &draw_rect, Color::RED, false, 1);

                if draw_rect.intersection(src_rect).is_some() {
                    let offset_origin = (x0 - src_rect.x0, y0 - src_rect.y0);
                    render_rich_text(dst_fb, offset_origin, formatted);
                }
            }

            RenderItem::Block { rect, color } => {
                if let Some(color) = color {
                    let offset_rect = Rect {
                        x0: rect.x0 - src_rect.x0,
                        y0: rect.y0 - src_rect.y0,
                        w: rect.w,
                        h: rect.h,
                    };
                    draw_rect(dst_fb, &offset_rect, *color, false);
                }
            }
        }
    }
}
