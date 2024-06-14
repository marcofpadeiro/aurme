use std::{path::Path, process::Command};

fn main() {
    let src = Path::new("autocomplete/_aurme");
    let dst = Path::new("/usr/share/zsh/site-functions/_aurme");
    println!();
    println!("cargo:warning=Adding autocomplete to Zsh");
    Command::new("sudo")
        .arg("cp")
        .arg(src)
        .arg(dst)
        .output()
        .expect("failed to execute process");
}
