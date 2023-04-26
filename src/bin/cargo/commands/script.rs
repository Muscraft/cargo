use crate::command_prelude::*;

pub fn exec(config: &mut Config, _args: &ArgMatches) -> CliResult {
    config
        .cli_unstable()
        .fail_if_stable_command(config, "<file>.rs", 0)?;
    Ok(())
}
