use crate::{
    cmd_args,
    eframe_tools::{self, make_rich, ModalMachine},
    font_size_default,
    live_watch::watcher_keep,
    MagicError,
};
use std::{
    collections::{BTreeMap, HashMap},
    ops::DerefMut,
    path::{Path, PathBuf},
};

use egui::RichText;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use walkdir::WalkDir;

use std::task::Context;
use std::task::Poll;

pub struct MasterPath {
    path_buf: Option<PathBuf>,
    pub path_tx: tokio::sync::mpsc::Sender<PathBuf>,
}

impl MasterPath {
    pub fn get_path_buf_ref(&self) -> &Option<PathBuf> {
        &self.path_buf
    }
}

impl MasterPath {
    pub fn path_ref(&self) -> &Option<PathBuf> {
        &self.get_path_buf_ref()
    }

    pub fn path_clone(&self) -> Option<PathBuf> {
        self.get_path_buf_ref().clone()
    }

    pub fn new(path_buf: Option<PathBuf>, path_tx: Sender<PathBuf>) -> MasterPath {
        MasterPath { path_buf, path_tx }
    }
}

type Standard = RichText;
type LineSeparation = Vec<RichText>;
type AllLineSeparation = Vec<RichText>;

#[derive(Default)]
pub struct FileForm {
    standard: Option<Standard>,
    line_separation: Option<LineSeparation>,
    all_line_separation: Option<AllLineSeparation>,
}

impl FileForm {
    fn get_standard_mut(&mut self) -> &mut Option<Standard> {
        &mut self.standard
    }

    fn get_line_separation_mut(&mut self) -> &mut Option<LineSeparation> {
        &mut self.line_separation
    }
    fn get_line_separation_ref(&self) -> &Option<LineSeparation> {
        &self.line_separation
    }

    fn get_all_line_separation_mut(&mut self) -> &mut Option<AllLineSeparation> {
        &mut self.all_line_separation
    }
}

impl FileForm {
    pub fn line_separation_ref(&self) -> &Option<LineSeparation> {
        &self.line_separation
    }
}

fn should_separate(chr: &char) -> bool {
    if *chr != ' ' || *chr != '\n' {
        return true;
    }
    false
}
fn do_seperation(buffer: Vec<char>) -> String {
    buffer.into_iter().collect::<String>()
}

impl FileForm {
    fn new(
        standard: Option<String>,
        line_separation: Option<String>,
        all_line_separation: Option<String>,
    ) -> FileForm {
        let standard = standard.and_then(|s| Some(eframe_tools::make_rich(s, font_size_default())));

        let line_separation = line_separation.and_then(|s| {
            let mut rich_sep = Vec::new();
            let mut buffer = Vec::new();
            let mut track = false;
            for chr in s.chars() {
                buffer.push(chr);
                if chr != ' ' || chr != '\n' && !track {
                    track = true
                }

                if chr == '\n' {
                    if track {
                        let text = do_seperation(buffer);
                        let rich_text = make_rich(text, font_size_default());
                        rich_sep.push(rich_text);
                    }
                    buffer = Vec::new();
                    track = false;
                }
            }
            Some(rich_sep)
        });

        let all_line_separation = all_line_separation.and_then(|s| {
            let mut rich_sep = Vec::new();
            let mut buffer = Vec::new();
            for chr in s.chars() {
                buffer.push(chr);
                if chr == '\n' {
                    let text = do_seperation(buffer);
                    let rich_text = make_rich(text, font_size_default());
                    rich_sep.push(rich_text);
                    buffer = Vec::new();
                }
            }
            Some(rich_sep)
        });

        FileForm {
            standard,
            line_separation,
            all_line_separation,
        }
    }

    // fn to_line_sep(string: String) -> String {}
}

pub struct File {
    file: FileForm,
    path: PathBuf,
}

impl File {
    // Internal
    fn get_path_ref(&self) -> &PathBuf {
        &self.path
    }

    fn get_file_mut(&mut self) -> &mut FileForm {
        &mut self.file
    }

    fn get_file_ref(&self) -> &FileForm {
        &self.file
    }

    fn get_path_mut(&mut self) -> &mut PathBuf {
        &mut self.path
    }
}

impl File {
    fn new(path: PathBuf) -> Self {
        File {
            file: FileForm::default(),
            path,
        }
    }

    pub fn load_file(path_buf: &PathBuf) -> Result<File, MagicError> {
        let file_string = String::from_utf8(std::fs::read(path_buf)?)?;

        // TODO: Look at this more closesly. Should all fields be filled or Leave all but one field as None for FileForm::new(,,)?
        let file_form = FileForm::new(Some(file_string.clone()), Some(file_string.clone()), None);

        let file = File {
            file: file_form,
            path: path_buf.clone(),
        };

        Ok(file)
    }

    fn file_ref(&self) -> &FileForm {
        &self.get_file_ref()
    }

    // fn reload(&mut self) -> Result<(), MagicError> {
    // let file_string = String::from_utf8(std::fs::read(self.get_path_ref())?)?;

    // self.get_file_mut()
    // }
}

type CachedFiles = HashMap<PathBuf, Option<File>>;
pub struct FileCache {
    current_file: PathBuf,
    cached_files: CachedFiles,
    allow_caching: bool,
}

// Private
impl FileCache {
    fn get_current_file_ref(&self) -> &PathBuf {
        &self.current_file
    }
    fn get_current_file_mut(&mut self) -> &mut PathBuf {
        &mut self.current_file
    }
    fn get_cached_files_ref(&self) -> &CachedFiles {
        &self.cached_files
    }
    fn get_cached_files_mut(&mut self) -> &mut CachedFiles {
        &mut self.cached_files
    }
}

// Setters
impl FileCache {
    pub fn current_file_set(&mut self, new_current_file: PathBuf) {
        *self.get_current_file_mut() = new_current_file;
    }
}

// Public
impl FileCache {
    pub fn current_file_ref(&self) -> &PathBuf {
        &self.get_current_file_ref()
    }

    pub fn current_file_mut(&mut self) -> &mut PathBuf {
        self.get_current_file_mut()
    }

    pub fn full_file(&self) -> Option<&FileForm> {
        // println!("Current Path: <{:?}>", self.get_current_file_ref());
        let mut b = Vec::new();
        self.get_cached_files_ref()
            .keys()
            .for_each(|s| b.push(s.display()));
        let cached_files = self.get_cached_files_ref();
        println!("cached files");
        let file = cached_files.get(self.get_current_file_ref())?.as_ref();
        println!("files: <{}>", self.current_file.display());
        let file = file?;
        println!("files again?");
        Some(file.file_ref())
    }

    pub fn new(
        current_path: impl Into<PathBuf>,
        first_filepath: impl Into<PathBuf>,
        dir_list: BTreeMap<PathBuf, ()>,
    ) -> FileCache {
        // let current_file = File::new(current_path.into());
        let current_file = current_path.into();
        let dir_list = dir_list.into_keys().collect();
        let cached_files = load_dir_files(dir_list);
        let allow_caching = true;
        FileCache {
            current_file: first_filepath.into(),
            cached_files,
            allow_caching,
        }
        // .map(|pb| Path::canonicalize(&pb).unwrap())
    }
}

pub struct WatchList {
    mm: ModalMachine,
    file_cache: FileCache,
    file_update_rx: Receiver<watcher_keep::WatcherUpdate>,
}

// Private
impl WatchList {
    fn get_file_cache_ref(&self) -> &FileCache {
        &self.file_cache
    }
    fn get_file_cache_mut(&mut self) -> &mut FileCache {
        &mut self.file_cache
    }

    fn get_modal_machine_mut(&mut self) -> &mut ModalMachine {
        &mut self.mm
    }

    fn get_file_update_rx_mut(&mut self) -> &mut Receiver<watcher_keep::WatcherUpdate> {
        &mut self.file_update_rx
    }

    fn get_file_update_rx_ref(&self) -> &Receiver<watcher_keep::WatcherUpdate> {
        &self.file_update_rx
    }
}

impl WatchList {
    pub fn modal_machine_mut(&mut self) -> &mut ModalMachine {
        self.get_modal_machine_mut()
    }

    pub fn file_cache_ref(&self) -> &FileCache {
        self.get_file_cache_ref()
    }

    pub fn file_cache_mut(&mut self) -> &mut FileCache {
        self.get_file_cache_mut()
    }

    pub fn file_update_tx_ref(&self) -> &Receiver<watcher_keep::WatcherUpdate> {
        self.get_file_update_rx_ref()
    }
}

// Public
use crate::windows::error_messages::ErrorSender;
impl WatchList {
    pub fn new(
        current_dir: &PathBuf,
        name: impl Into<String>,
        file_update_rx: Receiver<watcher_keep::WatcherUpdate>,
        err_rx: ErrorSender,
    ) -> WatchList {
        let dir_list = make_dir_list(current_dir);
        let mut first = PathBuf::from("");

        for k in dir_list.keys() {
            if k.is_file() {
                first = k.clone()
            }
        }

        let mm = ModalMachine::new(first.clone(), dir_list.clone(), name.into());
        println!("First Path Buff: <{}>", first.display());
        let file_cache = FileCache::new(current_dir, first, dir_list);

        WatchList {
            mm,
            file_cache,
            file_update_rx,
        }
    }

    pub fn handle_udpates(&mut self, master_path_pb: PathBuf) {
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        while let Poll::Ready(op) = self.get_file_update_rx_mut().poll_recv(&mut cx) {
            match op {
                Some(updated_file) => {
                    // while let Some(updated_file) = self.get_file_update_rx_mut().poll_recv() {
                    // let path_buf = updated_file.get_path_ref()
                    println!(
                        "-------------------------- <{}> --------------------------",
                        updated_file.get_path_ref().display()
                    );
                    // let pb = &Path::canonicalize(updated_file.get_path_ref()).unwrap();
                    println!(
                        "All Keys: <{:?}>",
                        self.get_file_cache_mut()
                            .get_cached_files_ref()
                            .keys()
                            .collect::<Vec<&PathBuf>>(),
                    );
                    let key =
                        get_directory_specific_path(&master_path_pb, updated_file.get_path_ref())
                            .unwrap();
                    self.get_file_cache_mut()
                        .get_cached_files_mut()
                        .get_mut(&key)
                        .and_then(|file| {
                            *file = Some(updated_file);
                            None::<Option<File>>
                        });
                }
                None => {
                    panic!("Error, handle updates some how broke");
                }
            } // .unwrap() = Some(updated_file);
              // .as_deref_mut()
              // .insert(updated_file.get_path_ref(), Some(updated_file));
        }
    }
}

fn get_directory_specific_path(
    base: &PathBuf,
    strip: &PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base = Path::canonicalize(&base)?;
    let absolute_path = base.parent().unwrap();

    let strip = Path::canonicalize(strip)?;
    let stripped = strip.strip_prefix(&absolute_path)?.to_path_buf();
    // let prefix = .strip_prefix()

    Ok(stripped)
}

pub fn make_dir_list(current_dir: &PathBuf) -> BTreeMap<PathBuf, ()> {
    WalkDir::new(current_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .into_iter()
        .map(|entry| (entry.path().to_path_buf(), ()))
        .collect::<BTreeMap<PathBuf, ()>>()
    // .map(|entry| (Path::canonicalize(&entry.path().to_path_buf()).unwrap(), ()))
    // .map(|entry| (entry.path().to_path_buf(), ()))
}

fn load_dir_files(file_list: Vec<PathBuf>) -> CachedFiles {
    // file_list.iter_mut().for_each(|(pb, file)| file.);
    let mut cached_files = HashMap::new();
    file_list.into_iter().for_each(|s| {
        // let f = s.is_file().then_some(File::load_file(&s));
        let f = if s.is_file() {
            Some(File::load_file(&s).unwrap())
        } else {
            None
        };
        cached_files.insert(s, f);
    });

    cached_files
}

pub fn get_master_path() -> Result<(MasterPath, Receiver<PathBuf>), MagicError> {
    let maybe_args = cmd_args::get_arg()?;
    println!("maybe args:__________<{:?}>", maybe_args);

    let path_buf = maybe_args.and_then(|cmd_arg| Some(Path::new(&cmd_arg).to_path_buf()));

    let (tx, rx) = channel(32);
    let master_path = MasterPath::new(path_buf, tx);

    Ok((master_path, rx))
}
