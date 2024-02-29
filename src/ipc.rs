use mfek_ipc::IPCInfo;
use std::{path::PathBuf, thread};

use crate::viewer::UFOViewer;

pub fn launch_fs_watcher(viewer: &mut UFOViewer, path: &PathBuf) -> thread::JoinHandle<()> {
    let ipc_info = IPCInfo::from_glif_path("MFEKglif".to_string(), &path);
    if let Some(font) = ipc_info.font {
        mfek_ipc::notifythread::launch(font, viewer.filesystem_watch_tx.clone())
    } else {
        mfek_ipc::notifythread::launch(
            ipc_info.glyph.unwrap().parent().unwrap().to_owned(),
            viewer.filesystem_watch_tx.clone(),
        )
    }
}
