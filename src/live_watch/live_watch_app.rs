// use std::fs::create_dir;

// use egui::Context;

// use egui::RichText;
// use crate::{err_tools, files};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::files::MasterPath;
use crate::files::{self, WatchList};
use crate::live_watch::settings;
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
    fn get_master_path_ref(&self) -> &MasterPath {
        &self.master_path
    }
    fn get_settings_mut(&mut self) -> &mut settings::Settings {
        &mut self.settings
    }
}

impl LiveWatch {
    fn watch_list_mut(&mut self) -> &mut Option<files::WatchList> {
        self.get_watch_list_mut()
    }

    pub fn master_path_pb_ref(&self) -> &Option<PathBuf> {
        self.get_master_path_ref().get_path_buf_ref()
    }

    fn settings_mut(&mut self) -> &mut settings::Settings {
        self.get_settings_mut()
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

fn header() {}

fn user_settings(lw: &mut LiveWatch, ui: &mut egui::Ui, ctx: egui::Context) {
    lw.err_msg.display.namae("Error Messages");
    GenericWindow::display_generic_window(&mut lw.err_msg.display, 3, ui, ctx);

    egui::introspection::font_id_ui(ui, &mut lw.settings_mut().font_size_mut());

    ui.horizontal(|ui| {
        ui.radio_value(
            lw.settings_mut().text_mode_mut(),
            settings::TextMode::Newline,
            "Separate Lines",
        );
        ui.radio_value(
            lw.settings_mut().text_mode_mut(),
            settings::TextMode::AllNewline,
            "All Separate Lines",
        );
        ui.radio_value(
            lw.settings_mut().text_mode_mut(),
            settings::TextMode::Selectable,
            "Highlight/Copyable Mode",
        );
    });
}

fn load_watch_list(lw: &mut LiveWatch) {
    if let None = lw.watch_list {}
}

fn display_paths(lw: &mut LiveWatch, ui: &mut egui::Ui) {
    if let None = lw.watch_list_mut() {}
}

// fn handle_directory_modal_machine(lw: &mut LiveWatch, ui: &mut egui::Ui) {
//     lw.watch_list_mut().as_mut().and_then(|wl| {
//         // Display Modal Machine
//         wl.modal_machine_mut().modal_machine(5, ui);

//         // On selected file, set the newly selected file
//         wl.modal_machine_mut()
//             .use_event()
//             .and_then(|selected_option| {
//                 wl.file_cache_mut().current_file_set(selected_option);
//                 None::<PathBuf>
//             });

//         None::<WatchList>
//     });
// }

fn display_directory_list(lw: &mut LiveWatch, ui: &mut egui::Ui) -> Option<()> {
    let watch_list = lw.watch_list_mut().as_mut()?;
    watch_list.modal_machine_mut().modal_machine(5, ui);

    None
}

fn use_directory_list_mm_event(lw: &mut LiveWatch, ui: &mut egui::Ui) -> Option<()> {
    // On event (if an event happens), set the newly selected file as the current file
    let watch_list = lw.watch_list_mut().as_mut()?;
    let selected_option = watch_list.modal_machine_mut().use_event()?;
    watch_list
        .file_cache_mut()
        .current_file_set(selected_option);

    None
}

fn display_file(lw: &mut LiveWatch, ui: &mut egui::Ui) -> Option<()> {
    let watch_list = lw.watch_list.as_ref()?;
    let file_form = watch_list.file_cache_ref().full_file()?;
    let rich_texts = file_form.line_separation_ref().as_ref()?;

    for rt in rich_texts {
        ui.add(egui::Label::new(rt.clone()));
    }
    None
}

impl eframe::App for LiveWatch {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            user_settings(self, ui, ctx.clone());
            ui.label("Breaker");

            display_directory_list(self, ui);
            use_directory_list_mm_event(self, ui);
            display_file(self, ui);

            // if let None = self.watch_list {
            // } else {
            //     self.watch_list_mut()
            //         .as_mut()
            //         .unwrap()
            //         .modal_machine_mut()
            //         .modal_machine(5, ui);

            //     if let Some(selected_option) = self
            //         .watch_list_mut()
            //         .as_mut()
            //         .unwrap()
            //         .modal_machine_mut()
            //         .use_event()
            //     {
            //         self.watch_list_mut()
            //             .as_mut()
            //             .unwrap()
            //             .file_cache_mut()
            //             .current_file_set(selected_option);
            //     }
            //     || -> Option<()> {
            //         let watch_list = self.watch_list.as_ref()?;
            //         // println!("watch list");
            //         let file_form = watch_list.file_cache_ref().full_file()?;
            //         // println!("File form");
            //         let rich_texts = file_form.line_separation_ref().as_ref()?;
            //         // println!("rich texts");
            //         // println!("fileform before rich text: <{}>", file_form);
            //         for rt in rich_texts {
            //             ui.add(egui::Label::new(rt.clone()));
            //         }

            //         None
            //     }();
            // }
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
