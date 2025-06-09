use crate::Color;

const FONT_FAMILY_NAME_MAX_LEN: usize = 64;

#[derive(Clone)]
#[repr(C)]
pub struct StyleSheet {
    pub colors: StyleSheetColors,
    pub margin: u32,
    pub text: StyleSheetText,
}

#[derive(Clone)]
#[repr(C)]
pub struct StyleSheetColors {
    pub background: Color,
    pub hover_overlay: Color,
    pub selected_overlay: Color,
    pub red: Color,
    pub yellow: Color,
    pub green: Color,
    pub blue: Color,
    pub purple: Color,
    pub element: Color,
    pub frame: Color,
    pub text: Color,
    pub accent: Color,
    pub editable: Color,
    pub outline: Color,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct StyleSheetText {
    // We can't allow any pointer in the stylesheet struct,
    // so strings must be stored as byte arrays
    // (no String or CString or whatnot)
    font_family_bytes: [u8; FONT_FAMILY_NAME_MAX_LEN],
    font_family_len: u32,

    pub sizes: TextSizes,
}

impl StyleSheetText {
    pub fn new(family_name: &str, sizes: TextSizes) -> Self {
        let src_bytes = family_name.as_bytes();
        let len = src_bytes.len();
        let mut font_family_bytes = [0u8; FONT_FAMILY_NAME_MAX_LEN];
        font_family_bytes[..len].copy_from_slice(src_bytes);

        Self {
            font_family_bytes,
            font_family_len: len as u32,
            sizes,
        }
    }

    pub fn font_family(&self) -> &str {
        str::from_utf8(&self.font_family_bytes[..self.font_family_len as usize])
            .expect("Invalid stylesheet data")
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct TextSizes {
    pub small: u32,
    pub medium: u32,
    pub large: u32,
}
