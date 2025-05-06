use alloc::vec::Vec;
use alloc::vec;
use crate::uitk::{UiContext, ButtonConfig};
use crate::{FbViewMut, Framebuffer, OwnedPixels, Rect};
use crate::uitk::layout::{make_horizontal_layout, LayoutItem};
use alloc::string::String;

impl<'a, F: FbViewMut> UiContext<'a, F> {

    pub fn choice_buttons_exclusive(&mut self, config: &ChoiceButtonsConfig, selected: &mut usize) {

        let layout = make_horizontal_layout(&config.rect, &vec![LayoutItem::Float; config.choices.len()]);
        
        let mut new_selected = *selected;

        for (i, choice) in config.choices.iter().enumerate() {
    
            let button_rect = &layout[i];

            let mut active = i == *selected;

            self.button_toggle(
                &ButtonConfig {
                    rect: button_rect.clone(),
                    text: choice.text.clone(),
                    icon: choice.icon,
                    freeze: i == *selected,
                },
                &mut active
            );

            if active && i != *selected {
                new_selected = i;
            }
        }

        *selected = new_selected;
    }

    pub fn choice_buttons_multi(&mut self, config: &ChoiceButtonsConfig, selected: &mut Vec<usize>) {

        let Rect { x0: mut x, y0, w, h } = config.rect;
        let button_w = w / (config.choices.len() as u32);

        let mut new_selected = Vec::new();

        for (i, choice) in config.choices.iter().enumerate() {

            let mut active = selected.contains(&i);

            self.button_toggle(
                &ButtonConfig {
                    rect: Rect { x0: x, y0, w: button_w, h },
                    text: choice.text.clone(),
                    icon: choice.icon,
                    freeze: false,
                },
                &mut active
            );

            x += button_w as i64;

            if active {
                new_selected.push(i);
            }
        }

        *selected = new_selected;
    }
}


pub struct ChoiceButtonsConfig {
    pub rect: Rect,
    pub choices: Vec<ChoiceConfig>
}


#[derive(Clone, Default)]
pub struct ChoiceConfig {
    pub text: String,
    pub icon: Option<&'static Framebuffer<OwnedPixels>>,
}
