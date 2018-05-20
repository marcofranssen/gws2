use std::path::Path;

use ansi_term::ANSIString;

use cli::exit_codes;
use color::palette::Palette;
use config::read::read_workspace_file;
use data::status::BranchStatus;
use data::status::DirtyState;
use data::status::ProjectStatusMethods;
use super::common::format_branch_line;
use super::common::format_message_line;
use super::common::format_project_header;


trait BranchStatusPrinting {
    fn describe_sync_status(&self, palette: &Palette) -> ANSIString;
    fn describe_status(&self, palette: &Palette) -> ANSIString;
    fn describe_full(&self, palette: &Palette) -> String;
}

impl BranchStatusPrinting for BranchStatus {
    fn describe_sync_status(&self, palette: &Palette) -> ANSIString {
        match self.in_sync {
            Some(true) => palette.clean.paint("Clean".to_string()),
            Some(false) => palette.dirty.paint(format!("Not in sync with {}", self.upstream_name)),
            None => palette.missing.paint(format!("No remote branch {}", self.upstream_name)),
        }
    }

    fn describe_status(&self, palette: &Palette) -> ANSIString {
        if self.is_head {
            match self.dirty {
                DirtyState::Clean => self.describe_sync_status(palette),
                DirtyState::UncommittedChanges => palette.dirty.paint("Dirty (Uncommitted changes)".to_string()),
                DirtyState::UntrackedFiles => palette.dirty.paint("Dirty (Untracked files)".to_string()),
            }
        } else {
            self.describe_sync_status(palette)
        }
    }

    fn describe_full(&self, palette: &Palette) -> String {
        format_branch_line(
            &palette,
            self.is_head,
            &self.name,
            &self.describe_status(&palette)
        )
    }
}

pub fn run(palette: &Palette) -> Result<i32, ::git2::Error> {

    let ws_file_path = Path::new(".projects.gws");
    let ws = read_workspace_file(ws_file_path).unwrap();

    let mut exit_code: exit_codes::ExitCode = exit_codes::OK;

    for project in ws.projects {
        println!("{}", format_project_header(&project, &palette));

        match project.status() {
            Some(Ok(status)) => {
                for b in status {
                    println!("{}", b.describe_full(&palette));
                }
            },
            Some(Err(err)) => {
                eprintln!("{}", palette.error.paint(format!("Failed to compute status: {}", err)));
                exit_code = exit_codes::STATUS_PROJECT_FAILED;
            }
            None => {
                println!("{}", palette.missing.paint(format_message_line("Missing repository")));
            },
        }
    }

    Ok(exit_code)
}
