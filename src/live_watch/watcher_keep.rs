use crate::WatcherX;

use crate::files;
use crate::panik;
use crate::windows::error_messages::ErrorSender;
use crate::windows::generic_windows::Loglet;

use notify::event::EventKind::*;
use notify::event::ModifyKind;
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct RenameEvent {
    from: PathBuf,
    to: PathBuf,
}

impl RenameEvent {
    fn get_from_ref(&self) -> &PathBuf {
        &self.from
    }
    fn get_to_ref(&self) -> &PathBuf {
        &self.to
    }
}

impl RenameEvent {
    fn new(from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> RenameEvent {
        let from = from.into();
        let to = to.into();
        RenameEvent { from, to }
    }

    pub fn from_and_to_ref(&self) -> (&PathBuf, &PathBuf) {
        (self.get_from_ref(), self.get_to_ref())
    }
}

pub enum WatcherUpdate {
    FileContent(files::File),
    FileRename(RenameEvent),
    FileDelete(PathBuf),
}

impl WatcherUpdate {
    fn new_content(file: files::File) -> WatcherUpdate {
        WatcherUpdate::FileContent(file)
    }

    fn new_rename(from: PathBuf, to: PathBuf) -> WatcherUpdate {
        WatcherUpdate::FileRename(RenameEvent::new(from, to))
    }

    fn new_delete(deleted_path: PathBuf) -> WatcherUpdate {
        WatcherUpdate::FileDelete(deleted_path)
    }
}

// pub type WatcherUpdate = files::File;
async fn load_file(file_path: PathBuf, file_tx: Sender<WatcherUpdate>, err_tx: ErrorSender) {
    // let file = files::File::load_file(&file_path).map_err(|err| Loglet::err(err));
    let result = files::File::load_file(&file_path).map_err(|err| Loglet::err(err));

    // keep
    match result {
        Ok(file) => file_tx
            .send(WatcherUpdate::new_content(file))
            .await
            .unwrap(),
        Err(err) => err_tx.send(err).await.unwrap(),
    }

    // err_tx
    //     .send(Loglet::err_s("This is a test error"))
    //     .await
    //     .unwrap();

    // file_tx.send(file).await.unwrap();
}

fn send_watcher_update(
    future: impl std::future::Future<Output = ()> + Send + 'static,
    // effected_paths: Vec<PathBuf>,
    rt: Arc<Mutex<tokio::runtime::Runtime>>,
    // watcher_sender: Sender<WatcherUpdate>,
    // err_sender: ErrorSender,
) {
    loop {
        if let Ok(rt) = rt.try_lock() {
            rt.spawn(future);
            break;
        }
    }
}

fn on_modify_event(
    mod_kind: ModifyKind,
    effected_paths: Vec<PathBuf>,
    file_tx: Sender<WatcherUpdate>,
    err_tx: ErrorSender,
    rt_clone: Arc<Mutex<tokio::runtime::Runtime>>,
) {
    match mod_kind {
        ModifyKind::Name(rename) => match rename {
            notify::event::RenameMode::From => {}

            notify::event::RenameMode::To => {}

            notify::event::RenameMode::Both => {
                println!("--------------");
                println!("--------------");
                println!("--------------");
                let the_future = async move {
                    let from = effected_paths.get(0).unwrap().to_owned();
                    let to = effected_paths.get(1).unwrap().to_owned();
                    let watcher_update = WatcherUpdate::new_rename(from, to);
                    file_tx.send(watcher_update).await.unwrap();
                    println!("Sent");
                };
                send_watcher_update(the_future, rt_clone);
            }

            _ => panik(),
        },

        ModifyKind::Data(_data_change) => {
            println!(
                "Modification occured: Here is the list of affected paths: <{:?}>",
                effected_paths
            );
            let the_future = async move {
                for path_buf in effected_paths {
                    load_file(path_buf, file_tx.clone(), err_tx.clone()).await;
                }
            };
            send_watcher_update(the_future, rt_clone);
        }
        _ => {}
    }

    // if let ModifyKind::Name(rename) = mod_kind {
    //     match rename {
    //         notify::event::RenameMode::From => {}

    //         notify::event::RenameMode::To => {}

    //         notify::event::RenameMode::Both => {
    //             let the_future = async move {
    //                 let from = effected_paths.get(0).unwrap().to_owned();
    //                 let to = effected_paths.get(1).unwrap().to_owned();
    //                 let watcher_update = WatcherUpdate::new_rename(from, to);
    //                 file_tx.send(watcher_update).await.unwrap();
    //             };
    //             send_watcher_update(the_future, rt_clone.clone());
    //         }

    //         _ => panik(),
    //     }
    // }

    // if let ModifyKind::Data(_data_change) = mod_kind {
    //     println!(
    //         "Modification occured: Here is the list of affected paths: <{:?}>",
    //         effected_paths
    //     );
    //     let the_future = async move {
    //         for path_buf in effected_paths {
    //             load_file(path_buf, file_tx.clone(), err_tx.clone()).await;
    //         }
    //     };
    //     send_watcher_update(the_future, rt_clone);
    // }
}

fn to_proc<D: std::fmt::Debug>(d: D, msg: &str) {
    println!("{} Kind proc: <{:?}>", msg, d)
}

pub fn create_watcher_with_actions(
    file_tx: Sender<WatcherUpdate>,
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
