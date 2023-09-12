//

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::eframe_tools::make_rich;
use crate::files;
use crate::files::MasterPath;
use crate::live_watch::settings;
use crate::live_watch::settings::TextMode;
use crate::windows::error_messages::ErrorMessage;
use crate::windows::generic_windows::GenericWindow;

use super::watcher_keep;

pub struct LiveWatch {
    watch_list: Option<files::WatchList>,
    master_path: files::MasterPath,
    err_msg: ErrorMessage,
    rt: Arc<Mutex<tokio::runtime::Runtime>>,
    settings: settings::Settings,
}

impl LiveWatch {
    fn get_watch_list_mut(&mut self) -> &mut Option<files::WatchList> {
        &mut self.watch_list
    }
    fn get_watch_list_ref(&self) -> &Option<files::WatchList> {
        &self.watch_list
    }
    fn get_master_path_ref(&self) -> &MasterPath {
        &self.master_path
    }
    fn get_settings_mut(&mut self) -> &mut settings::Settings {
        &mut self.settings
    }
    fn get_settings_ref(&self) -> &settings::Settings {
        &self.settings
    }
}

impl LiveWatch {
    fn watch_list_mut(&mut self) -> &mut Option<files::WatchList> {
        self.get_watch_list_mut()
    }
    fn watch_list_ref(&self) -> Option<&files::WatchList> {
        self.get_watch_list_ref().as_ref()
    }

    pub fn master_path_pb_ref(&self) -> &Option<PathBuf> {
        self.get_master_path_ref().get_path_buf_ref()
    }

    fn settings_mut(&mut self) -> &mut settings::Settings {
        self.get_settings_mut()
    }
    fn settings_ref(&self) -> &settings::Settings {
        self.get_settings_ref()
    }
}

impl Default for LiveWatch {
    fn default() -> Self {
        let rt = Arc::new(Mutex::new(tokio::runtime::Runtime::new().unwrap()));
        let err_msg = ErrorMessage::new();
        let (master_path, path_buf_rx) = files::get_master_path().unwrap();

        // Handle async/parrallel watcher
        let (file_update_tx, file_update_rx) = tokio::sync::mpsc::channel(2);
        let watcher = watcher_keep::create_watcher_with_actions(
            file_update_tx,
            err_msg.sender_clone(),
            rt.clone(),
        );

        let watch_list = Some(files::WatchList::new(
            master_path.path_ref().as_ref().unwrap(),
            "Watch List",
            file_update_rx,
            err_msg.sender_clone(),
        ));

        watcher_keep::spawn_watcher_thread(
            watcher,
            master_path.path_clone(),
            path_buf_rx,
            err_msg.sender_clone(),
            rt.clone(),
        );

        // Settings
        let settings = settings::Settings::default();

        LiveWatch {
            watch_list,
            master_path,
            err_msg,
            rt,
            settings,
        }
    }
}

/// The Name of the file currently being displayed
fn header(lw: &mut LiveWatch, ui: &mut egui::Ui) {
    // Create the filename: TODO: change this as stored info in livewatch struct or something. It does this like 60 times a second every second LOL
    let filename = lw
        .watch_list_ref()
        .and_then(|wl| {
            let filename = wl
                .modal_machine_ref()
                .get_selected_option()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            Some(format!("File Name: <{}>", filename))
        })
        .unwrap_or(String::from("File Name: <No File Name>"));
    let header = make_rich(filename, lw.settings_ref().font_size_ref().to_owned());

    // display header to user interface
    ui.label(header);
}

fn user_settings(lw: &mut LiveWatch, ui: &mut egui::Ui, ctx: egui::Context) {
    lw.err_msg.display.namae("Error Messages");
    GenericWindow::display_generic_window(&mut lw.err_msg.display, 3, ui, ctx);

    egui::introspection::font_id_ui(ui, &mut lw.settings_mut().font_size_mut());

    ui.horizontal(|ui| {
        let text_mode = lw.settings_mut().text_mode_mut();
        ui.radio_value(text_mode, settings::TextMode::Standard, "Standard");
        ui.radio_value(text_mode, settings::TextMode::Newline, "Separate Lines");
        ui.radio_value(
            text_mode,
            settings::TextMode::AllNewline,
            "All Separate Lines",
        );
        ui.radio_value(
            text_mode,
            settings::TextMode::Selectable,
            "Highlight/Copyable Mode",
        );

        let current = lw.settings_ref().word_wrap_ref().to_owned();
        if ui.radio(current, "Word Wrap").clicked() {
            lw.settings_mut().word_wrap_set(!current);
        }
    });
}

fn display_directory_list(lw: &mut LiveWatch, ui: &mut egui::Ui) -> Option<()> {
    let watch_list = lw.watch_list_mut().as_mut()?;
    watch_list.modal_machine_mut().modal_machine(5, ui);

    None
}

fn use_directory_list_mm_event(lw: &mut LiveWatch) -> Option<()> {
    // On event (if an event happens), set the newly selected file as the current file
    let watch_list = lw.watch_list_mut().as_mut()?;
    let selected_option = watch_list.modal_machine_mut().use_event()?;
    watch_list
        .file_cache_mut()
        .current_file_set(selected_option);

    None
}

fn render_option(text_mode: &TextMode) -> impl Fn(&mut egui::Ui) {
    match text_mode {
        TextMode::Newline => |ui: &mut egui::Ui| {
            ui.separator();
        },
        TextMode::AllNewline => |ui: &mut egui::Ui| {
            ui.separator();
        },
        TextMode::Selectable => |ui: &mut egui::Ui| {},
        TextMode::Standard => |ui: &mut egui::Ui| {},
    }
}

fn display_file(lw: &mut LiveWatch, ui: &mut egui::Ui) {
    let subfn = |ui: &mut egui::Ui| -> Option<()> {
        let watch_list = lw.watch_list.as_ref()?;
        let file_form = watch_list.file_cache_ref().full_file()?;
        let rich_texts = file_form.get_file_text(lw.settings_ref().text_mode_ref())?;
        let font_id = lw.settings.font_size_ref().to_owned();

        // TODO: this is so unnecessarily expensive, but egui doesnt support selectable text properly yet... either store this somewhere as a string and rip it out or wait for egui to do its thing...
        if let TextMode::Selectable = lw.settings_ref().text_mode_ref() {
            let mut text: String = rich_texts.iter().map(|rt| rt.text().to_string()).collect();
            ui.add(egui::TextEdit::multiline(&mut text));
            return None;
        }

        // println!("file text");
        for rt in rich_texts {
            // println!("<{}>", rt.text());
            ui.add(egui::Label::new(rt.clone().font(font_id.clone())));
            render_option(lw.settings_ref().text_mode_ref())(ui);
        }
        // println!("/n");

        None
    };

    egui::ScrollArea::both().show(ui, |ui| {
        subfn(ui);
        ui.allocate_space(ui.available_size());
    });
}

impl eframe::App for LiveWatch {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            header(self, ui);
            user_settings(self, ui, ctx.clone());

            display_directory_list(self, ui);
            use_directory_list_mm_event(self);
            ui.separator();

            display_file(self, ui);
        });

        self.err_msg.block_update_log();
        let maybe_mp_pb = self.master_path.path_clone();
        if let Some(mp_pb) = maybe_mp_pb {
            self.watch_list_mut()
                .as_mut()
                .unwrap()
                .handle_updates(mp_pb);
        }

        ctx.request_repaint();
    }
}
