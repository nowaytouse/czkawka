use std::collections::HashSet;
use std::path::PathBuf;

use super::DuplicateFinder;
use crate::common::model::CheckingMethod;

impl DuplicateFinder {
    /// Retains visible name matches after frontend actions, without rescanning or changing files.
    /// Reference paths must be included; a group without its reference or enough matches is removed.
    /// Returns the active method without modifying results if this is not a name-based scan.
    pub fn retain_name_results(&mut self, visible_paths: &HashSet<PathBuf>) -> Result<(), CheckingMethod> {
        if self.params.check_method != CheckingMethod::Name {
            return Err(self.params.check_method);
        }
        self.files_with_identical_names.retain(|_, entries| {
            entries.retain(|entry| visible_paths.contains(&entry.path));
            entries.len() > 1
        });
        self.files_with_identical_names_referenced.retain(|_, (reference, entries)| {
            entries.retain(|entry| visible_paths.contains(&entry.path));
            visible_paths.contains(&reference.path) && !entries.is_empty()
        });
        self.calculate_name_stats();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::model::HashType;
    use crate::common::traits::PrintResults;
    use crate::tools::duplicate::{DuplicateEntry, DuplicateFinderParameters};

    fn entry(folder: &str, name: &str) -> DuplicateEntry {
        DuplicateEntry {
            path: std::env::temp_dir().join("czkawka-name-results").join(folder).join(name),
            ..DuplicateEntry::default()
        }
    }

    fn finder() -> DuplicateFinder {
        DuplicateFinder::new(DuplicateFinderParameters::new(CheckingMethod::Name, HashType::Blake3, false, 0, 0, true))
    }

    #[test]
    fn retains_visible_filename_groups_and_recounts_export_after_actions() {
        let mut finder = finder();
        let first = vec![entry("a", "renamed.txt"), entry("b", "renamed.txt")];
        let second = vec![entry("a", "kept.txt"), entry("b", "kept.txt"), entry("c", "kept.txt")];
        let visible = HashSet::from([first[1].path.clone(), second[0].path.clone(), second[1].path.clone()]);
        finder.files_with_identical_names.insert("renamed.txt".into(), first);
        finder.files_with_identical_names.insert("kept.txt".into(), second);
        finder.information.number_of_groups_by_name = 20;
        finder.information.number_of_duplicated_files_by_name = 30;

        finder.retain_name_results(&visible).unwrap();
        finder.retain_name_results(&visible).unwrap();
        assert_eq!(finder.get_information().number_of_groups_by_name, 1);
        assert_eq!(finder.get_information().number_of_duplicated_files_by_name, 1);
        assert_eq!(finder.get_files_sorted_by_names()["kept.txt"].len(), 2);
        let mut output = Vec::new();
        finder.write_results(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("kept.txt"));
        assert!(!output.contains("renamed.txt"));

        finder.retain_name_results(&HashSet::new()).unwrap();
        assert!(finder.get_files_sorted_by_names().is_empty());
        assert_eq!(finder.get_information().number_of_groups_by_name, 0);
        assert_eq!(finder.get_information().number_of_duplicated_files_by_name, 0);
    }

    #[test]
    fn retains_reference_only_while_reference_and_a_match_are_visible() {
        let mut finder = finder();
        finder.common_data.use_reference_folders = true;
        let reference = entry("reference", "same.txt");
        let other = entry("other", "same.txt");
        let removed = entry("removed", "same.txt");
        let paths = HashSet::from([reference.path.clone(), other.path.clone()]);
        finder
            .files_with_identical_names_referenced
            .insert("reference".into(), (reference, vec![other.clone(), removed]));

        finder.retain_name_results(&paths).unwrap();
        finder.retain_name_results(&paths).unwrap();
        assert_eq!(finder.get_files_with_identical_name_referenced()["reference"].1.len(), 1);
        assert_eq!(finder.get_information().number_of_groups_by_name, 1);
        assert_eq!(finder.get_information().number_of_duplicated_files_by_name, 1);

        finder.retain_name_results(&HashSet::from([other.path])).unwrap();
        assert!(finder.get_files_with_identical_name_referenced().is_empty());
        assert_eq!(finder.get_information().number_of_groups_by_name, 0);
        assert_eq!(finder.get_information().number_of_duplicated_files_by_name, 0);
    }

    #[test]
    fn rejects_non_name_retention_without_changing_results() {
        let mut finder = finder();
        finder.params.check_method = CheckingMethod::Hash;
        finder
            .files_with_identical_names
            .insert("same.txt".into(), vec![entry("a", "same.txt"), entry("b", "same.txt")]);
        finder.information.number_of_groups_by_name = 1;
        assert_eq!(finder.retain_name_results(&HashSet::new()), Err(CheckingMethod::Hash));
        assert_eq!(finder.get_files_sorted_by_names()["same.txt"].len(), 2);
        assert_eq!(finder.get_information().number_of_groups_by_name, 1);
    }
}
