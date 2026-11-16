mod execute;
mod plan;

use std::fmt::{self, Write as _};
use std::io;
use std::path::{Path, PathBuf};

pub(crate) use plan::Plan;

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
        format!(
            "Keep {} at their current paths · Move {} within A · Archive {} files and {} folders · Create {} folders · Missing {} · Ambiguous or blocked {}",
            self.unchanged,
            self.move_files.len(),
            self.archive_files.len(),
            self.archive_dirs.len(),
            self.create_dirs.len(),
            self.missing.len(),
            self.blocked.len(),
        )
    }

    pub(crate) fn preview_rows(&self) -> Vec<String> {
        let mut rows = Vec::new();
        rows.extend(self.archive_files.iter().map(|path| format!("Archive file: A/{path:?}")));
        rows.extend(self.create_dirs.iter().map(|path| format!("Create folder: A/{path:?}")));
        rows.extend(self.move_files.iter().map(|item| format!("Move: A/{:?} → A/{:?}", item.source, item.destination)));
        rows.extend(self.archive_dirs.iter().map(|path| format!("Archive folder: A/{path:?}")));
        rows.extend(self.missing.iter().map(|path| format!("Missing file: B/{path:?}")));
        rows.extend(self.blocked.iter().map(|path| format!("Unresolved destination: B/{path:?}")));
        rows.extend(self.ambiguous.iter().map(|path| format!("Retained candidate: A/{path:?}")));
        rows
    }
}

pub(crate) struct Report {
    archive: PathBuf,
    completed: usize,
    missing: usize,
    unresolved: usize,
}

impl fmt::Display for Report {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "Completed {} operations. Recovery record: {:?}", self.completed, self.archive)?;
        if self.missing != 0 || self.unresolved != 0 {
            output.write_char('\n')?;
            write!(
                output,
                "{} missing files and {} unresolved destinations remain. See the preview or plan.json.",
                self.missing, self.unresolved
            )?;
        }
        Ok(())
    }
}
