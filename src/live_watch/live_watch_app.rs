// use std::fs::create_dir;

// use egui::Context;

// use egui::RichText;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::files::MasterPath;
use crate::windows::error_messages::ErrorMessage;
use crate::windows::generic_windows::GenericWindow;
// use crate::{err_tools, files};
use crate::files;

use super::watcher_keep;

pub struct LiveWatch {
    watch_list: Option<files::WatchList>,
    master_path: files::MasterPath,
    err_msg: ErrorMessage,
    rt: Arc<Mutex<tokio::runtime::Runtime>>,
}

impl LiveWatch {
    fn get_watch_list_mut(&mut self) -> &mut Option<files::WatchList> {
        &mut self.watch_list
    }
    fn get_master_path_ref(&self) -> &MasterPath {
        &self.master_path
    }
}

impl LiveWatch {
    fn watch_list_mut(&mut self) -> &mut Option<files::WatchList> {
        self.get_watch_list_mut()
    }

    pub fn master_path_pb_ref(&self) -> &Option<PathBuf> {
        self.get_master_path_ref().get_path_buf_ref()
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

        LiveWatch {
            watch_list,
            master_path,
            err_msg,
            rt,
        }
    }
}

fn header() {}
fn settings(lw: &mut LiveWatch, ui: &mut egui::Ui, ctx: egui::Context) {
    lw.err_msg.display.namae("Error Messages");
    GenericWindow::display_generic_window(&mut lw.err_msg.display, 3, ui, ctx)
}

fn load_watch_list(lw: &mut LiveWatch) {
    if let None = lw.watch_list {}
}

fn display_paths(lw: &mut LiveWatch, ui: &mut egui::Ui) {
    if let None = lw.watch_list_mut() {}
}

impl eframe::App for LiveWatch {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            settings(self, ui, ctx.clone());
            ui.label("Breaker");

            if let None = self.watch_list {
            } else {
                self.watch_list_mut()
                    .as_mut()
                    .unwrap()
                    .modal_machine_mut()
                    .modal_machine(5, ui);

                if let Some(selected_option) = self
                    .watch_list_mut()
                    .as_mut()
                    .unwrap()
                    .modal_machine_mut()
                    .use_event()
                {
                    self.watch_list_mut()
                        .as_mut()
                        .unwrap()
                        .file_cache_mut()
                        .current_file_set(selected_option);
                }
                || -> Option<()> {
                    let watch_list = self.watch_list.as_ref()?;
                    // println!("watch list");
                    let file_form = watch_list.file_cache_ref().full_file()?;
                    // println!("File form");
                    let rich_texts = file_form.line_separation_ref().as_ref()?;
                    // println!("rich texts");
                    for rt in rich_texts {
                        ui.add(egui::Label::new(rt.clone()));
                    }

                    None
                }();
            }
        });

        self.err_msg.block_update_log();
        let maybe_mp_pb = self.master_path.path_clone();
        if let Some(mp_pb) = maybe_mp_pb {
            self.watch_list_mut()
                .as_mut()
                .unwrap()
                .handle_udpates(mp_pb);
        }

        ctx.request_repaint();
    }
}
