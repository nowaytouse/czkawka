"""Fork-specific Fluent overrides (load JSON packs from misc/i18n_fork/)."""

from __future__ import annotations

import json
import pathlib

_I18N_FORK = pathlib.Path(__file__).resolve().parent / "i18n_fork"


def _load_json(project: str, lang: str) -> dict[str, str]:
    path = _I18N_FORK / project / f"{lang}.json"
    if not path.is_file():
        return {}
    data: dict[str, str] = json.loads(path.read_text(encoding="utf-8"))
    return data


def _langs_for(project: str) -> list[str]:
    folder = _I18N_FORK / project
    if not folder.is_dir():
        return []
    return sorted(p.stem for p in folder.glob("*.json"))


def build_project_lang_packs() -> dict[str, dict[str, dict[str, str]]]:
    projects = [p.name for p in _I18N_FORK.iterdir() if p.is_dir()]
    out: dict[str, dict[str, dict[str, str]]] = {}
    for project in projects:
        out[project] = {lang: _load_json(project, lang) for lang in _langs_for(project)}
    return out


PROJECT_LANG_PACKS = build_project_lang_packs()
