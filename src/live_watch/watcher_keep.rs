// use crate::MagicError;
// use crate::MagicError;
// use crate::err_tools;
use crate::WatcherX;

use crate::files;
use crate::windows::error_messages::ErrorSender;
use crate::windows::generic_windows::Loglet;

use notify::event::EventKind::*;
use notify::event::ModifyKind;
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};

pub type WatcherUpdate = files::File;
async fn load_file(file_path: PathBuf, file_tx: Sender<WatcherUpdate>, err_tx: ErrorSender) {
    // let file = files::File::load_file(&file_path).map_err(|err| Loglet::err(err));
    let result = files::File::load_file(&file_path).map_err(|err| Loglet::err(err));

    // keep
    match result {
        Ok(file) => file_tx.send(file).await.unwrap(),
        Err(err) => err_tx.send(err).await.unwrap(),
    }

    // err_tx
    //     .send(Loglet::err_s("This is a test error"))
    //     .await
    //     .unwrap();

    // file_tx.send(file).await.unwrap();
}

fn on_modify_event(
    mod_kind: ModifyKind,
    effected_paths: Vec<PathBuf>,
    file_tx: Sender<WatcherUpdate>,
    err_tx: ErrorSender,
    rt_clone: Arc<Mutex<tokio::runtime::Runtime>>,
) {
    if let ModifyKind::Data(_d) = mod_kind {
        println!(
            "Modification occured: Here is the list of affected paths: <{:?}>",
            effected_paths
        );

        loop {
            if let Ok(rt) = rt_clone.try_lock() {
                let file_tx2 = file_tx.clone();
                let err_tx2 = err_tx.clone();
                rt.spawn(async move {
                    for path_buf in effected_paths {
                        load_file(path_buf, file_tx2.clone(), err_tx2.clone()).await;
                    }
                });
                break;
            }
        }
    }
}

fn to_proc<D: std::fmt::Debug>(d: D, msg: &str) {
    println!("{} Kind proc: <{:?}>", msg, d)
}

pub fn create_watcher_with_actions(
    file_tx: Sender<files::File>,
    err_tx: ErrorSender,
    rt: Arc<Mutex<tokio::runtime::Runtime>>,
) -> notify::INotifyWatcher {
    let watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        let rt_clone = rt.clone();
        || -> Result<(), notify::Error> {
            let event = res?;
            let effected_paths = event.paths;
            match event.kind {
                Modify(mod_kind) => on_modify_event(
                    mod_kind,
                    effected_paths,
                    file_tx.clone(),
                    err_tx.clone(),
                    rt_clone,
                ),
                Create(create_kind) => to_proc(create_kind, "Create"),
                Access(access_kind) => to_proc(access_kind, "Access"),
                Remove(remove_kind) => to_proc(remove_kind, "Remove"),
                Any => to_proc("-Any-", "Any"),
                Other => to_proc("-other-", "Other"),
            }

            Ok(())
        }()
        .unwrap();
    })
    .unwrap();

    watcher
}

pub fn spawn_watcher_thread(
    mut watcher: WatcherX,
    mut path_buf: Option<PathBuf>,
    mut path_rx: Receiver<PathBuf>,
    err_sender: Sender<Loglet>,
    rt_am: Arc<Mutex<tokio::runtime::Runtime>>,
) {
    crate::force_am_once(rt_am, |rt| {
        rt.spawn(async move {
            loop {
                if let Some(pb) = path_buf.as_mut() {
                    if let Err(err) = watcher.watch(&pb, RecursiveMode::Recursive) {
                        // let err_msg = Err(err_tools::ErrorX::magic_err(err.to_string()));
                        let err_msg = Loglet::err_s(err.to_string());
                        err_sender.send(err_msg).await.unwrap();
                    }

                    if let Some(new_pb) = path_rx.recv().await {
                        path_buf = Some(new_pb);
                    }
                }
            }
        });
    });
}
