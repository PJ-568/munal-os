use crate::Color;

#[derive(Clone)]
#[repr(C)]
pub struct StyleSheet {
    pub colors: StyleSheetColors,
    pub margin: u32,
    pub text_sizes: TextSizes,
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
pub struct TextSizes {
    pub small: u32,
    pub medium: u32,
    pub large: u32,
}
