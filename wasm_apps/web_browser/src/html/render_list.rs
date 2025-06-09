use super::block_layout::{Block, Orientation};
use super::tree::{NodeId, Tree};
use applib::drawing::text::{format_rich_lines, FormattedRichText, TextJustification};
use applib::{Color, Rect};

pub fn compute_render_list(block_tree: &Tree<Block>, canvas_w: u32) -> Vec<RenderItem> {
    let mut render_list = Vec::new();

    fn process_node(
        render_list: &mut Vec<RenderItem>,
        block_tree: &Tree<Block>,
        node_id: NodeId,
        canvas_w: u32,
        origin: (i64, i64),
    ) -> (u32, u32) {
        const MIN_TEXT_W: u32 = 20;
        const MARGIN: u32 = 2;

        let node = block_tree.get_node(node_id).unwrap();
        let (x0, y0) = origin;

        let (w, h) = match &node.data {
            Block::Text { text } => {
                let text_w = i64::max(MIN_TEXT_W as i64, canvas_w as i64 - x0) as u32;
                let formatted = format_rich_lines(text, text_w, TextJustification::Left);

                let text_h = formatted.h;
                let max_line_w = formatted.lines.iter().map(|line| line.w).max().unwrap_or(0);

                render_list.push(RenderItem::Text { formatted, origin });

                (max_line_w, text_h)
            }

            Block::Container {
                color,
                orientation,
                has_margin,
            } => {
                let (mut container_w, mut container_h) = (0, 0);

                let margin = if *has_margin { MARGIN } else { 0 };
                let (x0, y0) = (x0 + margin as i64, y0 + margin as i64);

                match orientation {
                    Orientation::Vertical => {
                        let mut y = y0;
                        for child_id in node.children.iter() {
                            let (child_w, child_h) =
                                process_node(render_list, block_tree, *child_id, canvas_w, (x0, y));
                            y += child_h as i64;
                            container_w = u32::max(container_w, child_w);
                            container_h += child_h;
                        }
                    }

                    Orientation::Horizontal => {
                        let mut x = x0;

                        for child_id in node.children.iter() {
                            let (child_w, child_h) =
                                process_node(render_list, block_tree, *child_id, canvas_w, (x, y0));
                            x += child_w as i64;
                            container_h = u32::max(container_h, child_h);
                            container_w += child_w;
                        }
                    }
                }

                render_list.push(RenderItem::Block {
                    rect: Rect {
                        x0,
                        y0,
                        w: container_w,
                        h: container_h,
                    },
                    color: *color,
                });

                (container_w + margin, container_h + margin)
            }

            // We don't support actually rendering the images yet
            Block::Image { w, h } => (*w, *h),
        };

        (w, h)
    }

    process_node(&mut render_list, block_tree, NodeId(0), canvas_w, (0, 0));

    render_list
}

#[derive(Debug)]
pub enum RenderItem {
    Block {
        rect: Rect,
        color: Option<Color>,
    },
    Text {
        formatted: FormattedRichText,
        origin: (i64, i64),
    },
}

impl RenderItem {
    pub fn get_rect(&self) -> Rect {
        match self {
            Self::Block { rect, .. } => rect.clone(),
            Self::Text { formatted, origin } => {
                let (x0, y0) = *origin;
                Rect {
                    x0,
                    y0,
                    w: formatted.w,
                    h: formatted.h,
                }
            }
        }
    }
}
