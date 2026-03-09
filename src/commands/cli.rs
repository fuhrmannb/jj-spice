use clap::builder::styling::AnsiColor;
use clap::builder::Styles;
use clap::{Args, Parser, Subcommand};
use jj_cli::cli_util::GlobalArgs;

/// Colour theme matching jj-cli's help output.
///
/// Copied from jj-cli (where the constant is crate-private) so that
/// `jj-spice --help` looks visually consistent with `jj --help`.
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold())
    .usage(AnsiColor::Yellow.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::Green.on_default());

/// jj-spice: forge integration for jj.
#[derive(Parser, Clone, Debug)]
#[command(name = "jj-spice", styles = STYLES)]
pub struct Cli {
    #[command(flatten)]
    pub global_args: GlobalArgs,

    #[command(subcommand)]
    pub command: SpiceCommand,
}

/// Top-level subcommands exposed by jj-spice.
#[derive(Subcommand, Clone, Debug)]
pub enum SpiceCommand {
    /// Manage the bookmark stack.
    Stack(StackArgs),
}

/// Arguments for the `stack` subcommand group.
#[derive(Args, Clone, Debug)]
pub struct StackArgs {
    /// The stack operation to perform.
    #[command(subcommand)]
    pub command: StackCommand,
}

/// Operations available under `jj-spice stack`.
#[derive(Subcommand, Clone, Debug)]
pub enum StackCommand {
    /// Submit the current stack of bookmarks for review.
    Submit,
    /// Discover and track existing change requests for bookmarks in the stack.
    Sync(SyncArgs),
}

/// Arguments for `jj-spice stack sync`.
#[derive(Args, Clone, Debug)]
pub struct SyncArgs {
    /// Re-discover change requests even for bookmarks that are already tracked.
    #[arg(long)]
    pub force: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_stack_submit() {
        let cli = Cli::try_parse_from(["jj-spice", "stack", "submit"]).unwrap();
        assert!(matches!(
            cli.command,
            SpiceCommand::Stack(StackArgs {
                command: StackCommand::Submit
            })
        ));
    }

    #[test]
    fn parse_stack_sync_without_force() {
        let cli = Cli::try_parse_from(["jj-spice", "stack", "sync"]).unwrap();
        match cli.command {
            SpiceCommand::Stack(StackArgs {
                command: StackCommand::Sync(args),
            }) => assert!(!args.force),
            _ => panic!("expected Sync"),
        }
    }

    #[test]
    fn parse_stack_sync_with_force() {
        let cli = Cli::try_parse_from(["jj-spice", "stack", "sync", "--force"]).unwrap();
        match cli.command {
            SpiceCommand::Stack(StackArgs {
                command: StackCommand::Sync(args),
            }) => assert!(args.force),
            _ => panic!("expected Sync"),
        }
    }

    #[test]
    fn parse_no_args_fails() {
        assert!(Cli::try_parse_from(["jj-spice"]).is_err());
    }

    #[test]
    fn parse_unknown_subcommand_fails() {
        assert!(Cli::try_parse_from(["jj-spice", "stack", "unknown"]).is_err());
    }

    // ---- Global args integration tests ----

    #[test]
    fn parse_repository_short_flag() {
        let cli = Cli::try_parse_from(["jj-spice", "-R", "/tmp/repo", "stack", "submit"]).unwrap();
        assert_eq!(cli.global_args.repository.as_deref(), Some("/tmp/repo"));
    }

    #[test]
    fn parse_repository_long_flag() {
        let cli = Cli::try_parse_from(["jj-spice", "--repository", "/tmp/repo", "stack", "submit"])
            .unwrap();
        assert_eq!(cli.global_args.repository.as_deref(), Some("/tmp/repo"));
    }

    #[test]
    fn parse_at_operation() {
        let cli = Cli::try_parse_from(["jj-spice", "--at-operation", "abc123", "stack", "submit"])
            .unwrap();
        assert_eq!(cli.global_args.at_operation.as_deref(), Some("abc123"));
    }

    #[test]
    fn parse_at_op_alias() {
        let cli = Cli::try_parse_from(["jj-spice", "--at-op", "@-", "stack", "submit"]).unwrap();
        assert_eq!(cli.global_args.at_operation.as_deref(), Some("@-"));
    }

    #[test]
    fn parse_color_flag() {
        let cli = Cli::try_parse_from(["jj-spice", "--color", "never", "stack", "submit"]).unwrap();
        assert!(cli.global_args.early_args.color.is_some());
    }

    #[test]
    fn parse_no_pager_flag() {
        let cli = Cli::try_parse_from(["jj-spice", "--no-pager", "stack", "submit"]).unwrap();
        assert_eq!(
            cli.global_args.early_args.no_pager.unwrap_or_default(),
            true
        );
    }

    #[test]
    fn parse_quiet_flag() {
        let cli = Cli::try_parse_from(["jj-spice", "--quiet", "stack", "submit"]).unwrap();
        assert_eq!(cli.global_args.early_args.quiet.unwrap_or_default(), true);
    }

    #[test]
    fn parse_config_flag() {
        let cli =
            Cli::try_parse_from(["jj-spice", "--config", "ui.color=never", "stack", "submit"])
                .unwrap();
        assert_eq!(cli.global_args.early_args.config, vec!["ui.color=never"]);
    }

    #[test]
    fn parse_multiple_config_flags() {
        let cli = Cli::try_parse_from([
            "jj-spice",
            "--config",
            "ui.color=never",
            "--config",
            "user.email=test@example.com",
            "stack",
            "submit",
        ])
        .unwrap();
        assert_eq!(
            cli.global_args.early_args.config,
            vec!["ui.color=never", "user.email=test@example.com"]
        );
    }

    #[test]
    fn parse_debug_flag() {
        let cli = Cli::try_parse_from(["jj-spice", "--debug", "stack", "submit"]).unwrap();
        assert!(cli.global_args.debug);
    }

    #[test]
    fn parse_ignore_working_copy_flag() {
        let cli =
            Cli::try_parse_from(["jj-spice", "--ignore-working-copy", "stack", "submit"]).unwrap();
        assert!(cli.global_args.ignore_working_copy);
    }

    #[test]
    fn parse_global_args_after_subcommand() {
        // Global args should work after the subcommand too.
        let cli = Cli::try_parse_from([
            "jj-spice",
            "stack",
            "submit",
            "-R",
            "/tmp/repo",
            "--no-pager",
        ])
        .unwrap();
        assert_eq!(cli.global_args.repository.as_deref(), Some("/tmp/repo"));
        assert_eq!(
            cli.global_args.early_args.no_pager.unwrap_or_default(),
            true
        );
    }

    #[test]
    fn parse_defaults_are_none_or_false() {
        let cli = Cli::try_parse_from(["jj-spice", "stack", "submit"]).unwrap();
        assert!(cli.global_args.repository.is_none());
        assert!(cli.global_args.at_operation.is_none());
        assert!(!cli.global_args.debug);
        assert!(!cli.global_args.ignore_working_copy);
        assert!(!cli.global_args.ignore_immutable);
        assert!(cli.global_args.early_args.color.is_none());
        assert!(cli.global_args.early_args.config.is_empty());
        assert!(cli.global_args.early_args.config_file.is_empty());
    }
}
