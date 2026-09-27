use std::fs;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tempfile::TempDir;

use crate::common::model::{CheckingMethod, HashType};
use crate::common::tool_data::CommonData;
use crate::common::traits::Search;
use crate::tools::duplicate::{DuplicateFinder, DuplicateFinderParameters};

#[test]
fn test_find_duplicates_by_hash() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create duplicate files with same content
    fs::write(path.join("file1.txt"), b"duplicate content").unwrap();
    fs::write(path.join("file2.txt"), b"duplicate content").unwrap();
    fs::write(path.join("unique.txt"), b"unique content").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Hash, HashType::Blake3, false, 0, 0, true);

    let mut finder = DuplicateFinder::new(params);
    finder.set_included_paths(vec![path.to_path_buf()]);
    finder.set_minimal_file_size(0);
    finder.set_recursive_search(true);
    finder.set_use_cache(false);

    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_hash, 1, "Should find 1 group of duplicates");
    assert_eq!(info.number_of_duplicated_files_by_hash, 1, "Should find 1 duplicate file");
}

#[test]
fn test_find_duplicates_by_size() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create files with same size
    fs::write(path.join("file1.txt"), b"12345").unwrap();
    fs::write(path.join("file2.txt"), b"abcde").unwrap();
    fs::write(path.join("unique.txt"), b"123").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Size, HashType::Blake3, false, 0, 0, true);

    let mut finder = DuplicateFinder::new(params);
    finder.set_included_paths(vec![path.to_path_buf()]);
    finder.set_recursive_search(true);
    finder.set_minimal_file_size(0);
    finder.set_use_cache(false);

    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_size, 1, "Should find 1 group by size");
    assert_eq!(info.number_of_duplicated_files_by_size, 1, "Should find 1 duplicate by size");
}

#[test]
fn test_find_duplicates_by_name() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let dir1 = path.join("dir1");
    let dir2 = path.join("dir2");
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();

    // Create files with same name in different directories
    fs::write(dir1.join("duplicate.txt"), b"content1").unwrap();
    fs::write(dir2.join("duplicate.txt"), b"content2").unwrap();
    fs::write(dir1.join("unique.txt"), b"unique").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Name, HashType::Blake3, false, 0, 0, true);

    let mut finder = DuplicateFinder::new(params);
    finder.set_recursive_search(true);
    finder.set_included_paths(vec![path.to_path_buf()]);
    finder.set_minimal_file_size(0);
    finder.set_use_cache(false);
    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_name, 1, "Should find 1 group by name");
    assert_eq!(info.number_of_duplicated_files_by_name, 1, "Should find 1 duplicate by name");
}

#[test]
fn test_no_duplicates_found() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create unique files
    fs::write(path.join("file1.txt"), b"content1").unwrap();
    fs::write(path.join("file2.txt"), b"content2").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Hash, HashType::Blake3, false, 0, 0, true);

    let mut finder = DuplicateFinder::new(params);
    finder.set_included_paths(vec![path.to_path_buf()]);
    finder.set_recursive_search(true);
    finder.set_use_cache(false);
    finder.set_minimal_file_size(0);

    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_hash, 0, "Should find no duplicate groups");
    assert_eq!(info.lost_space_by_hash, 0, "Should have no lost space");
}

#[test]
fn exact_filename_matches_full_case_sensitive_name_without_comparing_contents() {
    let temp_dir = TempDir::new().unwrap();
    let first_folder = temp_dir.path().join("first");
    let second_folder = temp_dir.path().join("second");
    let case_variant_folder = temp_dir.path().join("case_variant");
    fs::create_dir(&first_folder).unwrap();
    fs::create_dir(&second_folder).unwrap();
    fs::create_dir(&case_variant_folder).unwrap();
    fs::write(first_folder.join("影片.mp4"), b"first contents").unwrap();
    fs::write(second_folder.join("影片.mp4"), b"different and longer contents").unwrap();
    fs::write(case_variant_folder.join("影片.MP4"), b"different extension case").unwrap();
    fs::write(first_folder.join("Clip.mp4"), b"different filename case").unwrap();
    fs::write(second_folder.join("clip.mp4"), b"different filename case").unwrap();
    fs::write(second_folder.join("影片.mov"), b"different extension").unwrap();
    fs::write(second_folder.join("影片 (1).mp4"), b"different filename").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Name, HashType::Blake3, false, 0, 0, true);
    let mut finder = DuplicateFinder::new(params);
    finder.set_included_paths(vec![first_folder.clone(), second_folder.clone(), case_variant_folder]);
    finder.set_minimal_file_size(0);
    finder.search(&Arc::new(AtomicBool::new(false)), None);

    let groups = finder.get_files_sorted_by_names();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups["影片.mp4"].len(), 2);
    assert_ne!(groups["影片.mp4"][0].size, groups["影片.mp4"][1].size);
    assert_eq!(fs::read(first_folder.join("影片.mp4")).unwrap(), b"first contents");
    assert_eq!(fs::read(second_folder.join("影片.mp4")).unwrap(), b"different and longer contents");
}

#[test]
fn exact_filename_respects_media_extensions_and_reference_folders() {
    let temp_dir = TempDir::new().unwrap();
    let reference_folder = temp_dir.path().join("reference");
    let other_folder = temp_dir.path().join("other");
    fs::create_dir(&reference_folder).unwrap();
    fs::create_dir(&other_folder).unwrap();
    for folder in [&reference_folder, &other_folder] {
        fs::write(folder.join("movie.mp4"), b"media").unwrap();
        fs::write(folder.join("document.txt"), b"not media").unwrap();
    }

    let params = DuplicateFinderParameters::new(CheckingMethod::Name, HashType::Blake3, false, 0, 0, true);
    let mut finder = DuplicateFinder::new(params);
    finder.set_included_paths(vec![reference_folder.clone(), other_folder.clone()]);
    finder.set_reference_paths(vec![reference_folder.clone()]);
    finder.set_allowed_extensions(vec!["video".to_string()]);
    finder.set_minimal_file_size(0);
    finder.search(&Arc::new(AtomicBool::new(false)), None);

    let groups = finder.get_files_with_identical_name_referenced();
    assert_eq!(groups.len(), 1);
    let (original, matches) = groups.values().next().unwrap();
    assert_eq!(original.path, reference_folder.join("movie.mp4").canonicalize().unwrap());
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].path, other_folder.join("movie.mp4").canonicalize().unwrap());
    assert_eq!(fs::read(original.path.clone()).unwrap(), b"media");
    assert_eq!(fs::read(matches[0].path.clone()).unwrap(), b"media");
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn filename_modes_skip_non_utf8_names_without_lossy_collisions() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp_dir = TempDir::new().unwrap();
    let first_folder = temp_dir.path().join("first");
    let second_folder = temp_dir.path().join("second");
    fs::create_dir(&first_folder).unwrap();
    fs::create_dir(&second_folder).unwrap();
    let first_invalid = first_folder.join(OsString::from_vec(b"clip\x80.mp4".to_vec()));
    let second_invalid = second_folder.join(OsString::from_vec(b"clip\x81.mp4".to_vec()));
    fs::write(&first_invalid, b"media").unwrap();
    fs::write(&second_invalid, b"media").unwrap();
    fs::write(first_folder.join("clip\u{fffd}.mp4"), b"media").unwrap();
    fs::write(second_folder.join("clip\u{fffd}.mp4"), b"media").unwrap();

    for check_method in [CheckingMethod::Name, CheckingMethod::SizeName] {
        for case_sensitive in [true, false] {
            let params = DuplicateFinderParameters::new(check_method, HashType::Blake3, false, 0, 0, case_sensitive);
            let mut finder = DuplicateFinder::new(params);
            finder.set_included_paths(vec![first_folder.clone(), second_folder.clone()]);
            finder.set_minimal_file_size(0);
            finder.search(&Arc::new(AtomicBool::new(false)), None);

            let groups = match check_method {
                CheckingMethod::Name => finder.get_files_sorted_by_names().values().collect::<Vec<_>>(),
                CheckingMethod::SizeName => finder.get_files_sorted_by_size_name().values().collect::<Vec<_>>(),
                _ => unreachable!(),
            };
            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0].len(), 2);
            assert!(groups[0].iter().all(|file| file.path.file_name().unwrap().to_str().is_some()));
            assert_eq!(finder.get_text_messages().warnings.len(), 2);
        }
    }
    assert_eq!(fs::read(first_invalid).unwrap(), b"media");
    assert_eq!(fs::read(second_invalid).unwrap(), b"media");
}

#[cfg(unix)]
#[test]
fn filename_keys_reject_non_utf8_names_and_report_skipped_files() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;

    use crate::common::model::FileEntry;

    let invalid_files = [b"clip\x80.mp4".to_vec(), b"clip\x81.mp4".to_vec(), b"folder\x82/clip.mp4".to_vec()].map(|name| FileEntry {
        path: PathBuf::from(OsString::from_vec(name)),
        ..Default::default()
    });
    let valid_file = FileEntry {
        path: PathBuf::from("clip\u{fffd}.mp4"),
        ..Default::default()
    };
    for case_sensitive in [true, false] {
        for file in &invalid_files {
            assert_eq!(DuplicateFinder::filename_key(file, case_sensitive), None);
        }
        assert_eq!(DuplicateFinder::filename_key(&valid_file, case_sensitive), Some("clip\u{fffd}.mp4".to_string()));
    }
    let params = DuplicateFinderParameters::new(CheckingMethod::Name, HashType::Blake3, false, 0, 0, true);
    let mut finder = DuplicateFinder::new(params);
    assert_eq!(finder.validate_filename_group(None, &invalid_files), None);
    assert_eq!(finder.get_text_messages().warnings.len(), 3);
    assert!(finder.get_text_messages().warnings.iter().all(|warning| warning.contains("clip")));
}

#[test]
fn test_lost_space_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create 3 files with 100 bytes each, all duplicates
    let content = vec![b'A'; 100];
    fs::write(path.join("file1.txt"), &content).unwrap();
    fs::write(path.join("file2.txt"), &content).unwrap();
    fs::write(path.join("file3.txt"), &content).unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Hash, HashType::Blake3, false, 0, 0, true);

    let mut finder = DuplicateFinder::new(params);
    finder.set_minimal_file_size(0);
    finder.set_use_cache(false);
    finder.set_included_paths(vec![path.to_path_buf()]);

    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.lost_space_by_hash, 200, "Should calculate 200 bytes lost space (2 duplicate files * 100 bytes)");
}
