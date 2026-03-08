use std::io::Write as _;

use jj_cli::description_util::TextEditor;
use jj_lib::backend::CommitId;

use crate::bookmark::graph::BookmarkGraph;
use crate::commands::env::SpiceEnv;
use crate::forge::{CreateParams, Forge};

/// Create change requests for each bookmark in the current stack (trunk..@).
pub async fn run(
    env: &SpiceEnv,
    forge: &dyn Forge,
    trunk: &CommitId,
    head: &CommitId,
    trunk_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let graph = BookmarkGraph::new(env.repo.as_ref(), trunk, head)?;
    let iter_graph = graph.iter_graph()?;

    let text_editor = TextEditor::from_settings(&env.settings)?;

    for bookmark_node in iter_graph {
        let bookmark = bookmark_node.bookmark();
        let ascendants = bookmark_node.ascendants();

        writeln!(
            env.ui.stdout_formatter(),
            "Creating change request for: {}",
            bookmark.name()
        )?;

        let base_bookmark = if ascendants.len() > 1 {
            println!("Ascendants: {:#?}", ascendants);
            let index = env
                .ui
                .prompt_choice("Select base bookmark", ascendants, None)?;
            ascendants[index].as_str()
        } else if let Some(b) = ascendants.first() {
            b.as_str()
        } else {
            trunk_name
        };

        writeln!(
            env.ui.stdout_formatter(),
            "Base bookmark: {}",
            base_bookmark
        )?;

        let title = env.ui.prompt("Title")?;
        let description = text_editor.edit_str("", Some(".md"))?;
        let is_draft = env.ui.prompt_yes_no("Draft?", Some(false))?;

        let params = CreateParams {
            source_branch: bookmark.name(),
            target_branch: base_bookmark,
            title: &title,
            body: Some(&description),
            is_draft,
        };

        let cr = forge.create(params).await?;

        writeln!(
            env.ui.stdout_formatter(),
            "Created change request: {}",
            cr.url()
        )?;
    }

    Ok(())
}
