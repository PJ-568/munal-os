#![feature(vec_into_raw_parts)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use applib::content::TrackedContent;
use applib::drawing::text::{
    get_font, RichText
};
use applib::input::Keycode;
use applib::uitk::{self, UiStore, TextBoxState, EditableRichText};
use applib::{Color, Rect, StyleSheet};
use core::cell::OnceCell;
use guestlib::PixelData;
use guestlib::WasmLogger;

mod python;

#[derive(Debug)]
struct EvalResult {
    cmd: String,
    pyres: python::EvalResult,
}

static LOGGER: WasmLogger = WasmLogger;
const LOGGING_LEVEL: log::LevelFilter = log::LevelFilter::Debug;

struct AppState {
    pixel_data: PixelData,

    input_buffer: TrackedContent<RichText>,
    history: TrackedContent<Vec<EvalResult>>,

    ui_store: UiStore,
    uuid_provider: uitk::UuidProvider,
    textbox_state: TextBoxState,

    python: python::Python,
}

static mut APP_STATE: OnceCell<AppState> = OnceCell::new();

fn main() {}

#[no_mangle]
pub fn init() -> () {
    log::set_max_level(LOGGING_LEVEL);
    log::set_logger(&LOGGER).unwrap();

    let mut uuid_provider = uitk::UuidProvider::new();

    let state = AppState {
        pixel_data: PixelData::new(),
        input_buffer: TrackedContent::new(RichText::new(), &mut uuid_provider),
        history: TrackedContent::new(Vec::new(), &mut uuid_provider),
        ui_store: uitk::UiStore::new(),
        uuid_provider,
        textbox_state: TextBoxState::new(),
        python: python::Python::new(),
    };
    unsafe {
        APP_STATE
            .set(state)
            .unwrap_or_else(|_| panic!("App already initialized"))
    }
}

#[no_mangle]
pub fn step() {
    let state = unsafe { APP_STATE.get_mut().expect("App not initialized") };

    let input_state = guestlib::get_input_state();
    let mut framebuffer = state.pixel_data.get_framebuffer();

    let win_rect = guestlib::get_win_rect();
    let stylesheet = guestlib::get_stylesheet();

    let rich_text_prelude = get_rich_text_prelude(&stylesheet, &state.history);

    let time = guestlib::get_time();

    let mut uitk_context = state.ui_store.get_context(
        &mut framebuffer,
        &stylesheet,
        &input_state,
        &mut state.uuid_provider,
        time
    );

    let Rect {
        w: win_w, h: win_h, ..
    } = win_rect;

    let rect_console = Rect {
        x0: 0,
        y0: 0,
        w: win_w,
        h: win_h,
    };

    let font = get_font(
        &stylesheet.text.font_family(),
        stylesheet.text.sizes.small
    );

    let mut editable = EditableRichText { 
        rich_text: &mut state.input_buffer,
        font,
        color: Color::WHITE
    };

    uitk_context.editable_text_box(
        &rect_console,
        &mut editable,
        &mut state.textbox_state,
        true,
        false,
        Some(&rich_text_prelude),
    );

    if input_state.check_key_pressed(Keycode::KEY_ENTER) && !state.input_buffer.as_ref().is_empty() {
        let cmd = state.input_buffer.as_ref().as_string();
        let pyres = state.python.run_code(&cmd);
        state
            .history
            .mutate(&mut state.uuid_provider)
            .push(EvalResult { cmd, pyres });
        state.input_buffer.mutate(&mut state.uuid_provider).clear();
    }
}

fn get_rich_text_prelude(
    stylesheet: &StyleSheet,
    history: &TrackedContent<Vec<EvalResult>>,
) -> TrackedContent<RichText> {

    let font = get_font(
        &stylesheet.text.font_family(),
        stylesheet.text.sizes.small
    );

    let mut rich_text = RichText::new();

    for res in history.as_ref().iter() {
        rich_text.add_part(">>> ", stylesheet.colors.yellow, font, None);
        rich_text.add_part(&res.cmd, stylesheet.colors.text, font, None);

        let color = match &res.pyres {
            python::EvalResult::Failure(_) => Color::rgb(200, 150, 25),
            _ => stylesheet.colors.text,
        };

        let text = match &res.pyres {
            python::EvalResult::Failure(err) => format!("\n{}", err),
            python::EvalResult::Success(repr) => format!("\n{}", repr),
        };

        rich_text.add_part(&text, color, font, None);
        rich_text.add_part("\n", stylesheet.colors.text, font, None)
    }

    rich_text.add_part(">>> ", stylesheet.colors.text, font, None);

    TrackedContent::new_with_id(rich_text, history.get_id())
}
