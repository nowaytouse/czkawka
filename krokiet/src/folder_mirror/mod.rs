mod execute;
mod plan;

use std::io;
use std::path::{Path, PathBuf};

pub(crate) use plan::Plan;
use slint::SharedString;

use crate::{MirrorPreviewRow, flk};

pub(crate) fn preview(a: &Path, b: &Path, c: &Path) -> io::Result<Plan> {
    let plan = plan::build_plan(a, b, c)?;
    execute::check_volume(&plan)?;
    Ok(plan)
}

pub(crate) fn apply(plan: &Plan, reference_dirs: &[PathBuf]) -> io::Result<Report> {
    let protected = crate::file_protection::PROTECTED_FILES
        .lock()
        .map_err(|_| io::Error::other("File protection is unavailable"))?;
    if let Some(error) = protected.storage_error() {
        return Err(io::Error::other(format!("File protection is unavailable: {error}")));
    }
    execute::apply(plan, reference_dirs, &protected.files)
}

impl Plan {
    pub(crate) fn has_changes(&self) -> bool {
        !(self.move_files.is_empty() && self.archive_files.is_empty() && self.create_dirs.is_empty() && self.archive_dirs.is_empty())
    }

    pub(crate) fn summary(&self) -> String {
        flk!(
            "folder_mirror_summary",
            kept = self.unchanged,
            moved = self.move_files.len(),
            files = self.archive_files.len(),
            folders = self.archive_dirs.len(),
            created = self.create_dirs.len(),
            missing = self.missing.len(),
            unresolved = self.blocked.len(),
        )
    }

    pub(crate) fn preview_rows(&self) -> Vec<MirrorPreviewRow> {
        let mut rows = Vec::new();
        let move_file = flk!("folder_mirror_move_file");
        rows.extend(
            self.move_files
                .iter()
                .map(|item| preview_row(&move_file, format!("A/{}", item.source.display()), format!("A/{}", item.destination.display()))),
        );
        let create_folder = flk!("folder_mirror_create_folder");
        rows.extend(
            self.create_dirs
                .iter()
                .map(|path| preview_row(&create_folder, String::new(), format!("A/{}", path.display()))),
        );
        for (action, paths) in [
            (flk!("folder_mirror_archive_file"), &self.archive_files),
            (flk!("folder_mirror_archive_folder"), &self.archive_dirs),
        ] {
            rows.extend(
                paths
                    .iter()
                    .map(|path| preview_row(&action, format!("A/{}", path.display()), format!("C/.../files/{}", path.display()))),
            );
        }
        for (action, paths) in [(flk!("folder_mirror_missing_file"), &self.missing), (flk!("folder_mirror_unresolved"), &self.blocked)] {
            rows.extend(paths.iter().map(|path| preview_row(&action, String::new(), format!("B/{}", path.display()))));
        }
        let retained = flk!("folder_mirror_retained");
        rows.extend(self.ambiguous.iter().map(|path| preview_row(&retained, format!("A/{}", path.display()), String::new())));
        rows
    }
}

fn preview_row(action: &str, source_path: String, destination_path: String) -> MirrorPreviewRow {
    MirrorPreviewRow {
        action: SharedString::from(action),
        source_path: source_path.into(),
        destination_path: destination_path.into(),
    }
}

pub(crate) struct Report {
    archive: PathBuf,
    completed: usize,
    missing: usize,
    unresolved: usize,
}

impl Report {
    pub(crate) fn summary(&self) -> String {
        flk!(
            "folder_mirror_report",
            completed = self.completed,
            archive = self.archive.display().to_string(),
            missing = self.missing,
            unresolved = self.unresolved,
        )
    }
}
