extern crate alloc;

use alloc::collections::BTreeMap;
use applib::uitk::layout::{make_horizontal_layout, LayoutItem};
use lazy_static::lazy_static;

use applib::drawing::primitives::draw_rect;
use applib::drawing::text::{Font, RichText, TextJustification, DEFAULT_FONT_FAMILY};
use applib::Color;
use core::cell::OnceCell;
use guestlib::{PixelData, WasmLogger};
use applib::Rect;
use applib::content::TrackedContent;
use applib::uitk::{self, ButtonConfig, ButtonIndicatorMode, EditableRichText, TextBoxState, UuidProvider};
use applib::{Framebuffer, OwnedPixels};

const AVAILABLE_TEXT_COLORS: [Color; 3] = [
    Color::BLUE,
    Color::WHITE,
    Color::RED,
];

lazy_static! {
    pub static ref JUSTIF_LEFT_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/justif_left.png"));
    pub static ref JUSTIF_CENTER_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/justif_center.png"));
    pub static ref JUSTIF_RIGHT_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/justif_right.png"));

    pub static ref COLOR_ICONS: Vec<(Color, Framebuffer<OwnedPixels>)> = AVAILABLE_TEXT_COLORS.iter()
    .map(|&color| (color, Framebuffer::new_owned_filled(20, 20, color)))
    .collect();
}

struct AppState {
    pixel_data: PixelData,
    ui_store: uitk::UiStore,
    uuid_provider: UuidProvider,

    justification: SingleSelection<TextJustification>,
    font_size: SingleSelection<u32>,
    text_color: SingleSelection<Color>,

    textbox_text: TrackedContent<RichText>,
    textbox_prelude: TrackedContent<RichText>,
    textbox_state: TextBoxState,
}

static mut APP_STATE: OnceCell<AppState> = OnceCell::new();

static LOGGER: WasmLogger = WasmLogger;
const LOGGING_LEVEL: log::LevelFilter = log::LevelFilter::Debug;

fn main() {}

#[no_mangle]
pub fn init() -> () {

    log::set_max_level(LOGGING_LEVEL);
    log::set_logger(&LOGGER).unwrap();

    let mut uuid_provider = uitk::UuidProvider::new();

    let justification = SingleSelection(TextJustification::Left);
    let font_size = SingleSelection(12);
    let text_color = SingleSelection(Color::BLACK);

    let textbox_state = {
        let mut tb_state = TextBoxState::new();
        tb_state.justif = *justification.selected();
        tb_state
    };

    let font = DEFAULT_FONT_FAMILY.get_size(*font_size.selected());    

    let textbox_text = {
        let text = RichText::from_str("pouet\ntralala", *text_color.selected(), font, None);
        TrackedContent::new(text, &mut uuid_provider)
    };

    let textbox_prelude = {
        let text = RichText::from_str("Write text here >>>", *text_color.selected(), font, None);
        TrackedContent::new(text, &mut uuid_provider)
    };


    let state = AppState {
        pixel_data: PixelData::new(),
        ui_store: uitk::UiStore::new(),
        uuid_provider: UuidProvider::new(),

        justification,
        font_size,
        text_color,

        textbox_text,
        textbox_prelude,
        textbox_state,
    };
    unsafe {
        APP_STATE
            .set(state)
            .unwrap_or_else(|_| panic!("App already initialized"));
    }
}

struct SingleSelection<T: PartialEq>(T);

impl<T: PartialEq> SingleSelection<T> {
    fn scope(&mut self, index: T, mut scope_func: impl FnMut(&mut bool)) {
        let mut state = index == self.0;
        scope_func(&mut state);
        if state {
            self.0 = index;
        }
    }

    fn selected(&self) -> &T {
        &self.0
    }
}

#[no_mangle]
pub fn step() {

    const TOOLBAR_H: u32 = 40;
    const JUSTIF_BUTTON_W: u32 = 56;
    const DEFAULT_BUTTON_W: u32 = 32;
    const CANVAS_MARGIN: u32 = 20;

    let state = unsafe { APP_STATE.get_mut().expect("App not initialized") };

    let time = guestlib::get_time();
    let stylesheet = guestlib::get_stylesheet();
    let input_state = guestlib::get_input_state();
    let Rect { w, h, ..} = guestlib::get_win_rect();


    let mut framebuffer = state.pixel_data.get_framebuffer();

    let mut uitk_context = state.ui_store.get_context(
        &mut framebuffer,
        &stylesheet,
        &input_state,
        &mut state.uuid_provider,
        time
    );

    let toolbar_rect = Rect { x0: 0, y0: 0, w, h: TOOLBAR_H };
    let canvas_rect = Rect { 
        x0: CANVAS_MARGIN as i64,
        y0: (TOOLBAR_H + CANVAS_MARGIN ) as i64,
        w: w - 2 * CANVAS_MARGIN,
        h: h - TOOLBAR_H - CANVAS_MARGIN
    };

    let available_font_sizes: Vec<u32> = DEFAULT_FONT_FAMILY.get_available_sizes().collect();

    let toolbar_layout_items = {

        let mut v = Vec::new();

        for _ in 0..3 {
            v.push(LayoutItem::Fixed { size: JUSTIF_BUTTON_W });
        }

        v.push(LayoutItem::Float);

        for _ in 0..AVAILABLE_TEXT_COLORS.len() {
            v.push(LayoutItem::Fixed { size: DEFAULT_BUTTON_W });
        }

        v.push(LayoutItem::Float);

        for _ in 0..available_font_sizes.len() {
            v.push(LayoutItem::Fixed { size: DEFAULT_BUTTON_W });
        }

        v
    };

    let toolbar_layout = make_horizontal_layout(
        &toolbar_rect.offset(-(stylesheet.margin as i64)),
        stylesheet.margin,
        &toolbar_layout_items
    );

    let mut button_config = ButtonConfig {
        indicator_mode: ButtonIndicatorMode::Light,
        ..Default::default()
    };

    let mut layout_offset = 0;

    //
    // Justification

    state.justification.scope(TextJustification::Left, |button_state| {
        button_config.rect = toolbar_layout[layout_offset].clone();
        button_config.icon = Some(("justif_left_icon".to_owned(), &JUSTIF_LEFT_ICON));
        uitk_context.button_toggle_once(&button_config, button_state);
    });

    state.justification.scope(TextJustification::Center, |button_state| {
        button_config.rect = toolbar_layout[layout_offset + 1].clone();
        button_config.icon = Some(("justif_center_icon".to_owned(), &JUSTIF_CENTER_ICON));
        uitk_context.button_toggle_once(&button_config, button_state);
    });

    state.justification.scope(TextJustification::Right, |button_state| {
        button_config.rect = toolbar_layout[layout_offset + 2].clone();
        button_config.icon = Some(("justif_right_icon".to_owned(), &JUSTIF_RIGHT_ICON));
        uitk_context.button_toggle_once(&button_config, button_state);
    });

    layout_offset += 4;
    button_config.indicator_mode = ButtonIndicatorMode::Border;

    //
    // Text color

    for (i, (color, icon)) in COLOR_ICONS.iter().enumerate() {
        state.text_color.scope(*color, |button_state| {
            let icon_key = format!("{:?}", color);
            button_config.rect = toolbar_layout[layout_offset + i].clone();
            button_config.icon = Some((icon_key, icon));
            uitk_context.button_toggle_once(&button_config, button_state);
        });
    }

    layout_offset += COLOR_ICONS.len() + 1;

    //
    // Font size

    button_config.icon = None;

    for (i, size) in DEFAULT_FONT_FAMILY.get_available_sizes().enumerate() {
        state.font_size.scope(size, |button_state| {
            button_config.rect = toolbar_layout[layout_offset + i].clone();
            button_config.text = format!("{}", size);
            uitk_context.button_toggle_once(&button_config, button_state);
        });
    }

    state.textbox_state.justif = *state.justification.selected();
    uitk_context.style(|s| s.colors.editable = Color::WHITE).editable_text_box(
        &canvas_rect,
        &mut EditableRichText {
            color: *state.text_color.selected(),
            font: DEFAULT_FONT_FAMILY.get_size(*state.font_size.selected()),
            rich_text: &mut state.textbox_text
        },
        &mut state.textbox_state,
        true,
        true,
        Some(&state.textbox_prelude)
    );
}

