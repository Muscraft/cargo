use crate::command_prelude::*;
use cargo::core::{Verbosity, Workspace};
use cargo::CargoResult;
use cargo_util::ProcessError;
use std::path::{Path, PathBuf};

pub fn exec(path: &str, config: &mut Config, args: &ArgMatches) -> CliResult {
    config
        .cli_unstable()
        .fail_if_stable_command(config, "<file>.rs", 0)?;

    let file_path = file_path(path)?;
    let ws = workspace(&file_path, config)?;

    let compile_opts = args.compile_options(
        config,
        CompileMode::Build,
        Some(&ws),
        ProfileChecking::Custom,
    )?;

    cargo::ops::run(&ws, &compile_opts, &values_os(args, "args")).map_err(|err| {
        let proc_err = match err.downcast_ref::<ProcessError>() {
            Some(e) => e,
            None => return CliError::new(err, 101),
        };

        // If we never actually spawned the process then that sounds pretty
        // bad and we always want to forward that up.
        let exit_code = match proc_err.code {
            Some(exit) => exit,
            None => return CliError::new(err, 101),
        };

        // If `-q` was passed then we suppress extra error information about
        // a failed process, we assume the process itself printed out enough
        // information about why it failed so we don't do so as well
        let is_quiet = config.shell().verbosity() == Verbosity::Quiet;
        if is_quiet {
            CliError::code(exit_code)
        } else {
            CliError::new(err, exit_code)
        }
    })
}

fn file_path(cmd: &str) -> CargoResult<PathBuf> {
    let path = dunce::canonicalize(PathBuf::from(cmd))?;
    if path.exists() {
        Ok(path)
    } else {
        anyhow::bail!("single-file package `{}` does not exist", path.display())
    }
}

fn workspace<'a>(manifest_path: &Path, config: &'a Config) -> CargoResult<Workspace<'a>> {
    let mut ws = Workspace::new(&manifest_path, config)?;
    if config.cli_unstable().avoid_dev_deps {
        ws.set_require_optional_deps(false);
    }
    Ok(ws)
}
