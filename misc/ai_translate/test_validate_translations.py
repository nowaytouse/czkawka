import contextlib
import io
from pathlib import Path
import tempfile
import unittest

from ftl_utils import parse_ftl_file
from validate_translations import (
    find_literal_newlines,
    fix_literal_newlines_in_language_file,
    validate_i18n_folder,
    validate_translation,
)


class LiteralNewlineTests(unittest.TestCase):
    def test_absolute_paths_preserve_names(self) -> None:
        for value in [
            r"C:\Users\name\Desktop*",
            r"\\nas\newshare\name",
            r'"C:\Program Files\new name\notes.txt"',
            r"'\\nas\new share\notes.txt'",
            r"`C:\照片\newfiles`",
            r"C:/Users\name/Desktop*",
            "real\nnewline",
            r"unchanged\r\t",
        ]:
            with self.subTest(value=value):
                self.assertEqual(find_literal_newlines(value), [])
                self.assertEqual(validate_translation(value, value, "example"), [])

    def test_real_escapes_remain_errors(self) -> None:
        for value in [r"before\nafter", r"C:relative\name", r"\\nas", r"wordC:\name", r'"C:\Program Files\name']:
            with self.subTest(value=value):
                self.assertTrue(find_literal_newlines(value))
                self.assertTrue(
                    any("Literal \\n sequence" in error for error in validate_translation(value, value, "example"))
                )

    def test_fixer_preserves_paths_and_is_idempotent(self) -> None:
        cases = [
            (r"before\nC:\new\file", r"before C:\new\file"),
            (r"before\n\\nas\newshare\name", r"before \\nas\newshare\name"),
            (r"before\nC:/folder\name", r"before C:/folder\name"),
            (r"C:\new\file\nC:\new\file", r"C:\new\file C:\new\file"),
            (r"C:\new\file\n\\nas\newshare\name", r"C:\new\file \\nas\newshare\name"),
            (r'C:\new\file\n"C:\Program Files\name"', r'C:\new\file "C:\Program Files\name"'),
            (r"(C:\new\file)\nafter", r"(C:\new\file) after"),
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "sample.ftl"
            for original, expected in cases:
                with self.subTest(value=original):
                    self.assertEqual(len(find_literal_newlines(original)), 1)
                    path.write_text(f"example = {original}\nuntouched = keep\n", encoding="utf-8")
                    self.assertEqual(fix_literal_newlines_in_language_file(path, {"example"}), 1)
                    self.assertEqual(parse_ftl_file(path), {"example": expected, "untouched": "keep"})
                    self.assertEqual(fix_literal_newlines_in_language_file(path, {"example"}), 0)

    def test_source_guard_accepts_paths_but_rejects_newline_escapes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for language in ["en", "zh-CN"]:
                (root / language).mkdir()
                (root / language / "sample.ftl").write_text(
                    "example = " + r"C:\Users\name\Desktop*" + "\n", encoding="utf-8"
                )
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(validate_i18n_folder(root, ["zh-CN"]), 0)
                (root / "en/sample.ftl").write_text("example = " + r"broken\nEnglish" + "\n", encoding="utf-8")
                self.assertEqual(validate_i18n_folder(root, ["zh-CN"]), 1)
