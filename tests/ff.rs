extern crate git2;
extern crate gws2;
extern crate tempdir;

mod util;

use std::collections::BTreeSet;
use std::collections::HashSet;
use std::hash::Hash;
use std::path::Path;

use git2::Commit;
use git2::Repository;

use gws2::color::palette::Palette;
use gws2::commands::common::RepositoryCommand;
use gws2::commands::fetch::Fetch;
use gws2::commands::ff::FastForward;
use gws2::commands::status::Status;
use gws2::config::data::Workspace;
use gws2::data::status::BranchStatus;
use gws2::data::status::DirtyState;
use gws2::data::status::RepositoryStatus;

use util::in_example_workspace;
use util::Error;

pub fn tree_set<I, T>(items: I) -> BTreeSet<T>
where
    I: IntoIterator<Item = T>,
    T: Ord,
{
    items.into_iter().collect()
}

pub fn hash_set<I, T>(items: I) -> HashSet<T>
where
    I: IntoIterator<Item = T>,
    T: Eq + Hash,
{
    items.into_iter().collect()
}

#[test]
fn ff_gets_refs_and_updates_heads() {
    fn resolve_ref<'repo>(
        name: &str,
        repo: &'repo Repository,
        branch_type: git2::BranchType,
    ) -> Result<Commit<'repo>, Error> {
        Ok(repo
            .find_branch(name, branch_type)?
            .get()
            .peel_to_commit()?)
    }

    in_example_workspace(|working_dir, workspace: Workspace| {
        let project_path = "new_commit/unfetched_remote";

        let command: FastForward = FastForward {
            fetch_command: Fetch {
                status_command: Status {
                    only_changes: false,
                },
                projects: HashSet::new(),
            },
        };

        let repo: Repository = Repository::open(working_dir.join(project_path))?;

        let master_reference_before: Commit =
            resolve_ref("master", &repo, git2::BranchType::Local)?;
        let master2_reference_before: Commit =
            resolve_ref("master2", &repo, git2::BranchType::Local)?;

        let origin_master_reference_before: Commit =
            resolve_ref("origin/master", &repo, git2::BranchType::Remote)?;
        let ahead_master_reference_before: Commit =
            resolve_ref("ahead/master", &repo, git2::BranchType::Remote)?;

        command
            .run(working_dir, &workspace, &Palette::default())
            .expect("Fast-forward command failed");

        let master_reference_after: Commit = resolve_ref("master", &repo, git2::BranchType::Local)?;
        let master2_reference_after: Commit =
            resolve_ref("master2", &repo, git2::BranchType::Local)?;

        let origin_master_reference_after: Commit =
            resolve_ref("origin/master", &repo, git2::BranchType::Remote)?;
        let ahead_master_reference_after: Commit =
            resolve_ref("ahead/master", &repo, git2::BranchType::Remote)?;

        assert_eq!(master_reference_after.id(), master_reference_before.id());
        assert_eq!(
            origin_master_reference_after.id(),
            origin_master_reference_before.id()
        );

        assert_ne!(master2_reference_after.id(), master2_reference_before.id());
        assert_eq!(
            master2_reference_after.parents().next().unwrap().id(),
            master2_reference_before.id()
        );
        assert_ne!(
            ahead_master_reference_after.id(),
            ahead_master_reference_before.id()
        );
        assert_eq!(
            ahead_master_reference_after.parents().next().unwrap().id(),
            ahead_master_reference_before.id()
        );

        Ok(())
    });
}

fn run_test(
    working_dir: &Path,
    workspace: Workspace,
    projects: HashSet<String>,
    should_ff: bool,
) -> Result<(), Error> {
    let project_path = "new_commit/unfetched_remote";

    let command: FastForward = FastForward {
        fetch_command: Fetch {
            status_command: Status {
                only_changes: false,
            },
            projects,
        },
    };

    let status_report = command.run_command(working_dir, &workspace);

    for (project, project_status) in status_report {
        if project.path == project_path {
            let project_status = project_status.unwrap();
            for branch_status in project_status {
                if branch_status.name == "master2" {
                    assert_eq!(branch_status.fast_forwarded, should_ff);
                } else {
                    assert_eq!(branch_status.fast_forwarded, false);
                }
            }
        }
    }

    Ok(())
}

#[test]
fn ff_fetches_all_projects_if_none_are_named() {
    in_example_workspace(|working_dir, workspace: Workspace| {
        run_test(working_dir, workspace, HashSet::new(), true)
    });
}

#[test]
fn ff_fetches_only_named_projects_if_any_are_named() {
    in_example_workspace(|working_dir, workspace: Workspace| {
        run_test(
            working_dir,
            workspace,
            hash_set(vec!["new_commit/local".to_string()]),
            false,
        )
    });

    in_example_workspace(|working_dir, workspace: Workspace| {
        run_test(
            working_dir,
            workspace,
            hash_set(vec!["new_commit/unfetched_remote".to_string()]),
            true,
        )
    })
}

#[test]
fn ff_produces_correct_data_structure() {
    in_example_workspace(|working_dir, workspace| {
        let command: FastForward = FastForward {
            fetch_command: Fetch {
                status_command: Status {
                    only_changes: false,
                },
                projects: HashSet::new(),
            },
        };

        let status_report: Vec<Result<RepositoryStatus, gws2::commands::error::Error>> = command
            .run_command(working_dir, &workspace)
            .into_iter()
            .map(|(_, status)| status)
            .collect();

        assert_eq!(
            status_report,
            vec![
                // changes/changed_files
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::UncommittedChanges,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // changes/new_files
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::UntrackedFiles,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // clean
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // missing_repository
                Err(gws2::commands::error::Error::RepositoryMissing),
                // missing_repository_2
                Err(gws2::commands::error::Error::RepositoryMissing),
                // new_commit/local
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: true,
                        in_sync: Some(false),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    }
                ])),
                // new_commit/remote
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(false), // It was not in sync before fast-forward
                        upstream_fetched: false,
                        fast_forwarded: true,
                    }
                ])),
                // new_commit/unfetched_remote
                Ok(tree_set(vec![
                    BranchStatus {
                        name: "master".to_string(),
                        upstream_name: Some("origin/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: true,
                        in_sync: Some(true),
                        upstream_fetched: false,
                        fast_forwarded: false,
                    },
                    BranchStatus {
                        name: "master2".to_string(),
                        upstream_name: Some("ahead/master".to_string()),
                        dirty: DirtyState::Clean,
                        is_head: false,
                        in_sync: Some(true), // It was in sync before fetching
                        upstream_fetched: true,
                        fast_forwarded: true,
                    }
                ])),
            ]
        );

        Ok(())
    });
}
