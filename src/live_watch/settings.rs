use egui::FontId;

#[derive(PartialEq)]
pub enum TextMode {
    Newline,
    AllNewline,
    Selectable,
    Standard,
}

pub struct Settings {
    text_mode: TextMode,
    word_wrap: bool,
    font_size: FontId,
}

impl Settings {
    pub fn new(
        text_mode: Option<TextMode>,
        word_wrap: Option<bool>,
        font_size: Option<f32>,
    ) -> Self {
        let text_mode = text_mode.unwrap_or(TextMode::Standard);
        let word_wrap = word_wrap.unwrap_or(true);
        let font_size = font_size
            .and_then(|f| Some(FontId::proportional(f)))
            .unwrap_or(FontId::proportional(30.0));

        Settings {
            text_mode,
            word_wrap,
            font_size,
        }
    }
    pub fn default() -> Self {
        Self::new(None, None, None)
    }
}

impl Settings {
    fn get_word_wrap_ref(&self) -> &bool {
        &self.word_wrap
    }
    fn get_font_size_ref(&self) -> &FontId {
        &self.font_size
    }
    fn get_text_mode_mut(&mut self) -> &mut TextMode {
        &mut self.text_mode
    }
    fn get_text_mode_ref(&self) -> &TextMode {
        &self.text_mode
    }
    fn get_word_wrap_mut(&mut self) -> &mut bool {
        &mut self.word_wrap
    }
    fn get_font_size_mut(&mut self) -> &mut FontId {
        &mut self.font_size
    }

    pub fn text_mode_mut(&mut self) -> &mut TextMode {
        self.get_text_mode_mut()
    }
    pub fn text_mode_ref(&self) -> &TextMode {
        self.get_text_mode_ref()
    }
    pub fn word_wrap_mut(&mut self) -> &mut bool {
        self.get_word_wrap_mut()
    }
    pub fn word_wrap_set(&mut self, set_to: bool) {
        *self.get_word_wrap_mut() = set_to;
    }
    pub fn word_wrap_ref(&self) -> &bool {
        self.get_word_wrap_ref()
    }
    pub fn font_size_mut(&mut self) -> &mut FontId {
        self.get_font_size_mut()
    }

    // fn text_mode_ref(&self) -> &TextMode {
    //     self.get_text_mode_ref()
    // }
    // fn word_wrap_ref(&self) -> &bool {
    //     self.get_word_wrap_ref()
    // }
    pub fn font_size_ref(&self) -> &FontId {
        self.get_font_size_ref()
    }
}
