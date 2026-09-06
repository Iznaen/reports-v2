use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_fs::{FsExt, OpenOptions};
use std::io::Write;

pub fn test<R: Runtime>(app: &AppHandle<R>) {
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    let mut file = app.fs().open("some_path", opts).unwrap();
    file.write_all(b"test").unwrap();
}
