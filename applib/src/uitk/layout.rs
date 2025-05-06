use alloc::vec::Vec;
use num::traits::real::Real;
use crate::Rect;

pub fn make_horizontal_layout(rect: &Rect, items: &[LayoutItem]) -> Vec<Rect> {

    const MARGIN: u32 = 3;

    let total_fixed_w: u32 = items.iter()
        .map(|&it| match it {
            LayoutItem::Fixed { size } => size,
            LayoutItem::Float => 0
        })
        .sum();

    let n_float = items.iter().filter(|&it| it == &LayoutItem::Float).count();

    if n_float == 0 {
        panic!("Need at least one float element in layout!")
    }

    let total_gap_w = (items.len() as u32 + 1) * MARGIN;

    let total_float_w = rect.w - total_gap_w - total_fixed_w;

    let float_w = (total_float_w as f32) / (n_float as f32);

    let mut x1 = rect.x0 as f32;
    items.iter().map(|it| {

        x1 += MARGIN as f32;

        let size = match it {
            LayoutItem::Fixed { size } => *size as f32,
            LayoutItem::Float => float_w,
        };

        let x2 = x1 + size;

        let [_, y1, _, y2] = rect.as_xyxy();
        let item_rect = Rect::from_xyxy([
            f32::round(x1) as i64,
            y1 + MARGIN as i64,
            f32::round(x2) as i64,
            y2 - MARGIN as i64
        ]);

        x1 += size;

        item_rect

    })
    .collect()

}

pub fn make_vertical_layout(rect: &Rect, items: &[LayoutItem]) -> Vec<Rect> {
    let transposed_rect = Rect { x0: rect.y0, y0: rect.x0, w: rect.h, h: rect.w };
    let mut layout_rects = make_horizontal_layout(&transposed_rect, items);
    layout_rects.iter_mut().for_each(|r| *r = Rect { x0: r.y0, y0: r.x0, w: r.h, h: r.w });
    layout_rects
}


#[derive(Clone, Copy, PartialEq)]
pub enum LayoutItem {
    Fixed { size: u32 },
    Float,
}
