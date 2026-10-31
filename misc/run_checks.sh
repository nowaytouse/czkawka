#!/usr/bin/env bash

cmds=(
    "uv run --project misc python misc/delete_unused_krokiet_slint_imports.py krokiet"
    "uv run --project misc python misc/delete_unused_krokiet_slint_imports.py cedinia"
    "uv run --project misc python misc/find_unused_fluent_translations.py krokiet"
    "uv run --project misc python misc/find_unused_fluent_translations.py cedinia"
    "uv run --project misc python misc/find_unused_fluent_translations.py czkawka_core"
    "uv run --project misc python misc/find_unused_slint_translations.py krokiet"
    "uv run --project misc python misc/find_unused_slint_translations.py cedinia"
    "uv run --project misc python misc/find_unused_callbacks.py krokiet"
    "uv run --project misc python misc/find_unused_callbacks.py cedinia cedinia/ui/globals/app_state.slint"
    "uv run --project misc python misc/find_unused_settings_properties.py krokiet"
    "uv run --project misc python misc/find_unused_settings_properties.py cedinia"
)

failed=""
for cmd in "${cmds[@]}"; do
    out=$(eval "$cmd" 2>&1)
    if [ $? -ne 0 ]; then
        failed+="=== FAILED: $cmd ===\n$out\n\n"
    fi
done

if [ -n "$failed" ]; then
    echo -e "$failed"
    exit 1
fi
