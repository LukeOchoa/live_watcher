pub mod cmd_args;
pub mod files;
pub mod live_watch;

pub type MagicError = Box<dyn std::error::Error>;

pub mod err_tools {
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
}
