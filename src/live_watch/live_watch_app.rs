use egui::Context;

use crate::files;

pub struct LiveWatch {
    watch_list: Option<files::WatchList>,
    master_path: files::MasterPath,
}

impl Default for LiveWatch {
    fn default() -> Self {
        let watch_list = None;
        let (master_path, path_buf_rx) = files::get_master_path().unwrap();

        LiveWatch { watch_list }
    }
}

fn header() {}
fn settings() {}

fn load_watch_list(lw: &mut LiveWatch) {
    if let None = lw.watch_list {}
}

impl eframe::App for LiveWatch {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {});
        ctx.request_repaint();
    }
}
