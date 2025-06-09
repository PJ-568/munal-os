use crate::{blend_colors, decode_png, Color, FbView, FbViewMut, Rect};
use alloc::borrow::ToOwned;
use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use serde::Deserialize;

use super::primitives::draw_rect;
use crate::hash::compute_hash;

#[derive(Deserialize)]
struct FontSpec {
    size: usize,
    nb_chars: usize,
    char_h: usize,
    char_w: usize,
    base_y: usize,
}

struct FontData {
    bitmap_png_bytes: &'static [u8],
    spec_json_bytes: &'static [u8],
}

fn load_font(family_name: &str, data: &FontData) -> Font {
    let spec = serde_json::from_slice(data.spec_json_bytes).expect("Invalid font spec data");
    let bitmap = decode_png(data.bitmap_png_bytes);

    let FontSpec {
        size,
        nb_chars,
        char_h,
        char_w,
        base_y,
    } = spec;

    if bitmap.len() != nb_chars * char_w * char_h {
        panic!("Invalid font bitmap size");
    }

    Font {
        name: format!("{}-{}", family_name, size),
        size,
        bitmap,
        nb_chars,
        char_h,
        char_w,
        base_y,
    }
}

pub struct FontFamily {
    pub name: &'static str,
    by_size: BTreeMap<u32, Font>,
}

impl FontFamily {
    fn from_font_data(family_name: &'static str, fonts: &[FontData]) -> Self {
        let by_size = fonts
            .iter()
            .map(|data| {
                let font = load_font(family_name, data);
                (font.size as u32, font)
            })
            .collect();

        FontFamily {
            name: family_name,
            by_size,
        }
    }

    pub fn get_size(&self, size: u32) -> &Font {
        self.by_size
            .get(&size)
            .expect("No font available for this size")
    }

    pub fn get_available_sizes(&self) -> impl Iterator<Item = u32> + use<'_> {
        self.by_size.keys().map(|k| *k)
    }
}

pub struct Font {
    pub name: String,
    pub size: usize,
    bitmap: Vec<u8>,
    pub nb_chars: usize,
    pub char_h: usize,
    pub char_w: usize,
    pub base_y: usize,
}

macro_rules! font_data {
    ($name: expr, $($sizes:tt)*) => {
        FontFamily::from_font_data($name, &[$(
            FontData {
                bitmap_png_bytes: include_bytes!(concat!(
                    "../../fonts/", $name, "/", $sizes, "/bitmap.png"
                )),
                spec_json_bytes: include_bytes!(concat!(
                    "../../fonts/", $name, "/", $sizes, "/spec.json"
                )),
            },
        )*])
    };
}

lazy_static! {
    pub static ref FONT_FAMILIES: BTreeMap<&'static str, FontFamily> = [
        font_data!("NotoSansMono", 12 14 16 18 20 22),
        font_data!("XanMono", 12 14 16 18 20 22),
        font_data!("Libertinus", 12 14 16 18 20 22),
        font_data!("MajorMono", 12 14 16 18 20 22),
    ]
    .into_iter()
    .map(|family| (family.name, family))
    .collect();
}

pub fn get_font(family_name: &str, size: u32) -> &'static Font {
    let font_family = FONT_FAMILIES
        .get(family_name)
        .expect(&format!("Unknown font family {}", family_name));

    font_family.get_size(size)
}

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum TextJustification {
    Left,
    Center,
    Right,
}

pub fn draw_line_in_rect<F: FbViewMut>(
    fb: &mut F,
    s: &str,
    rect: &Rect,
    font: &Font,
    color: Color,
    justif: TextJustification,
) -> (i64, i64) {
    let text_h = font.char_h as u32;
    let text_w = (font.char_w * s.len()) as i64;
    let (xc, yc) = rect.center();

    let pad_y = i64::max(0, rect.h as i64 - text_h as i64) / 2;

    let pad_x = pad_y + (font.char_h - font.base_y) as i64;

    let text_x0 = match justif {
        TextJustification::Left => rect.x0 + pad_x,
        TextJustification::Center => xc - text_w / 2,
        TextJustification::Right => rect.x0 + rect.w as i64 - text_w - pad_x,
    };

    let text_y0 = rect.y0 + pad_y;

    draw_str(fb, s, text_x0, text_y0, font, color, None);

    let text_x1 = text_x0 + (font.char_w * s.len()) as i64;

    (text_x0, text_x1)
}

pub fn draw_str<F: FbViewMut>(
    fb: &mut F,
    s: &str,
    x0: i64,
    y0: i64,
    font: &Font,
    color: Color,
    bg_color: Option<Color>,
) {
    if let Some(bg_color) = bg_color {
        let text_w = (font.char_w * s.len()) as u32;
        let rect = Rect {
            x0,
            y0,
            w: text_w,
            h: font.char_h as u32,
        };
        draw_rect(fb, &rect, bg_color, true);
    }

    let mut x = x0;
    for c in s.chars() {
        draw_char(fb, c, x, y0, font, color, true);
        x += font.char_w as i64;
    }
}

pub fn draw_char<F: FbViewMut>(
    fb: &mut F,
    c: char,
    x0: i64,
    y0: i64,
    font: &Font,
    color: Color,
    blend: bool,
) {
    let mut c = c as u8;

    // Replacing unsupported chars with spaces
    if c < 32 || c > 126 {
        c = 32
    }

    let c_index = (c - 32) as usize;
    let Font {
        nb_chars,
        char_h,
        char_w,
        ..
    } = *font;
    let (r, g, b, _a) = color.as_rgba();

    let char_rect = Rect {
        x0,
        y0,
        w: char_w as u32,
        h: char_h as u32,
    };

    let [xc0, yc0, xc1, yc1] = char_rect.as_xyxy();

    for x in xc0..=xc1 {
        for y in yc0..=yc1 {
            let i_font =
                (y - yc0) as usize * char_w * nb_chars + (x - xc0) as usize + c_index * char_w;
            let val_font = font.bitmap[i_font];
            let is_in_font = val_font > 0;

            if !is_in_font {
                continue;
            }

            if let Some(curr_color) = fb.get_pixel(x, y) {
                let txt_color = Color::rgba(r, g, b, val_font);
                let new_color = match blend {
                    true => blend_colors(txt_color, curr_color),
                    false => txt_color,
                };
                fb.set_pixel(x, y, new_color);
            }
        }
    }
}

#[derive(Clone)]
pub struct RichText {
    chars: Vec<RichChar>,
    link_store: LinkStore,
}

type LinkStore = BTreeMap<LinkId, (usize, String)>;

impl alloc::fmt::Debug for RichText {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        let links: Vec<String> = self.link_store.values().map(|(_, s)| s.clone()).collect();
        let joined = links.join(" / ");

        write!(f, "RichText [{}] links: {}", self.as_string(), joined)
    }
}

impl RichText {
    pub fn new() -> Self {
        RichText {
            chars: Vec::new(),
            link_store: BTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn clear(&mut self) {
        self.chars.clear();
    }

    pub fn add_part(&mut self, s: &str, color: Color, font: &'static Font, link: Option<&str>) {
        // Adding link to store
        let link_id = link.map(|link| {
            let link_id = LinkId(compute_hash(link));
            self.link_store
                .entry(link_id)
                .and_modify(|(count, _)| *count += s.len())
                .or_insert((s.len(), link.to_owned()));
            link_id
        });

        self.chars.extend(s.chars().map(|c| RichChar {
            c,
            color,
            font,
            link_id,
        }));
    }

    pub fn insert(&mut self, pos: usize, c: char, color: Color, font: &'static Font) {
        self.chars.insert(
            pos,
            RichChar {
                c,
                color,
                font,
                link_id: None,
            },
        );
    }

    pub fn remove(&mut self, pos: usize) {
        // Decrementing counter in link store
        if let Some(link_id) = self.chars[pos].link_id {
            let (counter, _) = self.link_store.get_mut(&link_id).unwrap();
            *counter -= 1;
            if *counter == 0 {
                // If no more references to that link, delete
                self.link_store.remove(&link_id);
            }
        }

        self.chars.remove(pos);
    }

    pub fn from_str(s: &str, color: Color, font: &'static Font, link: Option<&str>) -> Self {
        let mut t = Self::new();
        t.add_part(s, color, font, link);
        t
    }

    pub fn as_string(&self) -> String {
        let mut s = String::new();
        for rich_char in self.chars.iter() {
            s.push(rich_char.c);
        }
        s
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn concat(&mut self, mut other: Self) {
        for (link_id, (other_count, other_link)) in other.link_store.into_iter() {
            self.link_store
                .entry(link_id)
                .and_modify(|(count, _)| *count += other_count)
                .or_insert((other_count, other_link));
        }

        self.chars.append(&mut other.chars);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LinkId(u64);

#[derive(Clone)]
pub struct RichChar {
    pub c: char,
    pub color: Color,
    pub font: &'static Font,
    link_id: Option<LinkId>,
}

impl RichChar {
    fn width(&self) -> u32 {
        if self.c == '\n' {
            0
        } else {
            self.font.char_w as u32
        }
    }

    fn height(&self) -> u32 {
        self.font.char_h as u32
    }
}

pub struct FormattedRichLine {
    pub chars: Vec<RichChar>,
    pub w: u32,
    pub h: u32,
    pub base_y: u32,
    pub x_offset: u32,
}
pub struct FormattedRichText {
    pub lines: Vec<FormattedRichLine>,
    pub w: u32,
    pub h: u32,
    justif: TextJustification,
    link_store: LinkStore,
}

impl FormattedRichText {
    pub fn index_to_xy(&self, index: usize) -> (i64, i64, u32) {
        if self.lines.is_empty() {
            return (0, 0, 0);
        }

        // OK because we checked lines was not empty
        let last_line = self.lines.last().unwrap();

        // OK because a single line cannot be empty (contains at least a \n)
        let last_char = last_line.chars.last().unwrap().c;

        let mut i = 0;
        let search_res = self.lines.iter().enumerate().find_map(|(line_i, line)| {
            if index < i + line.chars.len() {
                Some((line_i, index - i))
            } else {
                i += line.chars.len();
                None
            }
        });

        let get_line_pos = |line_i: usize, line_char_i: usize| -> (u32, u32, u32) {
            let line = &self.lines[line_i];
            let x_offset = line.x_offset;
            let left_chars = &line.chars[0..line_char_i];
            let x = left_chars.iter().map(|c| c.font.char_w as u32).sum::<u32>();
            let y = self.lines[..line_i].iter().map(|l| l.h).sum::<u32>();

            let ref_char = if line_char_i == 0 {
                &line.chars[0]
            } else {
                &line.chars[line_char_i - 1]
            };

            let char_h = ref_char.font.char_h as u32;
            let base_y = ref_char.font.base_y as u32;

            let y_offset = line.base_y - base_y;

            (x + x_offset, y + y_offset, char_h)
        };

        let (x, y, h) = match search_res {
            Some((line_i, line_char_i)) => get_line_pos(line_i, line_char_i),

            // Cursor at the end of the text, but just after a newline
            None if last_char == '\n' => {
                let y = self.lines.iter().map(|l| l.h).sum::<u32>();
                let x = match self.justif {
                    TextJustification::Left => 0,
                    TextJustification::Center => self.w / 2,
                    TextJustification::Right => self.w,
                };
                let line_h = self.lines.last().unwrap().h;
                (x, y, line_h)
            }

            // Cursor at the end of the text
            None => {
                let line_i = self.lines.len() - 1;
                let line_char_i = last_line.chars.len();
                get_line_pos(line_i, line_char_i)
            }
        };

        (x as i64, y as i64, h)
    }

    pub fn xy_to_index(&self, xy: (i64, i64)) -> Option<usize> {
        let (xp, yp) = xy;
        if xp < 0 || xp >= self.w as i64 || yp < 0 || yp >= self.h as i64 {
            return None;
        }

        let mut index = 0;
        let mut y = 0;
        for line in self.lines.iter() {
            let line_rect = Rect {
                x0: line.x_offset as i64,
                y0: y,
                w: line.w,
                h: line.h,
            };

            if line_rect.check_contains_point(xp, yp) {
                let mut x = line_rect.x0;

                let index = line
                    .chars
                    .iter()
                    .enumerate()
                    .find_map(|(i, c)| {
                        let char_w = c.font.char_w as i64;
                        if xp <= x + char_w {
                            return Some(index + i);
                        }
                        x += char_w;
                        None
                    })
                    .expect("Could not find rich line index (should not happen)");

                return Some(index);
            }

            y += line.h as i64;
            index += line.chars.len();
        }

        None
    }

    pub fn has_link(&self) -> bool {
        !self.link_store.is_empty()
    }

    pub fn get_link(&self, index: usize) -> Option<(&str, Vec<(i64, i64, i64)>)> {
        const UNDERLINE_GAP: u32 = 2;

        let rc = self.get_char(index);
        let link_id = rc.link_id?;

        let link_str = self
            .link_store
            .get(&link_id)
            .map(|(_, link)| link.as_str())?;

        let mut underlines = Vec::new();

        let mut y = 0;
        for line in self.lines.iter() {
            let mut underline_opt = None;

            let mut x = line.x_offset as i64;
            for rc in line.chars.iter() {
                if rc.link_id == Some(link_id) {
                    let (x0, x1) = underline_opt.get_or_insert((x, x));
                    *x0 = i64::min(*x0, x);
                    *x1 = i64::max(*x1, x + rc.font.char_w as i64);
                }
                x += rc.font.char_w as i64;
            }

            if let Some((x0, x1)) = underline_opt {
                let baseline_max = line.chars.iter().map(|rc| rc.font.base_y).max().unwrap();

                let y_ul = y + UNDERLINE_GAP as i64 + baseline_max as i64;
                underlines.push((y_ul, x0, x1));
            }

            y += line.h as i64;
        }

        Some((link_str, underlines))
    }

    pub fn get_char(&self, index: usize) -> &RichChar {
        let mut i = 0;
        for line in self.lines.iter() {
            if i <= index && index < i + line.chars.len() {
                return &line.chars[index - i];
            }
            i += line.chars.len();
        }

        panic!("Index out of bounds for rich line")
    }
}

impl alloc::fmt::Debug for FormattedRichText {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        let mut s = String::new();
        for line in self.lines.iter() {
            for rc in line.chars.iter() {
                s.push(rc.c);
            }
            s.push_str(" / ");
        }

        let links: Vec<String> = self.link_store.values().map(|(_, s)| s.clone()).collect();
        let joined = links.join(" / ");

        write!(f, "FormattedRichText [{}] links: {}", s, joined)
    }
}

impl FormattedRichLine {
    pub fn to_string(&self) -> String {
        self.chars.iter().map(|rc| rc.c).collect()
    }
}

pub fn format_rich_lines(
    text: &RichText,
    max_w: u32,
    justif: TextJustification,
) -> FormattedRichText {
    let RichText {
        chars, link_store, ..
    } = text;

    let lines: Vec<FormattedRichLine> = chars
        .split_inclusive(|rc| rc.c == '\n')
        .flat_map(|explicit_line| {
            let mut segments = Vec::new();
            let mut x = 0;
            let mut i1 = 0;
            let mut i2 = 0;
            loop {
                let ended = i2 == explicit_line.len();

                let push_line = {
                    if ended {
                        true
                    } else {
                        let rc = &explicit_line[i2];
                        let char_w = rc.width();
                        if x + char_w > max_w {
                            true
                        } else {
                            x += char_w;
                            i2 += 1;
                            false
                        }
                    }
                };

                if push_line {
                    let s = &explicit_line[i1..i2];
                    let line_w = s.iter().map(|rc| rc.width()).sum();
                    let line_h = s.iter().map(|rc| rc.height()).max().unwrap();
                    let line_base_y = s.iter().map(|rc| rc.font.base_y).max().unwrap();

                    let x_offset = match justif {
                        TextJustification::Left => 0,
                        TextJustification::Center => (max_w - line_w) / 2,
                        TextJustification::Right => max_w - line_w,
                    };

                    segments.push(FormattedRichLine {
                        chars: s.to_vec(),
                        w: line_w,
                        h: line_h,
                        base_y: line_base_y as u32,
                        x_offset,
                    });

                    i1 = i2;
                    x = 0;
                }

                if ended {
                    break;
                }
            }

            segments
        })
        .collect();

    let text_h = lines.iter().map(|line| line.h).sum();

    FormattedRichText {
        lines,
        w: max_w,
        h: text_h,
        justif,
        link_store: link_store.clone(),
    }
}

pub fn draw_rich_slice<F: FbViewMut>(fb: &mut F, rich_slice: &[RichChar], x0: i64, y0: i64) {
    if rich_slice.is_empty() {
        return;
    }

    let max_base_y = rich_slice
        .iter()
        .map(|rich_char| rich_char.font.base_y)
        .max()
        .unwrap();

    let mut x = x0;
    for rich_char in rich_slice.iter() {
        let dy = (max_base_y - rich_char.font.base_y) as i64;
        draw_char(
            fb,
            rich_char.c,
            x,
            y0 + dy,
            rich_char.font,
            rich_char.color,
            true,
        );
        x += rich_char.font.char_w as i64;
    }
}

pub fn compute_text_bbox(s: &str, font: &Font) -> (u32, u32) {
    let w = font.char_w * s.len();
    let h = font.char_h;
    (w as u32, h as u32)
}
