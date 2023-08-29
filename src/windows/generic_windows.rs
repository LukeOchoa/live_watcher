use crate::{
    eframe_tools::{scroll_and_vert, space_vert},
    string_tools::*,
    time_of_day,
};
use eframe::egui::{Context, Ui};

#[derive(Default)]
pub struct GenericWindow {
    pub is_window_open: bool,
    pub try_to_open_window: bool,
    pub log: MessageLog,
    pub name: String,
}

impl GenericWindow {
    pub fn default() -> Self {
        Default::default()
    }

    pub fn new(name: &str) -> Self {
        //! Creates/Returns a GenericWindow and simultaneously allows you to name it
        let mut gw = Self::default();
        gw.namae(name);
        gw
    }

    pub fn get_name(&self) -> String {
        self.name.to_string()
    }

    pub fn namae(&mut self, name: &str) -> String {
        //! Set the (Display.name) while also return that same name
        self.name = name.to_string();
        self.name.to_string()
    }

    pub fn open_window(&mut self) {
        self.is_window_open = true;
    }

    pub fn show_open_window_on_click_button(&mut self, ui: &mut Ui, name: &str) {
        // Shows to screen a ui.button
        eframe::egui::Grid::new(name).show(ui, |ui| {
            if ui.button(name).clicked() {
                self.is_window_open = !self.is_window_open
            }
            if self.try_to_open_window {
                self.is_window_open = false;
                self.try_to_open_window = false;
            }
            ui.end_row();
        });
    }

    pub fn show(&mut self, ctx: Context, f: impl Fn(&mut Ui, Context, &mut MessageLog)) -> bool {
        //! Shows the window to the screen but takes a Closure as well to allow
        //!
        //! Custom ui/behavior
        let mut is_window_shut: bool = self.is_window_open;
        eframe::egui::Window::new(&self.name)
            .resizable(true)
            .open(&mut is_window_shut)
            .show(&ctx, |ui| f(ui, ctx.clone(), &mut self.log));

        self.is_window_open = is_window_shut;

        is_window_shut
    }

    pub fn push_loglet(&mut self, loglet: Loglet) {
        //! Push loglet to the end of the vector
        self.log.push(loglet);
        self.open_window();
    }

    pub fn display_generic_window(gw: &mut GenericWindow, id: i64, ui: &mut Ui, ctx: Context) {
        //! Shows to the screen GenericWindow in all its Loggy glory!
        //!
        //! Meaning... it shows it specifically with all its log properties
        gw.show(ctx.clone(), |ui, _, log| {
            scroll_and_vert(ui, id, |ui| {
                Self::clear(log, ui);
                log.show(ui);
            })
        });
        gw.show_open_window_on_click_button(ui, &gw.get_name());
    }
    fn clear(log: &mut MessageLog, ui: &mut Ui) {
        if ui.button("Clear Window").clicked() {
            *log = MessageLog::default();
        }
        space_vert(2, ui);
    }
}

#[derive(Default)]
pub struct MessageLog {
    log: Vec<Loglet>,
}

impl MessageLog {
    pub fn default() -> Self {
        Default::default()
    }
    pub fn show(&self, ui: &mut Ui) {
        self.log
            .iter()
            .rev()
            .enumerate()
            .for_each(|(index, loglet)| {
                let formated_loglet =
                    format!("{}):{}{}", index, newliner(1), loglet.format_loglet());
                ui.label(formated_loglet);
                ui.label(newliner(3));
                ui.end_row()
            })
    }
    pub fn push(&mut self, loglet: Loglet) {
        // Max length of the Log
        while self.log.len() >= 30 {
            self.log.remove(0);
        }
        // Push
        self.log.push(loglet)
    }
}

#[derive(Debug)]
pub struct Loglet {
    kind: String,
    msg: String,
    time: String,
}

impl Loglet {
    pub fn new(kind: &str, msg: &str, time: &str) -> Loglet {
        Self {
            kind: kind.to_string(),
            msg: msg.to_string(),
            time: time.to_string(),
        }
    }
    pub fn err_s(msg: impl Into<String>) -> Loglet {
        Self {
            kind: "Error".to_string(),
            msg: msg.into(),
            time: time_of_day(),
        }
    }
    pub fn err(err: crate::MagicError) -> Loglet {
        Self {
            kind: "Error".to_string(),
            msg: err.to_string(),
            time: time_of_day(),
        }
    }

    pub fn format_loglet(&self) -> String {
        let lyne = |elem: &String| -> String {
            format!("{}{}{}{}", newliner(1), tabber(1), elem, newliner(2))
        };
        let lyne_2 = "/================================================/";
        format!(
            "Kind:{}Message:{}Time:{}{}",
            lyne(&self.kind),
            lyne(&self.msg),
            lyne(&self.time),
            lyne_2
        )
    }
}
