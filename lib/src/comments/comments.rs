use const_format::formatcp;
use thiserror::Error;

use crate::bookmark::graph::BookmarkGraph;
use crate::bookmark::Bookmark;
use crate::protos::change_request::forge_meta::Forge;
use crate::protos::change_request::{ChangeRequests, ForgeMeta};

const INDENT: &str = "  ";
const JJ_SPICE_URL: &str = "https://github.com/alejoborbo/jj-spice";
const HEADER_LINE: &str = "This change belongs to the following stack:\n";
const MANAGED_BY_COMMENT_LINE: &str = formatcp!(
    "\n<sub>Change managed by [jj-spice]({}).</sub>",
    JJ_SPICE_URL
);

#[derive(Debug, Error)]
pub enum CommentError {
    #[error("No change request found for bookmark {0}")]
    NoChangeRequestFound(String),
    #[error("No forge metadata found for bookmark {0}")]
    NoForgeMetadataFound(String),
}

/// Struct for creating change request comments.
/// Those comments would be added to the change request to vizualize the stack trace.
pub struct Comment<'a> {
    current_bookmark: &'a Bookmark<'a>,
    graph: &'a BookmarkGraph<'a>,
    change_requests: &'a ChangeRequests,
}

impl<'a> Comment<'a> {
    pub fn new(
        current_bookmark: &'a Bookmark<'a>,
        graph: &'a BookmarkGraph<'a>,
        change_requests: &'a ChangeRequests,
    ) -> Comment<'a> {
        Comment {
            current_bookmark,
            graph,
            change_requests,
        }
    }

    pub fn to_string(&self) -> Result<String, CommentError> {
        let mut comment = String::from(HEADER_LINE);

        // A queue of bookmark, and the depth of the bookmark in the stack trace
        let mut queue = Vec::from_iter(self.graph.root_bookmarks.iter().map(|b| (b.clone(), 0)));
        let mut visited = Vec::new();

        // Go through the bookmarks in a depth first order, and add comments to the change request
        while let Some((bookmark, depth)) = queue.pop() {
            if visited.contains(&bookmark) {
                continue;
            }
            visited.push(bookmark.clone());

            // Fetch the forge metadata of the bookmark
            let meta = self
                .change_requests
                .get(&bookmark)
                .ok_or_else(|| CommentError::NoChangeRequestFound(bookmark.clone()))?;

            // Add the comment to the change request
            let indent = INDENT.repeat(depth);
            let id = self
                .forge_meta_id(meta)
                .ok_or_else(|| CommentError::NoForgeMetadataFound(bookmark.clone()))?;

            // Add the change request to the comment
            comment.push_str(format!("{}- {}", indent, id).as_str());
            if bookmark == self.current_bookmark.name() {
                comment.push_str(" 👈 you are here!");
            }
            comment.push('\n');

            // Add the next bookmarks of the bookmark to the queue
            for next in self.graph.descendants_for(&bookmark) {
                queue.push((next.target.to_string().clone(), depth + 1));
            }
        }

        // Add a link to the repo URL
        comment.push_str(MANAGED_BY_COMMENT_LINE);

        Ok(comment)
    }

    fn forge_meta_id(&self, meta: &ForgeMeta) -> Option<String> {
        match &meta.forge {
            Some(Forge::Github(gh)) => Some(format!("#{}", gh.number)),
            _ => None,
        }
    }
}
