mod cli;

use clap::Parser;
use jj_cli::{cli_util::find_workspace_dir, config};
use jj_lib::{
    config::{ConfigGetError, StackedConfig},
    workspace::{DefaultWorkspaceLoaderFactory, WorkspaceLoaderFactory},
};
use std::env;

use cli::Cli;

fn main() {
    let _ = Cli::parse();
    let config = match setup_config() {
        Ok(c) => c,
        Err(e) => panic!("Failed to load config: {}", e),
    };

    let cwd = env::current_dir().and_then(dunce::canonicalize).unwrap();
    let workspace_loader_factory = Box::new(DefaultWorkspaceLoaderFactory);
    let maybe_cwd_workspace_loader = workspace_loader_factory.create(find_workspace_dir(&cwd));
}

fn setup_config() -> Result<StackedConfig, ConfigGetError> {
    let mut config_layers = config::default_config_layers();
    let raw_config = config::config_from_environment(config_layers.drain(..));
    let config_env = config::ConfigEnv::from_environment();
    config_env.resolve_config(&raw_config)
}
