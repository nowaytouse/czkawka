import subprocess
import tempfile
import unittest
from pathlib import Path


class BenchMatrixPrivacyTest(unittest.TestCase):
    def test_dry_run_preserves_existing_output_and_omits_input_path(self) -> None:
        script = Path(__file__).resolve().parent / "bench_disk_usage_matrix.sh"
        with tempfile.TemporaryDirectory(prefix="czkawka-bench-test-") as temp:
            output = Path(temp) / "output"
            output.mkdir()
            existing = output / "debug.txt"
            existing.write_text("keep existing output")
            source = Path(temp) / "sensitive-input-name"
            result = subprocess.run(
                ["bash", str(script), "--dry-run", "--configs", "debug", "--out", str(output), "--path", str(source)],
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(existing.read_text(), "keep existing output")
            self.assertNotIn(str(source), result.stdout + result.stderr)
            self.assertEqual(len(list(output.glob("run.*"))), 1)


if __name__ == "__main__":
    unittest.main()
