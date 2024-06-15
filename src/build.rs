use std::{error::Error, fmt, path::PathBuf, process::Command};

use which::which;

use crate::{
    clean::remove_cache,
    config::{expand_path, Config, VerboseOtion, PACKAGES_CACHE_PATH},
    package::Package,
    theme::{colorize, Type},
};

#[allow(dead_code)]
#[derive(Debug)]
enum BuildErrorType {
    Dependency(String),
    BuildProcess(String),
}

#[derive(Debug)]
#[allow(dead_code)]
struct BuildError(BuildErrorType, String, Option<Box<dyn std::error::Error>>);

impl Error for BuildError {}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.1)
    }
}

pub fn build_packages(
    packages: &Vec<&Package>,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    for package in packages.iter() {
        println!("{} {}...", colorize(Type::Info, "Building"), &package.name);
        let path = expand_path(PACKAGES_CACHE_PATH).join(&package.name);

        build(package, &path, config)?;
        eprintln!(
            "{} installed {}",
            colorize(Type::Success, "Successfully"),
            package.name
        );
    }
    Ok(())
}

fn build(package: &Package, path: &PathBuf, config: &Config) -> Result<(), Box<dyn Error>> {
    check_dependency("fakeroot")?;
    check_dependency("make")?;

    let no_confirm = match &config.no_confirm {
        true => "--noconfirm",
        false => "",
    };

    let (stdout, stderr) = config.get_verbose_config();

    let exit_status = Command::new("makepkg")
        .arg("-si")
        .arg(no_confirm)
        .stdout(stdout)
        .stderr(stderr)
        .current_dir(path)
        .spawn()?
        .wait_with_output()
        .unwrap();

    if !config.keep_cache {
        remove_cache(vec![package])?;
    }

    if exit_status.status.code().unwrap() != 0 {
        let err_msg = match config.verbose {
            VerboseOtion::Quiet => "Enable verbose and check above logs",
            _ => "Check above logs",
        };
        // TODO: Maybe make it non blocking
        return Err(Box::new(BuildError(
            BuildErrorType::BuildProcess(package.name.to_owned()),
            format!(
                "Makepkg failed to build package \"{}\". {}",
                package.name,
                err_msg
            ),
            None,
        )));
    }

    Ok(())
}

fn check_dependency(arg: &str) -> Result<(), Box<BuildError>> {
    if let Err(_) = which(arg) {
        return Err(Box::new(BuildError(
            BuildErrorType::Dependency(arg.to_owned()),
            format!("Required dependency \"{}\" not found", arg),
            None,
        )));
    }
    Ok(())
}
