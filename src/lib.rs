pub mod cmd_args;
pub mod files;
pub mod live_watch;
pub mod windows;

// Tis but a scratch

pub type MagicError = Box<dyn std::error::Error>;
pub type WatcherX = notify::INotifyWatcher;

// For development placeholder
pub fn panik() {
    panic!("Temp Panik!")
}

use std::sync::{Arc, Mutex, MutexGuard};
pub fn force_am_once<Any>(am: Arc<Mutex<Any>>, f: impl FnOnce(MutexGuard<Any>)) {
    // am.lock().unwrap()
    // let x = ;
    f(am.lock().unwrap());
}

pub fn force_am<Any>(am: Arc<Mutex<Any>>, f: impl Fn(MutexGuard<Any>)) {
    // am.lock().unwrap()
    // let x = ;
    f(am.lock().unwrap());
}

use chrono::{Timelike, Utc};
pub fn time_of_day() -> String {
    // "Hour:{} Minute:{} Second:{}",
    let time = Utc::now();
    format!("{}:{} -- {}", time.hour(), time.minute(), time.second())
}

pub fn font_size_default() -> egui::FontId {
    egui::FontId::proportional(30.0)
}

pub mod err_tools {
    use crate::MagicError;

    #[derive(Debug)]
    pub struct ErrorX {
        details: String,
    }

    impl ErrorX {
        pub fn new(msg: &str) -> ErrorX {
            ErrorX {
                details: msg.to_string(),
            }
        }
        pub fn new_box(msg: &str) -> Box<ErrorX> {
            Box::new(ErrorX {
                details: msg.to_string(),
            })
        }
        pub fn magic_err(msg: impl Into<String>) -> MagicError {
            Box::new(ErrorX {
                details: msg.into(),
            }) as MagicError
        }
    }

    impl std::fmt::Display for ErrorX {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.details)
        }
    }

    impl std::error::Error for ErrorX {
        fn description(&self) -> &str {
            &self.details
        }
    }
}

pub mod eframe_tools {
    use egui::Ui;
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    // Modal Machine Generic Types
    type Key = PathBuf;
    type Value = ();
    type Options = BTreeMap<Key, Value>;

    type SelectedOption = PathBuf;
    fn get_real_option<'a>(selected_option: (&'a PathBuf, &'a ())) -> &SelectedOption {
        selected_option.0
    }

    pub struct ModalMachine {
        selected_option: SelectedOption,
        options: Options,
        name: String,
        event: Option<()>,
    }

    fn to_text<D: std::fmt::Debug>(d: D) -> String {
        format!("{:?}", d)
    }

    impl ModalMachine {
        pub fn default() -> Self {
            Self {
                selected_option: SelectedOption::default(),
                options: Options::new(),
                name: "".to_string(),
                event: None,
            }
        }

        pub fn new(selected_option: SelectedOption, options: Options, name: String) -> Self {
            Self {
                selected_option,
                options,
                name,
                event: None,
            }
        }

        pub fn get_selected_option(&self) -> SelectedOption {
            self.selected_option.clone()
        }

        pub fn modal_machine(&mut self, id: i64, ui: &mut Ui) {
            ui.push_id(id, |ui| {
                eframe::egui::ComboBox::from_label(&self.name)
                    .selected_text(&to_text(&self.selected_option))
                    .show_ui(ui, |ui| {
                        self.options.iter().for_each(|option| {
                            let real_option = get_real_option(option);
                            if ui
                                .selectable_value(
                                    &mut self.selected_option,
                                    real_option.clone(),
                                    to_text(real_option),
                                )
                                .clicked()
                            {
                                println!("Clicked");
                                self.event = Some(());
                            };
                        });
                    });
            });
        }

        pub fn use_event(&mut self) -> Option<SelectedOption> {
            if let Some(_) = self.event {
                self.event = None;
                return Some(self.get_selected_option());
            }
            None
        }

        pub fn empty_event(&mut self) {
            self.event = None;
            println!("Inside event: <{:?}>", self.event);
        }
    }

    pub fn scroll_and_vert(ui: &mut Ui, id: impl std::hash::Hash, f: impl FnOnce(&mut Ui)) {
        eframe::egui::ScrollArea::vertical()
            .id_source(id)
            .show(ui, |ui| ui.horizontal_wrapped(|ui| f(ui)));
    }
    use crate::string_tools::newliner;
    pub fn space_vert(amount: usize, ui: &mut Ui) {
        //! Add vertical space using newlines
        ui.label(format!("{}", newliner(amount)));
    }

    pub fn make_rich(string: String, font_size: egui::FontId) -> egui::RichText {
        egui::RichText::new(string).font(font_size)
    }
}

pub mod string_tools {
    fn quick_maker(amount: usize, character: &str) -> String {
        let mut s = String::default();
        for _ in 0..amount {
            s = format!("{}{}", s, character)
        }
        s
    }

    pub fn newliner(amount: usize) -> String {
        quick_maker(amount, "\n")
    }

    pub fn tabber(amount: usize) -> String {
        quick_maker(amount, "\t")
    }
}
