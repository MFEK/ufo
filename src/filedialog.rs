use std::path::PathBuf;

pub fn open_folder(start_in: Option<&str>) -> Option<PathBuf> {
    match nfd::open_pick_folder(start_in) {
        Ok(nfd::Response::Okay(file)) => Some(file.into()),
        Ok(_) | Err(_) => None,
    }
}
