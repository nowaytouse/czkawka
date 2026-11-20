mod execute;
mod plan;

use std::io;
use std::path::{Path, PathBuf};

pub(crate) use plan::Plan;
use slint::{ModelRc, SharedString, VecModel};

use crate::{SingleMainListModel, flk};

pub(crate) fn preview(a: &Path, b: &Path, c: &Path, match_by_hash: bool) -> io::Result<Plan> {
    let plan = plan::build_plan(a, b, c, match_by_hash)?;
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

    pub(crate) fn preview_rows(&self) -> Vec<[SharedString; 6]> {
        let mut rows = Vec::new();
        let move_file = flk!("folder_mirror_move_file");
        rows.extend(self.move_files.iter().map(|item| {
            self.preview_row(
                &move_file,
                Some(&self.a.join(&item.source)),
                Some(&self.a.join(&item.destination)),
                self.source_hashes.get(&item.source),
            )
        }));
        let create_folder = flk!("folder_mirror_create_folder");
        rows.extend(self.create_dirs.iter().map(|path| self.preview_row(&create_folder, None, Some(&self.a.join(path)), None)));
        for (action, paths) in [
            (flk!("folder_mirror_archive_file"), &self.archive_files),
            (flk!("folder_mirror_archive_folder"), &self.archive_dirs),
        ] {
            rows.extend(
                paths
                    .iter()
                    .map(|path| self.preview_row(&action, Some(&self.a.join(path)), Some(&self.c.join("...").join("files").join(path)), None)),
            );
        }
        for (action, paths) in [(flk!("folder_mirror_missing_file"), &self.missing), (flk!("folder_mirror_unresolved"), &self.blocked)] {
            rows.extend(
                paths
                    .iter()
                    .map(|path| self.preview_row(&action, None, Some(&self.b.join(path)), self.template_hashes.get(path))),
            );
        }
        let retained = flk!("folder_mirror_retained");
        rows.extend(
            self.ambiguous
                .iter()
                .map(|path| self.preview_row(&retained, Some(&self.a.join(path)), None, self.source_hashes.get(path))),
        );
        rows
    }

    fn preview_row(&self, action: &str, source: Option<&Path>, destination: Option<&Path>, hash: Option<&String>) -> [SharedString; 6] {
        let (source_label, source_path) = self.preview_path(source);
        let (destination_label, destination_path) = self.preview_path(destination);
        [
            action.into(),
            source_label,
            destination_label,
            hash.map_or_default(|text| text.as_str().into()),
            source_path,
            destination_path,
        ]
    }

    fn preview_path(&self, path: Option<&Path>) -> (SharedString, SharedString) {
        let Some(path) = path else { return Default::default() };
        let full: SharedString = path.to_string_lossy().as_ref().into();
        for (label, root) in [("A", &self.a), ("B", &self.b), ("C", &self.c)] {
            if let Ok(relative) = path.strip_prefix(root) {
                return (format!("{label}/{}", relative.display()).into(), full);
            }
        }
        (full.clone(), full)
    }
}

pub(crate) fn preview_model(rows: Vec<[SharedString; 6]>) -> ModelRc<SingleMainListModel> {
    ModelRc::new(VecModel::from(
        rows.into_iter()
            .map(|columns| SingleMainListModel {
                val_str: ModelRc::new(VecModel::from(Vec::from(columns))),
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    ))
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
