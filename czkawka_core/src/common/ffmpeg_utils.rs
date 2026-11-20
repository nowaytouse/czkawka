use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use std::{env, io};

use crate::common::process_utils::disable_windows_console_window;
use crate::tools::video_optimizer::{HardwareEncoder, VideoCodec};

/// A video executable and its optional `CZKAWKA_FFMPEG` or `CZKAWKA_FFPROBE` override.
#[derive(Clone, Copy)]
pub enum VideoTool {
    Ffmpeg,
    Ffprobe,
}

impl VideoTool {
    fn override_variable(self) -> &'static str {
        match self {
            Self::Ffmpeg => "CZKAWKA_FFMPEG",
            Self::Ffprobe => "CZKAWKA_FFPROBE",
        }
    }

    /// The filename used by FFmpeg-dependent subprocesses and libraries.
    pub const fn filename(self) -> &'static str {
        match (self, cfg!(windows)) {
            (Self::Ffmpeg, false) => "ffmpeg",
            (Self::Ffmpeg, true) => "ffmpeg.exe",
            (Self::Ffprobe, false) => "ffprobe",
            (Self::Ffprobe, true) => "ffprobe.exe",
        }
    }

    /// Find and validate the executable in the explicit override, PATH or platform fallbacks.
    /// The path of the first working executable is reused for later calls.
    pub fn resolve(self) -> io::Result<PathBuf> {
        static FFMPEG_PATH: OnceLock<PathBuf> = OnceLock::new();
        static FFPROBE_PATH: OnceLock<PathBuf> = OnceLock::new();
        let cached = match self {
            Self::Ffmpeg => &FFMPEG_PATH,
            Self::Ffprobe => &FFPROBE_PATH,
        };
        if let Some(path) = cached.get() {
            return Ok(path.clone());
        }
        let configured = env::var_os(self.override_variable());
        let search_path = env::var_os("PATH");
        let path = search_executable(self.filename(), configured.as_deref(), search_path.as_deref(), platform_fallbacks())
            .map_err(|error| io::Error::new(error.kind(), format!("{error}. Set {} to its executable path.", self.override_variable())))?;
        Ok(cached.get_or_init(|| path).clone())
    }

    pub(crate) fn command(self) -> io::Result<Command> {
        let mut command = Command::new(self.resolve()?);
        disable_windows_console_window(&mut command);
        Ok(command)
    }
}

fn platform_fallbacks() -> impl Iterator<Item = PathBuf> {
    let mut directories = Vec::new();
    if cfg!(target_os = "macos") {
        directories.extend([PathBuf::from("/opt/homebrew/bin"), PathBuf::from("/usr/local/bin"), PathBuf::from("/opt/local/bin")]);
    }
    if cfg!(target_os = "windows")
        && let Ok(executable) = env::current_exe()
        && let Some(directory) = executable.parent()
    {
        directories.push(directory.to_path_buf());
    }
    directories.into_iter()
}

fn search_executable(filename: &str, configured: Option<&OsStr>, search_path: Option<&OsStr>, fallbacks: impl IntoIterator<Item = PathBuf>) -> io::Result<PathBuf> {
    if let Some(configured) = configured {
        let executable = std::path::absolute(Path::new(configured))?;
        return if version_works(&executable) {
            Ok(executable)
        } else {
            Err(io::Error::other(format!("{} did not run {filename} -version successfully", executable.display())))
        };
    }

    search_path
        .into_iter()
        .flat_map(env::split_paths)
        .chain(fallbacks)
        .filter_map(|directory| std::path::absolute(directory.join(filename)).ok())
        .find(|executable| version_works(executable))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("No working {filename} executable was found")))
}

fn version_works(executable: &Path) -> bool {
    if !executable.is_file() {
        return false;
    }
    let mut command = Command::new(executable);
    disable_windows_console_window(&mut command);
    command
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok_and(|mut child| wait_with_timeout(&mut child))
}

pub fn check_if_ffprobe_ffmpeg_exists() -> bool {
    let ffmpeg = VideoTool::Ffmpeg.resolve();
    let ffprobe = VideoTool::Ffprobe.resolve();
    for error in [ffmpeg.as_ref().err(), ffprobe.as_ref().err()].into_iter().flatten() {
        log::warn!("{error}");
    }
    ffmpeg.is_ok() && ffprobe.is_ok()
}

/// Returns the subset of hardware encoders that actually work on this machine.
/// Each candidate is tested by attempting a 1-frame encode; encoders that fail
/// (missing driver, missing library like libcuda.so, unsupported GPU, ...) are excluded.
pub fn get_working_hardware_encoders() -> Vec<HardwareEncoder> {
    HardwareEncoder::all_non_none().iter().copied().filter(|&enc| test_hardware_encoder(enc)).collect()
}

fn test_hardware_encoder(encoder: HardwareEncoder) -> bool {
    let Some(encoder_name) = encoder.encoder_name_for_codec(VideoCodec::H264) else {
        return false;
    };
    match encoder {
        HardwareEncoder::Vaapi => test_vaapi_encoder(encoder_name),
        _ => test_encoder_simple(encoder_name),
    }
}

fn test_encoder_simple(encoder_name: &str) -> bool {
    PROBE_FRAME_SIZES.iter().any(|&size| run_probe(encoder_name, size, None))
}

fn run_probe(encoder_name: &str, size: u32, vaapi_device: Option<&str>) -> bool {
    let Ok(mut cmd) = VideoTool::Ffmpeg.command() else {
        return false;
    };
    cmd.arg("-nostdin");
    if let Some(device) = vaapi_device {
        cmd.args(["-vaapi_device", device]);
    }
    cmd.args(["-f", "lavfi", "-i", &format!("color=size={size}x{size}:rate=1")]);
    if vaapi_device.is_some() {
        cmd.args(["-vf", "format=nv12,hwupload"]);
    }
    cmd.args(["-frames:v", "1", "-c:v", encoder_name, "-f", "null", "-"]);
    let Ok(mut child) = cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn() else {
        return false;
    };
    wait_with_timeout(&mut child)
}

// Encoders have different minimum frame sizes (NVENC rejects 128x128, AMF rejects 64x64), so try big to small
const PROBE_FRAME_SIZES: [u32; 4] = [512, 256, 128, 64];

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

fn wait_with_timeout(child: &mut std::process::Child) -> bool {
    let deadline = Instant::now() + PROBE_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return false;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return false,
        }
    }
}

/// Returns the path of the first available DRI render node, or None if none exist.
pub fn find_vaapi_device() -> Option<String> {
    (128..=132u32)
        .map(|index| format!("/dev/dri/renderD{index}"))
        .find(|device| std::path::Path::new(device).exists())
}

/// VAAPI encoding requires an explicit DRI render node and a hwupload filter step.
/// Scan /dev/dri/renderD* in order and return true on the first successful test encode.
fn test_vaapi_encoder(encoder_name: &str) -> bool {
    // renderD128 is the standard first render node; try up to renderD132 for multi-GPU systems
    (128..=132u32)
        .map(|index| format!("/dev/dri/renderD{index}"))
        .filter(|device| std::path::Path::new(device).exists())
        .any(|device| PROBE_FRAME_SIZES.iter().any(|&size| run_probe(encoder_name, size, Some(&device))))
}

/// Returns a list of hardware acceleration methods supported by the installed ffmpeg,
/// e.g. `["cuda", "vaapi", "qsv", "videotoolbox"]`.
/// Returns an empty Vec if ffmpeg is not found or reports no hwaccels.
pub fn get_available_hw_accelerations() -> Vec<String> {
    let Ok(mut command) = VideoTool::Ffmpeg.command() else {
        return Vec::new();
    };
    let Ok(output) = command.arg("-hwaccels").stderr(Stdio::null()).output() else {
        return Vec::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .skip(1) // first line is the "Hardware acceleration methods:" header
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn write_executable(path: &Path, exit_code: u8) {
        fs::write(path, format!("#!/bin/sh\n[ \"$1\" = -version ] || exit 2\nexit {exit_code}\n")).expect("Write video tool test executable");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("Allow executing video tool test fixture");
    }

    #[test]
    fn explicit_path_precedes_path_and_invalid_override_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path_executable = dir.path().join("ffmpeg");
        let configured = dir.path().join("custom ffmpeg");
        write_executable(&path_executable, 0);
        write_executable(&configured, 0);
        assert_eq!(
            search_executable("ffmpeg", Some(configured.as_os_str()), Some(dir.path().as_os_str()), []).unwrap(),
            configured
        );

        write_executable(&configured, 1);
        search_executable("ffmpeg", Some(configured.as_os_str()), Some(dir.path().as_os_str()), []).unwrap_err();
    }

    #[test]
    fn path_precedes_platform_fallback_and_unusable_candidate_is_skipped() {
        let path_dir = tempfile::tempdir().unwrap();
        let fallback = tempfile::tempdir().unwrap();
        let preferred = path_dir.path().join("ffprobe");
        let alternate = fallback.path().join("ffprobe");
        write_executable(&preferred, 0);
        write_executable(&alternate, 0);
        assert_eq!(
            search_executable("ffprobe", None, Some(path_dir.path().as_os_str()), [fallback.path().to_path_buf()]).unwrap(),
            preferred
        );

        write_executable(&preferred, 1);
        assert_eq!(
            search_executable("ffprobe", None, Some(path_dir.path().as_os_str()), [fallback.path().to_path_buf()]).unwrap(),
            alternate
        );
    }
}
