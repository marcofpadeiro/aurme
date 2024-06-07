use std::path::{Path, PathBuf};

use home::home_dir;

pub fn expand_path(path: &str) -> PathBuf {
    let mut expanded_path = path.to_string();

    if expanded_path.starts_with("~") {
        if let Some(home) = home_dir() {
            let home_str = home.to_str().unwrap_or("~");
            expanded_path = expanded_path.replacen("~", home_str, 1);
        }
    }

    if expanded_path.contains('$') {
        expanded_path = shellexpand::env(&expanded_path)
            .unwrap_or_else(|_| expanded_path.clone().into())
            .to_string();
    }

    Path::new(&expanded_path).to_path_buf()
}
