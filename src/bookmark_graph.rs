use std::collections::{HashMap, HashSet};

use jj_lib::{
    backend::CommitId,
    dag_walk::topo_order_forward,
    graph::{GraphEdge, GraphNode, reverse_graph},
    repo::Repo,
    revset::{
        Revset, RevsetEvaluationError, RevsetExpression, RevsetExtensions, RevsetResolutionError,
        SymbolResolver, UserRevsetExpression,
    },
    workspace::Workspace,
};
use thiserror::Error;

use crate::bookmark::Bookmark;

#[derive(Clone, Debug)]
pub struct BookmarkNode {
    bookmark: Bookmark,
    ascendants: Vec<String>,
}

impl BookmarkNode {
    pub fn new(bookmark: Bookmark) -> Self {
        Self {
            bookmark,
            ascendants: Vec::new(),
        }
    }

    pub fn bookmark(&self) -> &Bookmark {
        &self.bookmark
    }

    pub fn name(&self) -> &str {
        self.bookmark.name()
    }

    pub fn ascendants(&self) -> &[String] {
        &self.ascendants
    }

    pub fn add_ascendant(&mut self, ascendant: String) {
        self.ascendants.push(ascendant);
    }
}

#[derive(Debug, Error)]
pub enum BookmarkGraphError {
    #[error("revset evaluation failed")]
    RevsetEvaluation(#[from] RevsetEvaluationError),
    #[error("revset resolution failed")]
    RevsetResolution(#[from] RevsetResolutionError),
    #[error("no root commit found in branch")]
    NoRootCommit,
    #[error("cycle detected in bookmark graph")]
    Cycle,
}

#[derive(Debug)]
pub struct BookmarkGraph {
    nodes: HashMap<String, BookmarkNode>,
    edges: HashMap<String, Vec<GraphEdge<String>>>,
    head_bookmarks: HashSet<String>,
}

impl BookmarkGraph {
    pub fn new(
        repo: &dyn Repo,
        workspace: &Workspace,
        trunk_name: &str,
    ) -> Result<Self, BookmarkGraphError> {
        let bookmarks_per_commit = Self::build_bookmark_commit_map(repo);
        let reversed = Self::build_reversed_commit_graph(repo, workspace, trunk_name)?;
        let (nodes, edges) = Self::build_bookmark_graph(&reversed, &bookmarks_per_commit);
        let head_bookmarks = Self::find_head_bookmarks(&edges);
        Ok(Self {
            nodes,
            edges,
            head_bookmarks,
        })
    }

    pub fn iter_graph(&self) -> Result<impl Iterator<Item = &BookmarkNode>, BookmarkGraphError> {
        let result = topo_order_forward(
            self.head_bookmarks.iter().map(|name| &self.nodes[name]),
            |node| node.name(),
            |&node| {
                self.edges[node.name()]
                    .iter()
                    .map(|e| &self.nodes[e.target.as_str()])
            },
            |_| BookmarkGraphError::Cycle,
        )?;
        Ok(result.into_iter())
    }

    fn symbol_resolver(repo: &dyn Repo) -> SymbolResolver<'_> {
        SymbolResolver::new(repo, RevsetExtensions::default().symbol_resolvers())
    }

    fn find_root_commit(
        repo: &dyn Repo,
        workspace: &Workspace,
        trunk_name: &str,
    ) -> Result<CommitId, BookmarkGraphError> {
        let trunk = UserRevsetExpression::symbol(trunk_name.to_string());
        let wc = RevsetExpression::working_copy(workspace.workspace_name().to_owned());
        let branch_commits = trunk.range(&wc);
        let first_mutable = branch_commits
            .roots()
            .resolve_user_expression(repo, &Self::symbol_resolver(repo))?;
        let expression = first_mutable.evaluate(repo)?;
        expression
            .iter()
            .next()
            .and_then(|r| r.ok())
            .ok_or(BookmarkGraphError::NoRootCommit)
    }

    fn evaluate_branch_commits<'a>(
        repo: &'a dyn Repo,
        workspace: &Workspace,
        trunk_name: &str,
    ) -> Result<Box<dyn Revset + 'a>, BookmarkGraphError> {
        let first_commit = Self::find_root_commit(repo, workspace, trunk_name)?;
        let expression = RevsetExpression::commit(first_commit).descendants();
        Ok(expression.evaluate(repo)?)
    }

    fn build_bookmark_commit_map(repo: &dyn Repo) -> HashMap<CommitId, Bookmark> {
        let mut map = HashMap::new();
        repo.view().bookmarks().for_each(|(ref_name, ref_target)| {
            if let Some(commit_id) = ref_target.local_target.as_normal() {
                map.entry(commit_id.clone())
                    .or_insert_with(|| Bookmark::new(ref_name.as_str().to_string()));
            }
        });
        map
    }

    fn build_reversed_commit_graph(
        repo: &dyn Repo,
        workspace: &Workspace,
        trunk_name: &str,
    ) -> Result<Vec<GraphNode<CommitId>>, BookmarkGraphError> {
        let revset = Self::evaluate_branch_commits(repo, workspace, trunk_name)?;
        Ok(reverse_graph(revset.iter_graph(), |id| id).expect("commit graph should be acyclic"))
    }

    fn find_head_commits(reversed: &[GraphNode<CommitId>]) -> Vec<&CommitId> {
        let all_edge_targets: HashSet<&CommitId> = reversed
            .iter()
            .flat_map(|(_, edges)| edges.iter().map(|e| &e.target))
            .collect();

        reversed
            .iter()
            .map(|(id, _)| id)
            .filter(|id| !all_edge_targets.contains(id))
            .collect()
    }

    fn build_bookmark_graph(
        reversed: &[GraphNode<CommitId>],
        bookmarks_per_commit: &HashMap<CommitId, Bookmark>,
    ) -> (
        HashMap<String, BookmarkNode>,
        HashMap<String, Vec<GraphEdge<String>>>,
    ) {
        let commit_index: HashMap<&CommitId, &GraphNode<CommitId>> =
            reversed.iter().map(|node| (&node.0, node)).collect();

        let head_commits = Self::find_head_commits(reversed);

        let mut nodes: HashMap<String, BookmarkNode> = HashMap::new();
        let mut edges: HashMap<String, Vec<GraphEdge<String>>> = HashMap::new();
        let mut visited: HashSet<&CommitId> = HashSet::new();

        let mut stack: Vec<(&CommitId, Option<&str>)> =
            head_commits.into_iter().map(|c| (c, None)).collect();

        while let Some((commit_id, parent_name)) = stack.pop() {
            if !visited.insert(commit_id) {
                continue;
            }

            let maybe_bookmark = bookmarks_per_commit.get(commit_id);

            if let Some(bookmark) = maybe_bookmark {
                let name = bookmark.name().to_string();

                if !nodes.contains_key(&name) {
                    let mut node = BookmarkNode::new(bookmark.clone());

                    // Ascendants = parent's ascendants + parent
                    if let Some(pn) = parent_name {
                        if let Some(parent_node) = nodes.get(pn) {
                            for asc in parent_node.ascendants() {
                                node.add_ascendant(asc.clone());
                            }
                        }
                        node.add_ascendant(pn.to_string());
                    }

                    nodes.insert(name.clone(), node);
                }

                let edge_list = edges.entry(name.clone()).or_default();
                if let Some(pn) = parent_name
                    && pn != name
                {
                    edge_list.push(GraphEdge::direct(pn.to_string()));
                }
            }

            let next_name = maybe_bookmark.map(|b| b.name()).or(parent_name);

            if let Some(node) = commit_index.get(commit_id) {
                for edge in &node.1 {
                    stack.push((&edge.target, next_name));
                }
            }
        }

        (nodes, edges)
    }

    fn find_head_bookmarks(edges: &HashMap<String, Vec<GraphEdge<String>>>) -> HashSet<String> {
        let all_edge_targets: HashSet<&str> = edges
            .values()
            .flatten()
            .map(|e| e.target.as_str())
            .collect();

        edges
            .keys()
            .filter(|name| !all_edge_targets.contains(name.as_str()))
            .cloned()
            .collect()
    }
}
