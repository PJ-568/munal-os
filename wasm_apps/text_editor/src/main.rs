extern crate alloc;

use applib::uitk::layout::{make_grid_layout, make_horizontal_layout, make_vertical_layout, LayoutItem};
use lazy_static::lazy_static;

use applib::drawing::primitives::draw_rect;
use applib::drawing::text::{draw_line_in_rect, get_font, RichText, TextJustification, FONT_FAMILIES};
use applib::{Color, StyleSheetText};
use core::cell::OnceCell;
use std::vec;
use guestlib::{PixelData, WasmLogger};
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

const INTRO_COLOR: Color = Color::BLACK;
const INTRO_TITLE_TEXT: &'static str = "Demo text editor\n";
const INTRO_TITLE_SIZE: u32 = 22;
const INTRO_BODY_TEXT: &'static str = "You can change text justification, font, size and colors on the right.
Left/right arrow keys or left click to change the cursor position.
";
const INTRO_BODY_SIZE: u32 = 16;

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

    let justification = SingleSelection(TextJustification::Center);
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

    let textbox_text = {
        let mut text = RichText::new();
        text.add_part(INTRO_TITLE_TEXT, INTRO_COLOR, font_family.get_size(INTRO_TITLE_SIZE), None);
        text.add_part(INTRO_BODY_TEXT, INTRO_COLOR, font_family.get_size(INTRO_BODY_SIZE), None);
        for _ in 0..POEM_SPACING {
            text.add_part("\n", INTRO_COLOR, font_family.get_size(INTRO_BODY_SIZE), None);
        }
        let poem_font_family = FONT_FAMILIES
            .get(POEM_FONT)
            .expect("Unknown font family");
        text.add_part(POEM_TEXT, POEM_COLOR, poem_font_family.get_size(18), None);
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
    const SECTION_TITLE_H: u32 = 18;
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
                LayoutItem::Fixed { size: SECTION_TITLE_H },
                LayoutItem::Fixed { size: SELECTION_GRID_H },
                LayoutItem::Float,
                LayoutItem::Fixed { size: SECTION_TITLE_H },
                LayoutItem::Fixed { size: SELECTION_GRID_H },
            ]
        ].concat()
    );

    let font_family = FONT_FAMILIES
        .get(state.font_family.selected().as_str())
        .expect("Unknown font family");

    let ui_font = get_font(stylesheet.text.font_family(), stylesheet.text.sizes.small);
    let ui_text_color = stylesheet.colors.text;

    let mut layout_offset = 0;

    //
    // Justification

    let justif_layout = make_horizontal_layout(
        &right_col_layout[layout_offset],
        stylesheet.margin,
        &[LayoutItem::Float; 3]
    );

    let mut button_config = ButtonConfig {
        indicator_mode: ButtonIndicatorMode::Light,
        ..Default::default()
    };

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

    layout_offset += 1;

    draw_rect(uitk_context.fb, &right_col_layout[layout_offset], stylesheet.colors.element, false);

    layout_offset += 1;


    // Font family

    let mut button_config = ButtonConfig {
        indicator_mode: ButtonIndicatorMode::Light,
        ..Default::default()
    };

    for (i, &family) in available_families.iter().enumerate() {
        state.font_family.scope(family.to_owned(), |button_state| {
            button_config.rect = right_col_layout[layout_offset + i].clone();
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

    layout_offset += n_families;

    //
    // Font size

    let mut button_config = ButtonConfig {
        indicator_mode: ButtonIndicatorMode::Border,
        ..Default::default()
    };

    let sizes_layout = make_grid_layout(
        &right_col_layout[layout_offset], stylesheet.margin,
        3, 2
    );

    for (i, &size) in AVAILABLE_FONT_SIZES.iter().enumerate() {
        state.font_size.scope(size, |button_state| {
            button_config.rect = sizes_layout[i].clone();
            button_config.text = format!("{}", size);
            uitk_context
                .style(|ss| ss.text.sizes.medium = ss.text.sizes.small)
                .button_toggle_once(&button_config, button_state);
        });
    }

    layout_offset += 1;

    draw_rect(uitk_context.fb, &right_col_layout[layout_offset], stylesheet.colors.element, false);

    layout_offset += 1;

    //
    // Text color

    let mut button_config = ButtonConfig {
        indicator_mode: ButtonIndicatorMode::Border,
        ..Default::default()
    };

    draw_rect(uitk_context.fb, &right_col_layout[layout_offset], stylesheet.colors.element, false);
    draw_line_in_rect(
        uitk_context.fb, "Foreground", &right_col_layout[layout_offset],
        ui_font, ui_text_color, TextJustification::Left
    );

    layout_offset += 1;

    let colors_layout = make_grid_layout(
        &right_col_layout[layout_offset],
        stylesheet.margin,
        5, 2
    );

    for (i, (color, icon)) in COLOR_ICONS.iter().enumerate() {
        state.text_color.scope(*color, |button_state| {
            let icon_key = format!("{:?}", color);
            button_config.rect = colors_layout[i].clone();
            button_config.icon = Some((icon_key, icon));
            uitk_context.button_toggle_once(&button_config, button_state);
        });
    }

    layout_offset += 1;

    draw_rect(uitk_context.fb, &right_col_layout[layout_offset], stylesheet.colors.element, false);

    layout_offset += 1;

    //
    // Background color

    let mut button_config = ButtonConfig {
        indicator_mode: ButtonIndicatorMode::Border,
        ..Default::default()
    };

    draw_rect(uitk_context.fb, &right_col_layout[layout_offset], stylesheet.colors.element, false);
    draw_line_in_rect(
        uitk_context.fb, "Background", &right_col_layout[layout_offset],
        ui_font, ui_text_color, TextJustification::Left
    );

    layout_offset += 1;

    let bg_colors_layout = make_grid_layout(
        &right_col_layout[layout_offset],
        stylesheet.margin,
        5, 2
    );

    for (i, (color, icon)) in COLOR_ICONS.iter().enumerate() {
        state.bg_color.scope(*color, |button_state| {
            let icon_key = format!("{:?}", color);
            button_config.rect = bg_colors_layout[i].clone();
            button_config.icon = Some((icon_key, icon));
            uitk_context.button_toggle_once(&button_config, button_state);
        });
    }

    //
    // Canvas

    let canvas_rect = &columns_layout[0];

    state.textbox_state.justif = *state.justification.selected();
    uitk_context.style(|s| s.colors.editable = *state.bg_color.selected()).editable_text_box(
        &canvas_rect,
        &mut EditableRichText {
            color: *state.text_color.selected(),
            font: font_family.get_size(*state.font_size.selected()),
            rich_text: &mut state.textbox_text
        },
        &mut state.textbox_state,
        false,
        true,
        None::<&EditableRichText>
    );

}

const POEM_TEXT: &'static str = "Across old bark
It's always dark
In the ancient glade
The quiet shade";
const POEM_COLOR: Color = Color::BLUE;
const POEM_SPACING: usize = 30;
const POEM_FONT: &'static str = "XanMono";
