#!/usr/bin/env python3
"""Fill missing Simplified Chinese entries from English for a project i18n folder."""

from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent / "ai_translate"))

from ftl_utils import parse_ftl_file  # noqa: E402
from translate import serialize_ftl_entries  # noqa: E402

# Fork-maintained zh-CN for keys missing from Crowdin sync (krokiet).
KROKIET_ZH_CN: dict[str, str] = {
    "compare_overlay_text": "叠加",
    "compare_split_text": "分屏",
    "context_menu_copy_file_name_text": "复制文件名",
    "context_menu_copy_full_path_text": "复制完整路径",
    "context_menu_protect_text": "保护此文件",
    "context_menu_remove_all_from_folder_recursive_text": "从文件夹移除全部（递归）",
    "context_menu_select_all_from_folder_recursive_text": "从文件夹全选（递归）",
    "context_menu_unprotect_text": "取消保护此文件",
    "rust_protected_files_storage_error": "受保护文件列表不可用：{ $error }。已阻止破坏性文件操作。",
    "rust_rename_single_protected": "无法重命名受保护的文件。",
    "optimize_crf_hint": "数值越低画质越好。0 接近无损，51 最差。建议：18–28。",
    "optimize_noise_reduction_hint": "降噪可能显著增加编码时间。",
    "optimize_noise_reduction_strength_hint": "1 = 最轻，10 = 最强降噪。",
    "option_check_method_hash": "哈希",
    "option_check_method_name": "名称",
    "option_check_method_size": "大小",
    "option_check_method_size_and_name": "大小和名称",
    "option_crop_detect_letterbox": "黑边",
    "option_crop_detect_motion": "运动",
    "option_crop_detect_none": "无",
    "option_music_method_fingerprint": "指纹",
    "option_music_method_tags": "标签",
    "option_noise_reduction_hqdn3d": "hqdn3d（快速）",
    "option_noise_reduction_none": "无",
    "option_search_mode_biggest": "最大",
    "option_search_mode_smallest": "最小",
    "option_video_crop_type_black_bars": "黑边",
    "option_video_crop_type_static_content": "静态内容",
    "option_video_optimizer_mode_crop": "裁剪",
    "option_video_optimizer_mode_transcode": "转码",
    "popup_custom_save_restore_text": "保存并恢复筛选数据",
    "rust_checking_empty_files_content": "正在检查 { $items_stats } 个文件的内容（{ $size_stats }）",
    "rust_hiding_links": "正在隐藏硬链接 { $items_stats }",
    "rust_trash_confirmation": "确定要将所选项目移到回收站吗？",
    "rust_trash_confirmation_number_groups": "已在 { $groups } 个组中选中 { $items } 项。",
    "rust_trash_confirmation_number_simple": "已选中 { $items } 项。",
    "rust_trash_confirmation_selected_all_in_group": "已在 { $groups } 个组中全选。",
    "rust_trash_confirmation_unsupported_volumes": "注意：并非所有卷都支持移到回收站。不支持的卷上此操作将始终失败。",
    "selection_all_except_biggest_resolution": "全选除最大分辨率外",
    "selection_all_except_biggest_size": "全选除最大大小外",
    "selection_all_except_highest_quality": "全选除最高画质外",
    "selection_all_except_longest_path": "全选除最长路径外",
    "selection_all_except_newest": "全选除最新外",
    "selection_all_except_oldest": "全选除最旧外",
    "selection_all_except_shortest_path": "全选除最短路径外",
    "selection_all_except_smallest_resolution": "全选除最小分辨率外",
    "selection_all_except_smallest_size": "全选除最小大小外",
    "settings_select_group_date_text": "修改日期",
    "settings_select_group_path_text": "路径",
    "settings_select_group_resolution_text": "分辨率",
    "settings_select_group_size_text": "大小",
    "settings_select_header_text": "选择弹窗选项",
    "settings_select_label_except_biggest_text": "除最大外",
    "settings_select_label_except_longest_text": "除最长外",
    "settings_select_label_except_newest_text": "除最新外",
    "settings_select_label_except_oldest_text": "除最旧外",
    "settings_select_label_except_shortest_text": "除最短外",
    "settings_select_label_except_smallest_text": "除最小外",
    "settings_select_label_longest_text": "最长",
    "settings_select_label_one_biggest_text": "保留最大一项",
    "settings_select_label_one_newest_text": "保留最新一项",
    "settings_select_label_one_oldest_text": "保留最旧一项",
    "settings_select_label_one_smallest_text": "保留最小一项",
    "settings_select_label_shortest_text": "最短",
    "sort_by_focus": "按焦点排序",
    "subsettings_broken_files_font": "字体",
    "subsettings_broken_files_markup": "标记（JSON/XML/TOML）",
    "subsettings_broken_files_video_ffmpeg": "视频（ffmpeg）",
    "subsettings_broken_files_video_ffmpeg_info": "使用 ffmpeg 深度检查视频（完整解码）。非常慢，且可能报告吹毛求疵的错误，即使文件可正常播放。",
    "subsettings_broken_files_video_ffprobe": "视频（ffprobe）",
    "subsettings_broken_files_video_ffprobe_info": "使用 ffprobe 快速检查视频（头信息校验）。",
    "subsettings_empty_files_non_printable_content": "仅含不可打印字符的文件",
    "subsettings_empty_files_non_printable_content_hint": "同时查找非空但仅含不可打印 ASCII 字符的文件：空字符、空格、制表符、回车、换行、垂直制表符、换页符。",
    "subsettings_empty_files_type": "要查找的其他文件类型",
    "subsettings_empty_files_zero_byte_content": "仅含空字节的文件",
    "subsettings_empty_files_zero_byte_content_hint": "同时查找非空但内容全为 0x00 空字节的文件。",
    "subsettings_images_ignore_same_resolution": "忽略分辨率相同的图片",
    "subsettings_images_max_size_ratio": "最大尺寸比例",
    "subsettings_images_only_same_size_hint": "启用后，结果仅显示文件大小完全相同的图片",
    "subsettings_images_size_ratio": "尺寸比例筛选",
    "subsettings_temporary_files_extensions_hint_text": "以逗号分隔的临时文件扩展名/后缀（例如 .tmp,.bak,~）。重置可恢复内置默认值",
    "subsettings_temporary_files_extensions_text": "扩展名：",
    "subsettings_video_optimizer_custom_command_hint": '命令中使用 {"{PATH}"} 表示输入文件。输出路径会自动追加。',
    "subsettings_video_optimizer_generate_template": "生成模板",
    "subsettings_video_optimizer_hardware_encoder": "硬件编码器",
    "subsettings_video_optimizer_noise_reduction": "降噪",
    "subsettings_video_optimizer_noise_reduction_strength": "降噪强度",
    "subsettings_video_optimizer_use_custom_command": "使用自定义 FFmpeg 命令",
    "subsettings_videos_audio_check_content": "按音频指纹比较",
    "subsettings_videos_audio_length_ratio": "最小长度比（较短/较长）",
    "subsettings_videos_audio_maximum_difference": "最大音频差异",
    "subsettings_videos_audio_min_duration_seconds": "最短文件时长 [秒]",
    "subsettings_videos_audio_preset": "快速预设",
    "subsettings_videos_audio_preset_clip": "较长视频中的片段",
    "subsettings_videos_audio_preset_custom": "自定义",
    "subsettings_videos_audio_preset_identical": "相同视频",
    "subsettings_videos_audio_preset_similar": "相似内容",
    "subsettings_videos_audio_similarity_percent": "相似度 [%]",
    "subsettings_videos_ignore_same_resolution": "忽略分辨率相同的视频",
    "trash": "将项目移到回收站",
    "trash_button": "回收站",
}


def merge_zh_cn(i18n_root: pathlib.Path, project: str, extra: dict[str, str]) -> int:
    en_file = i18n_root / "en" / f"{project}.ftl"
    zh_file = i18n_root / "zh-CN" / f"{project}.ftl"
    if not en_file.is_file() or not zh_file.is_file():
        raise SystemExit(f"Missing en or zh-CN ftl for {project} under {i18n_root}")

    en_entries = parse_ftl_file(en_file)
    zh_entries = parse_ftl_file(zh_file)
    added = 0

    for key in en_entries:
        if key in zh_entries:
            continue
        if key in extra:
            zh_entries[key] = extra[key]
            added += 1
        else:
            print(f"  {project}: no zh-CN translation for {key!r}, skipping", file=sys.stderr)

    ordered = {k: zh_entries[k] for k in en_entries if k in zh_entries}
    for k, v in zh_entries.items():
        if k not in ordered:
            ordered[k] = v

    zh_file.write_text(serialize_ftl_entries(ordered) + "\n", encoding="utf-8")

    # Match Fluent trailing '.' convention used by validate_translations.py
    zh_entries = parse_ftl_file(zh_file)
    fix_trailing_dots(en_entries, zh_entries)
    ordered2 = {k: zh_entries[k] for k in en_entries if k in zh_entries}
    for k, v in zh_entries.items():
        if k not in ordered2:
            ordered2[k] = v
    zh_file.write_text(serialize_ftl_entries(ordered2) + "\n", encoding="utf-8")

    return added


def fix_trailing_dots(en_entries: dict[str, str], zh_entries: dict[str, str]) -> None:
    for key in zh_entries:
        if key not in en_entries:
            continue
        base_value = en_entries[key]
        translated_value = zh_entries[key]
        if base_value.strip().endswith(".") and not translated_value.strip().endswith("."):
            s = translated_value.rstrip()
            if s.endswith("。"):
                s = s[:-1] + "."
            elif not s.endswith("."):
                s += "."
            zh_entries[key] = s
        elif not base_value.strip().endswith(".") and translated_value.strip().endswith("."):
            zh_entries[key] = translated_value.rstrip()[:-1]


def main() -> None:
    root = pathlib.Path(__file__).resolve().parent.parent
    added = merge_zh_cn(root / "krokiet/i18n", "krokiet", KROKIET_ZH_CN)
    print(f"krokiet: added {added} zh-CN keys")
    if added == 0:
        print("Nothing to add.")


if __name__ == "__main__":
    main()
