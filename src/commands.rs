/// CLI definitions (clap structs and enums).
pub mod cli;
/// Shell completion script generation.
mod completion;
/// Shared environment bootstrapped from the jj config and workspace.
pub(crate) mod env;
/// `stack submit` command implementation.
pub mod stack_submit;
/// `stack sync` command implementation.
pub mod stack_sync;

use jj_cli::cli_util::RevisionArg;

use cli::{Cli, SpiceCommand, StackCommand, UtilCommand};
use env::SpiceEnv;

/// Dispatch to the appropriate command.
///
/// Commands that don't need a workspace (e.g. `util completion`) are handled
/// directly. Commands that do need one lazily initialise [`SpiceEnv`].
pub(crate) fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let Cli {
        global_args,
        command,
    } = cli;

    match command {
        SpiceCommand::Util(util_args) => match util_args.command {
            UtilCommand::Completion(args) => completion::run(args.shell),
        },
        
        SpiceCommand::Stack(stack_args) => {
            let env = SpiceEnv::init(&global_args)?;

            let trunk_rev = RevisionArg::from("trunk()".to_string());
            let trunk = env.resolve_single_rev(&trunk_rev).map_err(|_| {
                "could not resolve trunk()\n\n\
                 Set the trunk bookmark in your jj config:\n  \
                 [revset-aliases]\n  \
                 'trunk()' = 'main@origin'"
            })?;

            let head = env
                .resolve_single_rev(&RevisionArg::AT)
                .map_err(|e| format!("failed to resolve @: {e}"))?;

            let rt = tokio::runtime::Runtime::new()?;

            match stack_args.command {
                StackCommand::Submit => stack_submit::run(&env, &trunk, &head),
                StackCommand::Sync(sync_args) => {
                    rt.block_on(stack_sync::run(&env, &trunk, &head, sync_args.force))
                }
            }
        }
    }
}
