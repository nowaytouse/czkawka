use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::{env, fs, io};

use czkawka_core::common::ffmpeg_utils::VideoTool;

const READY_ENV: &str = "KROKIET_VIDEO_TOOL_PATH_READY";
const MACOS_TOOL_DIRECTORIES: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin", "/opt/local/bin"];

pub(crate) fn relaunch_if_needed() -> io::Result<()> {
    if env::var_os(READY_ENV).is_some() {
        return Ok(());
    }
    let (Ok(ffmpeg), Ok(ffprobe)) = (VideoTool::Ffmpeg.resolve(), VideoTool::Ffprobe.resolve()) else {
        return Ok(());
    };
    let tools = [(VideoTool::Ffmpeg, ffmpeg.as_path()), (VideoTool::Ffprobe, ffprobe.as_path())];
    let inherited: Vec<_> = env::var_os("PATH").iter().flat_map(env::split_paths).collect();
    if finds_selected_tools(&inherited, &tools) {
        return Ok(());
    }

    // Similario resolves the bare names itself. Give only this app the matching PATH.
    let mut command = Command::new(env::current_exe()?);
    command.args(env::args_os().skip(1)).env(READY_ENV, "1");
    if cfg!(target_os = "macos")
        && tools
            .iter()
            .all(|(_, tool)| MACOS_TOOL_DIRECTORIES.iter().any(|directory| tool.parent() == Some(Path::new(directory))))
        && let Some(path) = path_with_tools(&inherited, &tools).and_then(|path| env::join_paths(path).ok())
    {
        return Err(command.env("PATH", path).exec());
    }

    let aliases = tempfile::Builder::new().prefix("krokiet-video-tools-").tempdir()?;
    for (tool, executable) in tools {
        symlink(executable, aliases.path().join(tool.filename()))?;
    }
    let path = join_paths(std::iter::once(aliases.path().to_path_buf()).chain(inherited))?;
    let status = command.env("PATH", path).status()?;
    if let Err(error) = aliases.close() {
        log::warn!("Could not remove the temporary video tool paths: {error}");
    }
    process::exit(status.code().unwrap_or(1));
}

fn join_paths(paths: impl IntoIterator<Item = PathBuf>) -> io::Result<std::ffi::OsString> {
    env::join_paths(paths).map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
}

fn path_with_tools(inherited: &[PathBuf], tools: &[(VideoTool, &Path); 2]) -> Option<Vec<PathBuf>> {
    let mut path = inherited.to_vec();
    for (_, tool) in tools {
        let directory = tool.parent()?;
        if !path.iter().any(|existing| existing == directory) {
            path.push(directory.to_path_buf());
        }
    }
    finds_selected_tools(&path, tools).then_some(path)
}

fn finds_selected_tools(path: &[PathBuf], tools: &[(VideoTool, &Path); 2]) -> bool {
    tools.iter().all(|(tool, selected)| {
        let first_on_path = path
            .iter()
            .map(|directory| directory.join(tool.filename()))
            .find(|candidate| candidate.metadata().is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0));
        let actual = first_on_path.and_then(|candidate| fs::canonicalize(candidate).ok());
        fs::canonicalize(selected).is_ok_and(|expected| Some(expected) == actual)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_installation_can_be_appended_without_changing_path_precedence() {
        let selected_dir = tempfile::tempdir().unwrap();
        let inherited_dir = tempfile::tempdir().unwrap();
        let inherited = vec![inherited_dir.path().to_path_buf()];
        let ffmpeg = selected_dir.path().join("ffmpeg");
        let ffprobe = selected_dir.path().join("ffprobe");

        for path in [ffmpeg.clone(), ffprobe.clone(), inherited_dir.path().join("ffmpeg")] {
            fs::write(&path, "").unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let tools = [(VideoTool::Ffmpeg, ffmpeg.as_path()), (VideoTool::Ffprobe, ffprobe.as_path())];
        assert!(!finds_selected_tools(&inherited, &tools));
        assert!(path_with_tools(&inherited, &tools).is_none());
        assert_eq!(path_with_tools(&[], &tools).unwrap(), [selected_dir.path().to_path_buf()]);
    }
}
