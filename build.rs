use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SHARED_DIR: &str = "shared/win32";

fn main() {
    let app_exec_name: String = format!("{}.exe", env!("CARGO_PKG_NAME"));
    //let app_exec_name = "update_generator.exe".to_string();
    let target_file = String::from(app_exec_name.as_str());

    println!("target_file = {:?}", &target_file);

    let current_dir = env::current_dir().unwrap();

    if let Ok(Some(target_dir_path)) = get_exe_dir(&current_dir, &target_file) {

        // copying from shared project subdir to bin root.
        for entry in fs::read_dir(SHARED_DIR).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                fs::copy(
                    &path
                    , Path::new(&target_dir_path).join(&path.file_name().unwrap()))
                    .unwrap();
            }
        }
    }
}

fn get_exe_dir(dir: &PathBuf, target_file: &str) -> io::Result<Option<PathBuf>> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(inner_result) = get_exe_dir(&path, &target_file)? {
                    return Ok(Some(inner_result));
                }
            } else {
                if entry.file_name().to_str().unwrap() == target_file {
                    return Ok(Some(dir.to_path_buf()));
                }
            }
        }
    }
    Ok(None)
}
