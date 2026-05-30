# ruff: noqa: E501
"""Smaller fork translation packs (cedinia, czkawka_gui, czkawka_core)."""

CEDINIA_DE = {
    "compare_cancelling": "Abbrechen…",
    "compare_computing": "Unterschied wird berechnet…",
    "compare_label": "Vergleichen",
    "compare_loading": "Bilder werden geladen…",
    "compare_mode_diff": "Diff",
    "compare_mode_normal": "Seite",
    "compare_mode_overlay": "Überlagerung",
    "compare_mode_split": "Geteilt",
    "compare_res_mismatch": "Unterschiedliche Auflösung – Diff kann ungenau sein",
    "dir_open_folder": "Ordner öffnen",
    "home_similar_videos_description": "Videos mit ähnlichem Audio finden (kein FFmpeg nötig)",
    "option_audio_preset_clip": "Clip im längeren",
    "option_audio_preset_identical": "Identisch",
    "option_audio_preset_similar": "Ähnlich",
    "settings_appearance_label": "ERSCHEINUNGSBILD",
    "settings_broken_font": "Schrift",
    "settings_broken_markup": "Markup (JSON/XML/TOML)",
    "settings_dark_theme": "Dunkles Design",
    "settings_dark_theme_desc": "Dunkles Farbschema verwenden",
    "settings_ignore_same_resolution": "Bilder mit gleicher Auflösung ignorieren",
    "settings_similar_videos_audio_preset": "Audio-Ähnlichkeitsvoreinstellung",
    "settings_similar_videos_audio_preset_desc": "Legt fest, wie streng Audio übereinstimmen muss",
    "settings_similar_videos_header": "ÄHNLICHE VIDEOS (AUDIO)",
    "settings_temporary_files_extensions_label": "ERWEITERUNGEN",
    "settings_temporary_files_extensions_placeholder": "z. B. .tmp,.bak,~",
    "settings_temporary_files_header": "TEMPORÄRE DATEIEN",
    "settings_temporary_files_reset": "Auf Standard zurücksetzen",
    "similar_videos_group_header": "{ $count } ähnliche Videos",
    "stage_all_hiding_links": "Hardlinks werden ausgeblendet",
    "stage_empty_files_checking_content": "Dateiinhalt wird geprüft",
    "tool_similar_videos": "Ähnliche Videos (Audio)",
}

GUI_ZH_CN = {
    "bottom_protect_button": "保护",
    "bottom_protect_button_tooltip": "保护所选文件，避免删除或移动",
    "bottom_unprotect_button": "取消保护",
    "bottom_unprotect_button_tooltip": "取消所选文件的保护",
    "check_button_general_only_same_size": "仅相同大小",
    "check_button_general_only_same_size_tooltip": "结果中仅显示文件大小完全相同的图片",
    "check_button_image_size_ratio": "尺寸比例筛选",
    "entry_image_size_ratio_tooltip": "组内最大文件大小比例（1.0 = 完全相同，1.05 = 5% 以内，1.5 = 50% 以内）",
    "header_krokiet_button_tooltip": "试试 Krokiet——全新改进版！",
    "krokiet_promo_link_download": "下载 Krokiet/Cedinia",
    "krokiet_promo_link_project": "项目页面",
    "krokiet_promo_title": "认识 Krokiet！",
    "krokiet_promo_message": "你好，勇敢的 Czkawka 用户！\n\n原力与你同在，但 Krokiet 更胜一筹——更快、更现代的界面。GTK 版仅作维护，新功能请用 Krokiet。",
    "popover_select_all_except_highest_quality": "全选除最高画质外",
    "progress_hiding_hard_link": "正在隐藏硬链接 {$file_checked}/{$all_files}",
    "settings_clear_protected_files_button": "清除受保护文件 ({$count})",
}

GUI_DE = {
    "bottom_protect_button": "Schützen",
    "bottom_protect_button_tooltip": "Ausgewählte Dateien vor Löschen/Verschieben schützen",
    "bottom_unprotect_button": "Schutz aufheben",
    "bottom_unprotect_button_tooltip": "Schutz von ausgewählten Dateien entfernen",
    "check_button_general_only_same_size": "Nur gleiche Größe",
    "check_button_general_only_same_size_tooltip": "Nur Bilder mit identischer Dateigröße in den Ergebnissen anzeigen",
    "check_button_image_size_ratio": "Größenverhältnis-Filter",
    "entry_image_size_ratio_tooltip": "Max. Dateigrößenverhältnis in einer Gruppe (1.0 = exakt gleich, 1.05 = innerhalb 5 %, 1.5 = innerhalb 50 %)",
    "header_krokiet_button_tooltip": "Probiere Krokiet – die neue, verbesserte Version!",
    "krokiet_promo_link_download": "Krokiet/Cedinia herunterladen",
    "krokiet_promo_link_project": "Projektseite",
    "krokiet_promo_title": "Lerne Krokiet kennen!",
    "krokiet_promo_message": "Hallo, mutiger Czkawka-Nutzer!\n\nDie Macht ist mit dir, aber Krokiet ist es auch – schneller und moderner. GTK wird nur gewartet; neue Funktionen gibt es in Krokiet.",
    "popover_select_all_except_highest_quality": "Alle außer höchster Qualität auswählen",
    "progress_hiding_hard_link": "Hardlinks werden ausgeblendet {$file_checked}/{$all_files}",
    "settings_clear_protected_files_button": "Geschützte Dateien löschen ({$count})",
}

CORE_ZH_CN = {
    "core_custom_command_empty": "自定义 FFmpeg 命令不能为空",
    "core_custom_command_missing_path_placeholder": '自定义 FFmpeg 命令必须包含 {"{PATH}"} 作为输入文件占位符',
}

CORE_DE = {
    "core_custom_command_empty": "Benutzerdefinierter FFmpeg-Befehl darf nicht leer sein",
    "core_custom_command_missing_path_placeholder": 'Der Befehl muss {"{PATH}"} als Platzhalter für die Eingabedatei enthalten',
}

# Registry: (project, lang, attribute name in this module)
PACKS: list[tuple[str, str, str]] = [
    ("cedinia", "de", "CEDINIA_DE"),
    ("czkawka_gui", "zh-CN", "GUI_ZH_CN"),
    ("czkawka_gui", "de", "GUI_DE"),
    ("czkawka_core", "zh-CN", "CORE_ZH_CN"),
    ("czkawka_core", "de", "CORE_DE"),
]
