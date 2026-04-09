use std::collections::BTreeMap;

use but_core::RefMetadata;
use but_rebase::graph_rebase::SuccessfulRebase;

use crate::types::WorkspaceState;

/// Build a [`WorkspaceState`] from an already-prepared overlayed graph.
///
/// Use this when the caller already has a graph describing the workspace after the
/// intended operation, regardless of whether that graph came from a preview, a
/// materialized rebase, or another graph-producing workflow. The caller is
/// responsible for supplying the matching `replaced_commits` map for that graph.
///
/// This is the most direct constructor in this module and is the right choice when
/// there is no need to inspect or materialize a [`SuccessfulRebase`].
pub(crate) fn from_overlayed_graph(
    graph: but_graph::Graph,
    repo: &gix::Repository,
    cache: &mut but_db::CacheHandle,
    replaced_commits: BTreeMap<gix::ObjectId, gix::ObjectId>,
) -> anyhow::Result<WorkspaceState> {
    let head_info = but_workspace::ref_info_from_graph(
        graph,
        repo,
        but_workspace::ref_info::Options {
            traversal: but_graph::init::Options::limited(),
            expensive_commit_info: true,
        },
        cache,
    )?
    .pruned_to_entrypoint();

    Ok(WorkspaceState::new(replaced_commits, head_info))
}

/// Build a preview [`WorkspaceState`] from a successful rebase without materializing it.
///
/// Use this when the caller needs to report the post-rebase workspace layout before
/// writing the rebase result back to the repository, such as dry-run flows or
/// operations that intentionally preview the outcome first and materialize later.
///
/// The `replaced_commits` map should describe the commit rewrites visible in the
/// preview graph, which typically comes from `rebase.history.commit_mappings()`.
pub(crate) fn from_rebase_preview<M: RefMetadata>(
    rebase: &SuccessfulRebase<'_, '_, M>,
    cache: &mut but_db::CacheHandle,
    replaced_commits: BTreeMap<gix::ObjectId, gix::ObjectId>,
) -> anyhow::Result<WorkspaceState> {
    from_overlayed_graph(
        rebase.overlayed_graph()?,
        rebase.repository(),
        cache,
        replaced_commits,
    )
}

/// Build a [`WorkspaceState`] from a successful rebase, materializing it when needed.
///
/// Use this as the default entry point when an operation ends with a
/// [`SuccessfulRebase`] and the API should return the resulting workspace state.
/// When `dry_run` is `true`, this delegates to [`from_rebase_preview`] so the caller
/// sees the projected state without changing the repository. Otherwise it
/// materializes the rebase, then reports the workspace state together with the final
/// commit-replacement mappings returned by the materialized history.
pub(crate) fn from_successful_rebase<M: RefMetadata>(
    rebase: SuccessfulRebase<'_, '_, M>,
    repo: &gix::Repository,
    cache: &mut but_db::CacheHandle,
    dry_run: bool,
) -> anyhow::Result<WorkspaceState> {
    if dry_run {
        return from_rebase_preview(&rebase, cache, rebase.history.commit_mappings());
    }

    let graph = rebase.overlayed_graph()?;
    let materialized = rebase.materialize()?;
    from_overlayed_graph(graph, repo, cache, materialized.history.commit_mappings())
}
