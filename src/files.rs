use crate::{cmd_args, eframe_tools::ModalMachine, MagicError};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct MasterPath {
    pub path_buf: Option<PathBuf>,
    pub path_tx: tokio::sync::mpsc::Sender<PathBuf>,
}

impl MasterPath {
    pub fn path_ref(&self) -> &Option<PathBuf> {
        &self.path_buf
    }

    pub fn new(path_buf: Option<PathBuf>, path_tx: Sender<PathBuf>) -> MasterPath {
        MasterPath { path_buf, path_tx }
    }
}

struct FileForm {
    standard: Option<String>,
    line_separation: Option<String>,
    all_line_separation: Option<String>,
}

pub struct File {
    file: FileForm,
    path: PathBuf,
}

pub struct FileCache {
    current_file: File,
    cached_files: HashMap<PathBuf, File>,
    allow_caching: bool,
}

// Private
impl FileCache {
    fn get_current_file_ref(&self) -> &File {
        &self.current_file
    }
    fn get_current_file_mut(&mut self) -> &mut File {
        &mut self.current_file
    }
}

// Public
impl FileCache {
    pub fn current_file_ref(&self) -> &File {
        &self.get_current_file_ref()
    }

    pub fn current_file_mut(&mut self) -> &mut File {
        &mut self.get_current_file_mut()
    }
}

pub struct WatchList {
    mm: ModalMachine,
    file_cache: FileCache,
}

// Private
impl WatchList {
    fn get_file_cache_ref(&self) -> &FileCache {
        &self.file_cache
    }
}

// Public
impl WatchList {
    pub fn file_cache_ref(&self) -> &FileCache {
        self.get_file_cache_ref()
    }

    // pub fn modal_machine_ref()
}

pub fn get_master_path() -> Result<(MasterPath, Receiver<PathBuf>), MagicError> {
    let maybe_args = cmd_args::get_arg()?;

    let path_buf = maybe_args.and_then(|cmd_arg| Some(Path::new(&cmd_arg).to_path_buf()));

    let (tx, rx) = channel(32);
    let master_path = MasterPath::new(path_buf, tx);

    Ok((master_path, rx))
}

// pub fn load_file(path_buf: PathBuf) -> File {

// }
