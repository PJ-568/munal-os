use super::render::render_html;
use super::render_list::RenderItem;
use applib::content::{ContentId, TrackedContent};
use applib::uitk::{self, UiContext};
use applib::Color;
use applib::FbViewMut;
use applib::Rect;

pub fn html_canvas<'a, F: FbViewMut>(
    uitk_context: &mut UiContext<'a, F>,
    render_list: &'a TrackedContent<Vec<RenderItem>>,
    dst_rect: &Rect,
    offsets: &mut (i64, i64),
    dragging: &mut (bool, bool),
) -> Option<&'a str> {
    uitk_context.dynamic_canvas(dst_rect, &HtmlRenderer { render_list }, offsets, dragging);

    let UiContext {
        fb, input_state, ..
    } = uitk_context;

    let (ox, oy) = *offsets;
    let p = &input_state.pointer;
    let vr = dst_rect;

    if !dst_rect.check_contains_point(p.x, p.y) {
        return None;
    }

    let (x_p_canvas, y_p_canvas) = (p.x - vr.x0 + ox, p.y - vr.y0 + oy);

    //
    // Checking for hovered hyperlinks and adding an underline bar to them

    // The underline is not a property of RichText but an overlay drawn on top
    // (which avoids throwing out the cached canvas)

    let hovered_link = render_list.as_ref().iter().find_map(|render_item| {
        if let RenderItem::Text { formatted, origin } = render_item {
            if formatted.has_link() {
                let (x0, y0) = *origin;
                let text_rect = Rect {
                    x0,
                    y0,
                    w: formatted.w,
                    h: formatted.h,
                };
                let (x_text, y_text) = (x_p_canvas - text_rect.x0, y_p_canvas - text_rect.y0);

                let index = formatted.xy_to_index((x_text, y_text))?;

                let (link, underlines) = formatted.get_link(index)?;

                for (y_ul, x0_ul, x1_ul) in underlines.iter().cloned() {
                    let (y_ul_fb, x0_ul_fb) = (
                        y_ul + text_rect.y0 + vr.y0 - oy,
                        x0_ul + text_rect.x0 + vr.x0 - ox,
                    );
                    let line_w = (x1_ul - x0_ul + 1) as u32;
                    fb.fill_line(x0_ul_fb, line_w, y_ul_fb, Color::BLUE, false);
                }

                return Some(link);
            }
        }

        None
    });

    hovered_link
}

struct HtmlRenderer<'a> {
    render_list: &'a TrackedContent<Vec<RenderItem>>,
}

impl<'a> uitk::TileRenderer for HtmlRenderer<'a> {
    fn shape(&self) -> (u32, u32) {
        let Rect { w, h, .. } = get_render_rect(self.render_list.as_ref());
        (w, h)
    }

    fn tile_shape(&self) -> (u32, u32) {
        let Rect { w, .. } = get_render_rect(self.render_list.as_ref());
        (u32::max(w, 300), 300)
    }

    fn content_id(&self, tile_rect: &Rect) -> ContentId {
        let layout_rect = get_render_rect(self.render_list.as_ref());

        if tile_rect.intersection(&layout_rect).is_none() {
            ContentId::from_hash(&(tile_rect.w, tile_rect.h))
        } else {
            ContentId::from_hash(&(tile_rect, self.render_list.get_id()))
        }
    }

    fn render<F: FbViewMut>(&self, dst_fb: &mut F, tile_rect: &Rect) {
        // DEBUG
        // draw_rect_outline(dst_fb, tile_rect, Color::GREEN, false, 1);

        dst_fb.fill(Color::WHITE);
        render_html(dst_fb, self.render_list.as_ref(), tile_rect);
    }
}

fn get_render_rect(render_list: &[RenderItem]) -> Rect {
    let root = render_list.as_ref().last().expect("Empty render list");
    root.get_rect()
}
