#!/usr/bin/env python3
"""Sync fork-specific Fluent keys into locale files (Crowdin gaps)."""

from __future__ import annotations

import argparse
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent / "ai_translate"))
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from ftl_utils import parse_ftl_file  # noqa: E402
from fill_zh_cn_missing import (  # noqa: E402
    CEDINIA_ZH_CN,
    KROKIET_ZH_CN,
    fix_trailing_dots,
)
from fork_i18n_overrides import PROJECT_LANG_PACKS  # noqa: E402
from translate import serialize_ftl_entries  # noqa: E402

try:
    import opencc  # type: ignore[import-untyped]

    _S2T = opencc.OpenCC("s2t")
except ImportError:
    _S2T = None

PRIORITY_LANGS = [
    "zh-CN",
    "zh-TW",
    "ja",
    "ko",
    "de",
    "fr",
    "es-ES",
    "pl",
    "ru",
    "it",
    "pt-BR",
    "nl",
]

BASE_ZH_CN: dict[str, dict[str, str]] = {
    "krokiet": KROKIET_ZH_CN,
    "cedinia": CEDINIA_ZH_CN,
}


def zh_tw_from_zh_cn(zh_cn: dict[str, str]) -> dict[str, str]:
    if _S2T is None:
        raise SystemExit("opencc-python-reimplemented is required for zh-TW (uv sync in repo root)")
    return {k: _S2T.convert(v) for k, v in zh_cn.items()}


def pack_for(project: str, lang: str) -> dict[str, str]:
    if lang == "zh-CN" and project in BASE_ZH_CN:
        return BASE_ZH_CN[project]
    if lang == "zh-TW" and project in BASE_ZH_CN:
        return zh_tw_from_zh_cn(BASE_ZH_CN[project])
    project_packs = PROJECT_LANG_PACKS.get(project, {})
    return dict(project_packs.get(lang, {}))


def merge_locale(
    i18n_root: pathlib.Path,
    project: str,
    lang: str,
    pack: dict[str, str],
    *,
    copy_en: bool,
) -> int:
    en_file = i18n_root / "en" / f"{project}.ftl"
    loc_file = i18n_root / lang / f"{project}.ftl"
    if not en_file.is_file():
        raise SystemExit(f"Missing {en_file}")
    if not loc_file.is_file():
        raise SystemExit(f"Missing {loc_file}")

    en_entries = parse_ftl_file(en_file)
    loc_entries = parse_ftl_file(loc_file)
    added = 0

    for key, en_value in en_entries.items():
        if key in loc_entries:
            continue
        if key in pack:
            loc_entries[key] = pack[key]
            added += 1
        elif copy_en:
            loc_entries[key] = en_value
            added += 1

    ordered = {k: loc_entries[k] for k in en_entries if k in loc_entries}
    for k, v in loc_entries.items():
        if k not in ordered:
            ordered[k] = v

    loc_file.write_text(serialize_ftl_entries(ordered) + "\n", encoding="utf-8")

    loc_entries = parse_ftl_file(loc_file)
    fix_trailing_dots(en_entries, loc_entries)
    ordered2 = {k: loc_entries[k] for k in en_entries if k in loc_entries}
    for k, v in loc_entries.items():
        if k not in ordered2:
            ordered2[k] = v
    loc_file.write_text(serialize_ftl_entries(ordered2) + "\n", encoding="utf-8")
    return added


def fix_ref_trailing_dots(i18n_root: pathlib.Path, project: str) -> int:
    en_file = i18n_root / "en" / f"{project}.ftl"
    en_entries = parse_ftl_file(en_file)
    if "ref" not in en_entries or en_entries["ref"].strip().endswith("."):
        return 0
    fixed = 0
    for lang_dir in i18n_root.iterdir():
        if not lang_dir.is_dir() or lang_dir.name == "en":
            continue
        loc_file = lang_dir / f"{project}.ftl"
        if not loc_file.is_file():
            continue
        entries = parse_ftl_file(loc_file)
        if "ref" not in entries:
            continue
        if entries["ref"].strip().endswith("."):
            entries["ref"] = entries["ref"].strip()[:-1]
            ordered = {k: entries[k] for k in en_entries if k in entries}
            for k, v in entries.items():
                if k not in ordered:
                    ordered[k] = v
            loc_file.write_text(serialize_ftl_entries(ordered) + "\n", encoding="utf-8")
            fixed += 1
    return fixed


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "projects",
        nargs="*",
        choices=["krokiet", "cedinia", "czkawka_gui", "czkawka_core", "all"],
        default=["all"],
    )
    parser.add_argument(
        "--langs",
        default=",".join(PRIORITY_LANGS),
        help="Comma-separated locale codes to update",
    )
    parser.add_argument(
        "--copy-en",
        action="store_true",
        help="For missing keys without a pack entry, copy English text",
    )
    parser.add_argument(
        "--copy-en-all-langs",
        action="store_true",
        help="Run --copy-en for every locale directory (not only --langs)",
    )
    parser.add_argument("--fix-ref", action="store_true", help="Strip stray trailing '.' on ref key")
    args = parser.parse_args()

    root = pathlib.Path(__file__).resolve().parent.parent
    projects = (
        ["krokiet", "cedinia", "czkawka_gui", "czkawka_core"]
        if not args.projects or args.projects == ["all"]
        else args.projects
    )
    langs = [x.strip() for x in args.langs.split(",") if x.strip()]

    if args.copy_en_all_langs:
        langs = sorted(
            {p.name for proj in projects for p in (root / proj / "i18n").iterdir() if p.is_dir() and p.name != "en"}
        )

    for project in projects:
        i18n_root = root / project / "i18n"
        if args.fix_ref:
            n = fix_ref_trailing_dots(i18n_root, project)
            if n:
                print(f"{project}: fixed ref trailing dot in {n} locale(s)")
        for lang in langs:
            pack = pack_for(project, lang)
            if not pack and not args.copy_en:
                continue
            added = merge_locale(i18n_root, project, lang, pack, copy_en=args.copy_en)
            if added:
                print(f"{project}/{lang}: added {added} keys")


if __name__ == "__main__":
    main()
