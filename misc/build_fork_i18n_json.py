#!/usr/bin/env python3
"""Write misc/i18n_fork/**/*.json from Python translation modules."""

from __future__ import annotations

import importlib.util
import json
import pathlib
import sys
import types

ROOT = pathlib.Path(__file__).resolve().parent
I18N_FORK = ROOT / "i18n_fork"
DATA = ROOT / "i18n_fork_data"

KROKIET_LANG_MODULES: list[tuple[str, str, str]] = [
    ("de", "krokiet_de", "KROKIET_DE"),
    ("fr", "krokiet_fr", "KROKIET_FR"),
    ("ja", "krokiet_ja", "KROKIET_JA"),
    ("ko", "krokiet_ko", "KROKIET_KO"),
    ("pl", "krokiet_pl", "KROKIET_PL"),
    ("ru", "krokiet_ru", "KROKIET_RU"),
    ("es-ES", "krokiet_es", "KROKIET_ES"),
]


def _load_module(name: str) -> types.ModuleType:
    path = DATA / f"{name}.py"
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise SystemExit(f"Missing data module {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def write_pack(project: str, lang: str, data: dict[str, str]) -> None:
    folder = I18N_FORK / project
    folder.mkdir(parents=True, exist_ok=True)
    path = folder / f"{lang}.json"
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"  wrote {path.relative_to(ROOT.parent)} ({len(data)} keys)")


def main() -> None:
    sys.path.insert(0, str(ROOT))

    for lang, mod_name, attr in KROKIET_LANG_MODULES:
        mod = _load_module(mod_name)
        data = getattr(mod, attr)
        write_pack("krokiet", lang, data)

    misc = _load_module("misc_packs")
    for project, lang, attr in misc.PACKS:
        write_pack(project, lang, getattr(misc, attr))

    print("Done.")


if __name__ == "__main__":
    main()
