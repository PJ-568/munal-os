extern crate alloc;

use alloc::format;
use applib::drawing::primitives::draw_rect;
use lazy_static::lazy_static;
use applib::drawing::text::{draw_line_in_rect, draw_str, TextJustification, DEFAULT_FONT_FAMILY};
use applib::{Color, FbViewMut};
use applib::{Framebuffer, OwnedPixels};
use applib::uitk::{self, ButtonConfig, UuidProvider, TextBoxState};
use applib::uitk::layout::{make_horizontal_layout, make_vertical_layout, LayoutItem};
use core::cell::OnceCell;
use guestlib::{PixelData, WasmLogger};

mod drawing;
use drawing::draw_chrono;

static LOGGER: WasmLogger = WasmLogger;
const LOGGING_LEVEL: log::LevelFilter = log::LevelFilter::Debug;

lazy_static! {
    pub static ref PLAY_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/play.png"));
    pub static ref PAUSE_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/pause.png"));
    pub static ref STOP_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/stop.png"));
}



struct AppState {
    pixel_data: PixelData,
    chrono_state: ChronoState,

    uuid_provider: UuidProvider,
    ui_store: uitk::UiStore,
}

#[derive(Debug)]
enum ChronoState {
    Stopped,
    Paused { t_elapsed: f64 },
    Running { t_resume: f64, t_offset: f64 },
}

static mut APP_STATE: OnceCell<AppState> = OnceCell::new();

fn main() {}

#[no_mangle]
pub fn init() -> () {

    log::set_max_level(LOGGING_LEVEL);
    log::set_logger(&LOGGER).unwrap();

    let state = AppState {
        pixel_data: PixelData::new(),
        chrono_state: ChronoState::Stopped,

        ui_store: uitk::UiStore::new(),
        uuid_provider: UuidProvider::new(),
    };
    unsafe {
        APP_STATE
            .set(state)
            .unwrap_or_else(|_| panic!("App already initialized"));
    }
}

#[no_mangle]
pub fn step() {

    const BUTTON_W: u32 = 32;

    let state = unsafe { APP_STATE.get_mut().expect("App not initialized") };

    let input_state = guestlib::get_input_state();
    let win_rect = guestlib::get_win_rect().zero_origin();
    let t_now = guestlib::get_time();
    let stylesheet = guestlib::get_stylesheet();
    let mut framebuffer = state.pixel_data.get_framebuffer();

    let AppState {
        ui_store,
        uuid_provider,
        ..
    } = state;

    framebuffer.fill(Color::BLACK);

    let mut uitk_context = ui_store.get_context(
        &mut framebuffer,
        &stylesheet,
        &input_state,
        uuid_provider,
        t_now
    );

    let layout_1 = make_vertical_layout(
        &win_rect.offset(-(stylesheet.margin as i64)),
        stylesheet.margin,
        &[
            LayoutItem::Fixed { size: BUTTON_W },
            LayoutItem::Float,
        ]
    );

    let layout_2 = make_horizontal_layout(
        &layout_1[0],
        stylesheet.margin,
        &[
            LayoutItem::Float,
            LayoutItem::Fixed { size: BUTTON_W },
            LayoutItem::Fixed { size: BUTTON_W },
        ]
    );

    let font = DEFAULT_FONT_FAMILY.get_size(stylesheet.text_sizes.medium);

    let elapsed = match state.chrono_state {
        ChronoState::Stopped => 0.0,
        ChronoState::Paused { t_elapsed } => t_elapsed,
        ChronoState::Running { t_resume, t_offset } => t_now - t_resume + t_offset,
    };

    let time_str = {

        let t_ms = f64::round(elapsed) as u32;
        let disp_ms = t_ms % 1000;

        let t_s = t_ms / 1000;
        let disp_s = t_s % 60;

        let t_min = t_s / 60;
        let disp_min = t_min % 60;

        format!("{:02}:{:02}.{:03}", disp_min, disp_s, disp_ms)
    };

    draw_line_in_rect(
        uitk_context.fb,
        &time_str,
        &layout_2[0],
        font, Color::YELLOW, TextJustification::Left
    );

    match state.chrono_state {
        ChronoState::Stopped => {
            let play_pressed = uitk_context.button(&ButtonConfig { 
                rect: layout_2[2].clone(),
                icon: Some(("play_icon".to_owned(), &PLAY_ICON)),
                ..Default::default()
            });

            if play_pressed {
                state.chrono_state = ChronoState::Running { t_resume: t_now, t_offset: 0.0 };
            }
        },

        ChronoState::Paused { t_elapsed } => {

            let stop_pressed = uitk_context.button(&ButtonConfig { 
                rect: layout_2[1].clone(),
                icon: Some(("stop_icon".to_owned(), &STOP_ICON)),
                ..Default::default()
            });

            let play_pressed = uitk_context.button(&ButtonConfig { 
                rect: layout_2[2].clone(),
                icon: Some(("play_icon".to_owned(), &PLAY_ICON)),
                ..Default::default()
            });

            if stop_pressed {
                state.chrono_state = ChronoState::Stopped;
            } else if play_pressed {
                state.chrono_state = ChronoState::Running { t_resume: t_now, t_offset: t_elapsed };
            }

        }

        ChronoState::Running { t_resume, t_offset } => {

            let stop_pressed = uitk_context.button(&ButtonConfig { 
                rect: layout_2[1].clone(),
                icon: Some(("stop_icon".to_owned(), &STOP_ICON)),
                ..Default::default()
            });

            let pause_pressed = uitk_context.button(&ButtonConfig { 
                rect: layout_2[2].clone(),
                icon: Some(("pause_icon".to_owned(), &PAUSE_ICON)),
                ..Default::default()
            });

            if stop_pressed {
                state.chrono_state = ChronoState::Stopped;
            } else if pause_pressed {
                state.chrono_state = ChronoState::Paused { t_elapsed: t_now - t_resume + t_offset };
            }

        }
    };

    let mut canvas_fb = framebuffer.subregion_mut(&layout_1[1]);

    draw_chrono(&mut canvas_fb, elapsed);
}
