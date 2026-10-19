use std::path::Path;
use std::process::Output;
use std::{fs, io};

pub fn remove_previous_classes(path: &Path) -> io::Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io::Error::new(
            error.kind(),
            format!("failed to remove previous Java class directory {}: {error}", path.display()),
        )),
    }
}

pub fn tool_output_diagnostics(tool: &str, output: &Output) -> Result<Vec<String>, String> {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let warnings = [(&stdout, "stdout"), (&stderr, "stderr")]
        .into_iter()
        .flat_map(|(stream, name)| stream.lines().filter(|line| !line.trim().is_empty()).map(move |line| format!("{tool} {name}: {line}")))
        .collect();

    if output.status.success() {
        Ok(warnings)
    } else {
        Err(format!("{tool} failed with status {}\nstdout:\n{stdout}\nstderr:\n{stderr}", output.status))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    #[cfg(any(unix, windows))]
    use std::process::{ExitStatus, Output};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{remove_previous_classes, tool_output_diagnostics};

    static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

    fn exclusive_temp_root() -> PathBuf {
        loop {
            let root = std::env::temp_dir().join(format!("cedinia-build-support-{}-{}", std::process::id(), NEXT_TEMP.fetch_add(1, Ordering::Relaxed)));
            match fs::create_dir(&root) {
                Ok(()) => return root,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("failed to create test fixture {}: {error}", root.display()),
            }
        }
    }

    #[test]
    fn removes_obsolete_class_directory_and_rejects_a_file_at_its_path() {
        let root = exclusive_temp_root();
        let classes = root.join("classes");
        fs::create_dir_all(&classes).unwrap();
        fs::write(classes.join("Obsolete.class"), b"stale").unwrap();
        remove_previous_classes(&classes).unwrap();
        assert!(!classes.exists());
        remove_previous_classes(&classes).unwrap();

        let not_a_directory = root.join("not-a-directory");
        fs::write(&not_a_directory, b"keep").unwrap();
        let error = remove_previous_classes(&not_a_directory).unwrap_err();
        assert!(error.to_string().contains(&not_a_directory.display().to_string()));
        assert!(not_a_directory.is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    fn synthetic_output(code: i32, stdout: &str, stderr: &str) -> Output {
        use std::os::unix::process::ExitStatusExt;
        Output {
            status: ExitStatus::from_raw(if code == 0 { 0 } else { code << 8 }),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[cfg(windows)]
    fn synthetic_output(code: i32, stdout: &str, stderr: &str) -> Output {
        use std::os::windows::process::ExitStatusExt;
        Output {
            status: ExitStatus::from_raw(code as u32),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn preserves_success_warnings_and_failure_output_from_both_streams() {
        let success = synthetic_output(0, "", "warning: retained\n\n");
        assert_eq!(tool_output_diagnostics("javac", &success).unwrap(), ["javac stderr: warning: retained"]);

        let failure = synthetic_output(7, "compiler detail\n", "missing symbol\n");
        let error = tool_output_diagnostics("D8", &failure).unwrap_err();
        assert!(error.contains("D8 failed with status"));
        assert!(error.contains("stdout:\ncompiler detail\n"));
        assert!(error.contains("stderr:\nmissing symbol\n"));
    }
}
