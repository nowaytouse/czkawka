import os
import subprocess
import tempfile
import unittest
from pathlib import Path


class ScreensIsolationTest(unittest.TestCase):
    def test_failed_display_never_drives_an_existing_display(self) -> None:
        script = Path(__file__).resolve().parent / "screens.sh"
        with tempfile.TemporaryDirectory(prefix="czkawka-screens-test-") as temp:
            root = Path(temp)
            tools = root / "bin"
            tools.mkdir()
            marker = root / "xdotool-called"
            for name in ("Xvfb", "xdotool", "import", "uv", "convert"):
                executable = tools / name
                command = f"touch '{marker}'" if name == "xdotool" else "exit 1" if name == "Xvfb" else "exit 0"
                executable.write_text(f"#!/bin/sh\n{command}\n")
                executable.chmod(0o700)
            binary = root / "krokiet"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(0o700)
            env = os.environ.copy()
            env.update(
                {"PATH": f"{tools}:{env.get('PATH', '')}", "TMPDIR": temp, "BIN": str(binary), "SCREENS_DISPLAY": "95"}
            )
            result = subprocess.run(
                ["bash", str(script)], env=env, capture_output=True, text=True, timeout=30, check=False
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Xvfb failed to start", result.stderr)
            self.assertFalse(marker.exists())

    def test_app_uses_private_paths_despite_inherited_overrides(self) -> None:
        script = Path(__file__).resolve().parent / "screens.sh"
        with tempfile.TemporaryDirectory(prefix="czkawka-screens-test-") as temp:
            root = Path(temp)
            tools = root / "bin"
            tools.mkdir()
            commands = {
                "Xvfb": "exec /bin/sleep 30",
                "xdotool": 'if [ "$1" = search ]; then echo 1; fi',
                "import": 'printf "%s\\n" "$*" >> "$SCREEN_TEST_SHOTS"',
                "uv": "exit 0",
                "convert": "exit 0",
                "sleep": "exec /bin/sleep 0.02",
            }
            for name, command in commands.items():
                executable = tools / name
                executable.write_text(f"#!/bin/sh\n{command}\n")
                executable.chmod(0o700)
            marker = root / "app-env"
            shots = root / "shots"
            binary = root / "krokiet"
            binary.write_text(
                '#!/bin/sh\nprintf "%s\\n" "$HOME" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME" '
                '"$CZKAWKA_CONFIG_PATH" "$CZKAWKA_CACHE_PATH" "${KROKIET_AUDIO_STOP_FILE-unset}" > "$SCREEN_TEST_MARKER"\n'
                "exec /bin/sleep 30\n"
            )
            binary.chmod(0o700)
            env = os.environ.copy()
            env.update(
                {
                    "PATH": f"{tools}:{env.get('PATH', '')}",
                    "TMPDIR": temp,
                    "BIN": str(binary),
                    "SCREEN_TEST_MARKER": str(marker),
                    "SCREEN_TEST_SHOTS": str(shots),
                    "CZKAWKA_CONFIG_PATH": "/must-not-use",
                    "CZKAWKA_CACHE_PATH": "/must-not-use",
                    "XDG_CONFIG_HOME": "/must-not-use",
                    "XDG_CACHE_HOME": "/must-not-use",
                    "KROKIET_AUDIO_STOP_FILE": "/must-not-use",
                }
            )
            result = subprocess.run(
                ["bash", str(script)], env=env, capture_output=True, text=True, timeout=30, check=False
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            home, xdg_config, xdg_cache, config, cache, audio = marker.read_text().splitlines()
            self.assertTrue(Path(home).is_relative_to(root))
            self.assertEqual(xdg_config, f"{home}/.config")
            self.assertEqual(xdg_cache, f"{home}/.cache")
            self.assertEqual(config, f"{home}/.config/krokiet")
            self.assertEqual(cache, f"{home}/.cache/czkawka")
            self.assertEqual(audio, "unset")
            self.assertIn("02_exact_filenames.png", shots.read_text())
            self.assertIn("16_disk_usage.png", shots.read_text())


if __name__ == "__main__":
    unittest.main()
