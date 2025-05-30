use applib::drawing::primitives::{draw_arc, draw_triangle, ArcMode};
use applib::geometry::{Point2D, Triangle2D};
use applib::{Color, FbViewMut};
use num_traits::Float;

const PI: f32 = 3.14159265359;

const HAND_BASE_HALF_W: f32 = 6.0;
const HAND_LEN: f32 = 70.0;

const FACE_R_INNER: f32 = 70.0;

const BIG_GRAD_R_OUTER: f32 = 100.0;
const BIG_GRAD_ANGLE: f32 = 0.05;
const SMALL_GRAD_R_OUTER: f32 = 90.0;
const SMALL_GRAD_ANGLE: f32 = 0.02;

const N_GRAD: usize = 16;

const DIVIDER: f32 = 60_000.0;

const WINDOW_SCALE_FACTOR: f32 = 250.0;


pub fn draw_background<F: FbViewMut>(fb: &mut F) {

    let (fb_w, fb_h) = fb.shape();
    let win_scale = (u32::min(fb_w, fb_h) as f32) / WINDOW_SCALE_FACTOR;
    let center = Point2D::<f32> { x: fb_w as f32 / 2.0, y: fb_h as f32 / 2.0};

    let center = center.round_to_int();

    draw_arc(fb, center, 2.0, 7.0, ArcMode::Full, 5.0, Color::RED, false);

    for i in 0..N_GRAD {
        
        let a = 2.0 * PI * (i as f32) / (N_GRAD as f32);

        let (a_delta, r_outer) = match i % 2 == 0 {
            true => (BIG_GRAD_ANGLE, BIG_GRAD_R_OUTER),
            false => (SMALL_GRAD_ANGLE, SMALL_GRAD_R_OUTER),
        };

        let half_delta = a_delta / 2.0;
       
        draw_arc(
            fb, center,
            win_scale * FACE_R_INNER, win_scale * r_outer,
            ArcMode::AngleRange(a - half_delta, a + half_delta),
            100.0,
            Color::WHITE, false
        );
    }
}


pub fn draw_chrono<F: FbViewMut>(fb: &mut F, time: f64) {
    let angle = (time as f32 % DIVIDER) / DIVIDER * 2.0 * PI - PI / 2.0;

    let (fb_w, fb_h) = fb.shape();

    let win_scale = (u32::min(fb_w, fb_h) as f32) / WINDOW_SCALE_FACTOR;

    let center = Point2D::<f32> { x: fb_w as f32 / 2.0, y: fb_h as f32 / 2.0};

    let cos = angle.cos();
    let sin = angle.sin();

    let p0 = Point2D::<f32> {
        x: center.x + win_scale * HAND_LEN * cos,
        y: center.y + win_scale * HAND_LEN * sin,
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
}
