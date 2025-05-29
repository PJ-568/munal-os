use applib::drawing::primitives::{draw_arc, draw_triangle, ArcMode};
use applib::geometry::{Point2D, Triangle2D};
use applib::{Color, FbViewMut};
use num_traits::Float;

const PI: f32 = 3.14159265359;

const HAND_BASE_HALF_W: f32 = 3.0;
const HAND_LEN: f32 = 60.0;

const FACE_R_INNER: f32 = 60.0;
const FACE_R_OUTER: f32 = 80.0;
const GRADUATION_ANGLE: f32 = 0.05;

const DIVIDER: f32 = 60_000.0;

pub fn draw_chrono<F: FbViewMut>(fb: &mut F, time: f64) {
    let angle = (time as f32 % DIVIDER) / DIVIDER * 2.0 * PI - PI / 2.0;

    let (fb_w, fb_h) = fb.shape();

    let center = Point2D::<f32> { x: fb_w as f32 / 2.0, y: fb_h as f32 / 2.0};

    let cos = angle.cos();
    let sin = angle.sin();

    let p0 = Point2D::<f32> {
        x: center.x + HAND_LEN * cos,
        y: center.y + HAND_LEN * sin,
    };

    let p1 = Point2D::<f32> {
        x: center.x - HAND_BASE_HALF_W * sin,
        y: center.y + HAND_BASE_HALF_W * cos,
    };

    let p2 = Point2D::<f32> {
        x: center.x + HAND_BASE_HALF_W * sin,
        y: center.y - HAND_BASE_HALF_W * cos,
    };

    let points = [p0.round_to_int(), p1.round_to_int(), p2.round_to_int()];
    let tri = Triangle2D::<i64> { points };

    draw_triangle(fb, &tri, Color::RED, false);

    let center = center.round_to_int();
    let d = GRADUATION_ANGLE / 2.0;

    draw_arc(fb, center, 2.0, 6.0, ArcMode::Full, 5.0, Color::RED, false);

    for i in 0..8 {
        
        let a = 2.0 * PI * (i as f32) / 8.0;
       
        draw_arc(
            fb, center,
            FACE_R_INNER, FACE_R_OUTER,
            ArcMode::AngleRange(a - d, a + d),
            5.0,
            Color::WHITE, false
        );
    }
}
