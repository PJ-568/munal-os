extern crate alloc;

use applib::uitk::layout::{make_grid_layout, make_horizontal_layout, make_vertical_layout, LayoutItem};
use lazy_static::lazy_static;

use applib::drawing::primitives::draw_rect;
use applib::drawing::text::{Font, RichText, TextJustification, FONT_FAMILIES};
use applib::{Color, StyleSheetText};
use core::cell::OnceCell;
use std::vec;
use guestlib::{PixelData, WasmLogger};
use applib::Rect;
use applib::content::TrackedContent;
use applib::uitk::{self, ButtonConfig, ButtonIndicatorMode, EditableRichText, TextBoxState, UuidProvider};
use applib::{Framebuffer, OwnedPixels};

const AVAILABLE_TEXT_COLORS: [Color; 10] = [
    Color::WHITE,
    Color::BLACK,
    Color::GREY,
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::YELLOW,
    Color::FUCHSIA,
    Color::AQUA,
    Color::ORANGE,
];

lazy_static! {
    pub static ref JUSTIF_LEFT_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/justif_left.png"));
    pub static ref JUSTIF_CENTER_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/justif_center.png"));
    pub static ref JUSTIF_RIGHT_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/justif_right.png"));

    pub static ref COLOR_ICONS: Vec<(Color, Framebuffer<OwnedPixels>)> = AVAILABLE_TEXT_COLORS.iter()
    .map(|&color| (color, Framebuffer::new_owned_filled(19, 16, color)))
    .collect();
}

struct AppState {
    pixel_data: PixelData,
    ui_store: uitk::UiStore,
    uuid_provider: UuidProvider,

    justification: SingleSelection<TextJustification>,
    font_family: SingleSelection<String>,
    font_size: SingleSelection<u32>,
    text_color: SingleSelection<Color>,
    bg_color: SingleSelection<Color>,

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

    let stylesheet = guestlib::get_stylesheet();

    let justification = SingleSelection(TextJustification::Left);
    let font_family_name = SingleSelection(stylesheet.text.font_family().to_owned());
    let font_size = SingleSelection(stylesheet.text.sizes.medium);
    let text_color = SingleSelection(Color::BLACK);
    let bg_color = SingleSelection(Color::WHITE);

    let textbox_state = {
        let mut tb_state = TextBoxState::new();
        tb_state.justif = *justification.selected();
        tb_state
    };

    let font_family = FONT_FAMILIES
        .get(font_family_name.selected().as_str())
        .expect("Unknown font family");

    let font = font_family.get_size(*font_size.selected());    

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
        font_family: font_family_name,
        font_size,
        text_color,
        bg_color,

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

    const TOOL_PANEL_W: u32 = 143;
    const BUTTON_H: u32 = 30;
    const SELECTION_GRID_H: u32 = 50;
    const AVAILABLE_FONT_SIZES: [u32; 6] = [12, 14, 16, 18, 20, 22];

    let state = unsafe { APP_STATE.get_mut().expect("App not initialized") };

    let time = guestlib::get_time();
    let stylesheet = guestlib::get_stylesheet();
    let input_state = guestlib::get_input_state();
    let win_rect = guestlib::get_win_rect().zero_origin();


    let mut framebuffer = state.pixel_data.get_framebuffer();

    let mut uitk_context = state.ui_store.get_context(
        &mut framebuffer,
        &stylesheet,
        &input_state,
        &mut state.uuid_provider,
        time
    );

    let available_families: Vec<&str> = FONT_FAMILIES.keys().map(|s| *s).collect();
    let n_families = available_families.len();
    let m = stylesheet.margin;

    let columns_layout = make_horizontal_layout(
        &win_rect.offset(-(m as i64)), stylesheet.margin,
        &[
            LayoutItem::Float,
            LayoutItem::Fixed { size: TOOL_PANEL_W },
        ]
    );

    let right_col_layout = make_vertical_layout(
        &columns_layout.last().unwrap(), stylesheet.margin,
        &[
            vec![
                LayoutItem::Fixed { size: BUTTON_H },
                LayoutItem::Float,
            ],
            vec![LayoutItem::Fixed { size: BUTTON_H }; n_families],
            vec![
                LayoutItem::Fixed { size: SELECTION_GRID_H },
                LayoutItem::Float,
            ],
            vec![
                LayoutItem::Fixed { size: SELECTION_GRID_H },
                LayoutItem::Float,
                LayoutItem::Fixed { size: SELECTION_GRID_H },
            ]
        ].concat()
    );

    let justif_layout = make_horizontal_layout(
        &right_col_layout[0],
        stylesheet.margin,
        &[LayoutItem::Float; 3]
    );

    let colors_layout = make_grid_layout(
        &right_col_layout[n_families + 4],
        stylesheet.margin,
        5, 2
    );

    let sizes_layout = make_grid_layout(
        &right_col_layout[n_families + 2], stylesheet.margin,
        3, 2
    );

    draw_rect(uitk_context.fb, &right_col_layout[1], stylesheet.colors.element, false);
    draw_rect(uitk_context.fb, &right_col_layout[n_families + 3], stylesheet.colors.element, false);
    draw_rect(uitk_context.fb, &right_col_layout[n_families + 5], stylesheet.colors.element, false);

    let bg_colors_layout = make_grid_layout(
        &right_col_layout[n_families + 6],
        stylesheet.margin,
        5, 2
    );

    let canvas_rect = &columns_layout[0];

    let mut button_config = ButtonConfig {
        indicator_mode: ButtonIndicatorMode::Light,
        ..Default::default()
    };

    let font_family = FONT_FAMILIES
        .get(state.font_family.selected().as_str())
        .expect("Unknown font family");

    //
    // Justification

    state.justification.scope(TextJustification::Left, |button_state| {
        button_config.rect = justif_layout[0].clone();
        button_config.icon = Some(("justif_left_icon".to_owned(), &JUSTIF_LEFT_ICON));
        uitk_context.button_toggle_once(&button_config, button_state);
    });

    state.justification.scope(TextJustification::Center, |button_state| {
        button_config.rect = justif_layout[1].clone();
        button_config.icon = Some(("justif_center_icon".to_owned(), &JUSTIF_CENTER_ICON));
        uitk_context.button_toggle_once(&button_config, button_state);
    });

    state.justification.scope(TextJustification::Right, |button_state| {
        button_config.rect = justif_layout[2].clone();
        button_config.icon = Some(("justif_right_icon".to_owned(), &JUSTIF_RIGHT_ICON));
        uitk_context.button_toggle_once(&button_config, button_state);
    });

    button_config.indicator_mode = ButtonIndicatorMode::Border;

    //
    // Text color

    for (i, (color, icon)) in COLOR_ICONS.iter().enumerate() {
        state.text_color.scope(*color, |button_state| {
            let icon_key = format!("{:?}", color);
            button_config.rect = colors_layout[i].clone();
            button_config.icon = Some((icon_key, icon));
            uitk_context.button_toggle_once(&button_config, button_state);
        });
    }

    //
    // Background color

    for (i, (color, icon)) in COLOR_ICONS.iter().enumerate() {
        state.bg_color.scope(*color, |button_state| {
            let icon_key = format!("{:?}", color);
            button_config.rect = bg_colors_layout[i].clone();
            button_config.icon = Some((icon_key, icon));
            uitk_context.button_toggle_once(&button_config, button_state);
        });
    }

    //
    // Font size

    button_config.icon = None;

    for (i, &size) in AVAILABLE_FONT_SIZES.iter().enumerate() {
        state.font_size.scope(size, |button_state| {
            button_config.rect = sizes_layout[i].clone();
            button_config.text = format!("{}", size);
            uitk_context
                .style(|ss| ss.text.sizes.medium = ss.text.sizes.small)
                .button_toggle_once(&button_config, button_state);
        });
    }

    // Font family

    button_config.indicator_mode = ButtonIndicatorMode::Light;

    for (i, &family) in available_families.iter().enumerate() {
        state.font_family.scope(family.to_owned(), |button_state| {
            button_config.rect = right_col_layout[2 + i].clone();
            button_config.text = family.to_owned();
            uitk_context
                .style(|ss| {
                    let mut text_sizes = ss.text.sizes.clone();
                    text_sizes.medium = text_sizes.small;
                    ss.text = StyleSheetText::new(family, text_sizes);
                })
                .button_toggle_once(&button_config, button_state);
        });
    }

    state.textbox_state.justif = *state.justification.selected();
    uitk_context.style(|s| s.colors.editable = *state.bg_color.selected()).editable_text_box(
        &canvas_rect,
        &mut EditableRichText {
            color: *state.text_color.selected(),
            font: font_family.get_size(*state.font_size.selected()),
            rich_text: &mut state.textbox_text
        },
        &mut state.textbox_state,
        true,
        true,
        Some(&state.textbox_prelude)
    );

}

