# Czkawka – Codebase Guide

## Language

All code, comments, commit messages, and documentation must be written in **English**.

---

## Comments

Keep comments minimal. Code should be self-documenting through clear naming. Add a comment only
when the _why_ is not obvious from reading the code – algorithmic choices, non-obvious constraints,
workarounds for external library bugs, etc. Do not restate what the code already says.

---

## Panics and `expect()`

`expect()` (and `unwrap()` but only in tests) are acceptable and **preferred** over silently ignoring a failure when
a failed call would leave the program in an inconsistent or corrupted state. This is visible
throughout Krokiet's callback code: the Slint weak reference is upgraded with `expect()` because
if the window is already gone there is nothing meaningful to do except crash.

Rules of thumb:
- Use `expect()` for logic invariants – things that _cannot_ fail unless there is a programming
  error (e.g. "failed to upgrade MainWindow weak ref in callback that was just registered").
- Use proper `Result`/`Option` propagation for _expected_ failures (I/O errors, missing files,
  user-cancellable operations).
- Never silently swallow errors with `let _ = ...` unless the failure is genuinely irrelevant.
- Prefer `unwrap_or_default()` / `unwrap_or_else()` over `unwrap()` when a sensible fallback
  exists.

---

## Project Goals

Two properties are non-negotiable across every sub-project:

1. **Performance first** – scanning and file operations must be fast. Parallelism via `rayon`,
   efficient algorithms (e.g. perceptual hashing, blake3), and careful memory use are the norm.
   Avoid unnecessary allocations or copies in hot paths.

2. **Minimal non-Rust dependencies** – every additional C/C++ library (or any other non-Rust
   language) makes cross-compilation harder, narrows the set of supported targets, and increases
   build complexity for contributors. Prefer pure-Rust crates. If a native library is truly
   necessary (e.g. `libheif`, `libraw`), gate it behind an optional Cargo feature so the default
   build stays fully Rust.

---

## `just fix` – baseline quality gate

Running `just fix` must pass before any merge request. It runs, in order:

1. `ruff format` – Python code formatting.
2. `mypy misc --strict` – static type checking for all scripts in `misc/`.
3. `bash misc/run_checks.sh` – project-specific checks:
   - `delete_unused_krokiet_slint_imports.py` for krokiet and cedinia
   - `find_unused_fluent_translations.py` for all four projects
   - `find_unused_slint_translations.py` for krokiet and cedinia
   - `find_unused_callbacks.py` for krokiet and cedinia
   - `find_unused_settings_properties.py` for krokiet and cedinia
4. `cargo +nightly fmt` + `cargo fmt` – Rust formatting.
5. `cargo clippy --fix` – Rust linting (two passes: with and without default features).

If `just fix` produces any output on stderr or exits non-zero the code is not ready for review.

---

## Workspace Structure

```
czkawka/
├── czkawka_core/   # Scanning logic – shared library used by all frontends
├── czkawka_cli/    # Command-line interface
├── czkawka_gui/    # Legacy GTK 4 GUI (maintenance mode only)
├── krokiet/        # Primary desktop GUI – Slint-based
├── cedinia/        # Android / mobile GUI – Slint-based
└── misc/           # Scripts: AI translation, validation, benchmarks, CI helpers
```

Cargo workspace resolver v3, minimum Rust 1.94.1, edition 2024 throughout.

---

## czkawka_core

The shared scanning engine. Every frontend depends on it; it has no UI dependency.

**Key modules:**
- `common/` – `CommonToolData` (settings, stop-flag, progress sender), `DirTraversal`, cache,
  extension filtering, path helpers, progress types.
- `tools/` – One sub-module per scanning tool:
  `duplicate`, `empty_folder`, `empty_files`, `big_file`, `similar_images`, `similar_videos`,
  `same_music`, `broken_files`, `bad_extensions`, `bad_names`, `invalid_symlinks`, `temporary`,
  `exif_remover`, `video_optimizer`.
- `localizer_core.rs` – Fluent translation loader for Rust-side messages.

Each tool implements the `CommonData` trait (shared settings access) and `PrintResults` (CSV/JSON
export). The tool struct is constructed, configured, then its `find_*()` method is called in a
worker thread. Progress is reported over a `crossbeam` channel; a stop `AtomicBool` is polled to
support cancellation.

---

## krokiet

The primary desktop GUI. Built with [Slint](https://slint.dev/) (GPL-3.0). The UI is declared
in `ui/*.slint`; Rust connects callbacks and drives the Slint model.

**Build:** `slint_build` compiles `.slint` sources at build time; `slint::include_modules!()`
exposes the generated types.

**Entry point:** `src/main.rs`
- Loads settings, creates `MainWindow`, wires up all callbacks, starts the event loop.

**Callback pattern:**
```rust
let weak = app.as_weak();
app.global::<Callabler>().on_some_action(move || {
    let app = weak.upgrade().expect("MainWindow dropped while callback is still live");
    // ...
});
```
Each feature area lives in a dedicated `connect_*.rs` file (e.g. `connect_scan.rs`,
`connect_compare.rs`, `connect_delete_button.rs`).

**`SharedModels`** (`src/shared_models.rs`):
An `Arc<Mutex<SharedModels>>` holds the last scan result and parameters of scan of each tool. It is passed to every
`connect_*` function that needs to access or mutate scan state from a background thread.

**Model layer:**
- The Slint UI is driven by `ModelRc<VecModel<SingleMainListModel>>`.
- `SingleMainListModel` carries `val_str: [string]` and `val_int: [int]` vectors – a flat,
  index-based row representation.
- Column indices for each tool are defined as constants in `src/common.rs`
  (`StrDataSimilarImages`, `StrDataDuplicates`, …).

**Translation:** `flk!("key")` / `flk!("key", var = value)` macros defined in
`src/localizer_krokiet.rs`. Language files in `i18n/<lang-code>/krokiet.ftl`.

---

## cedinia

The Android (and secondary desktop) GUI. Architecture mirrors Krokiet but adapts to mobile
constraints. Compiled as `cdylib` for Android (loaded via `android-activity`).

**Entry points:**
- Android: `#[no_mangle] pub fn android_main(app: AndroidApp)` in `src/lib.rs`
- Desktop: `fn run_app()` in `src/app.rs`

**Android-specific:**
- File picker uses JNI to call into a Kotlin/Java helper embedded via `include_bytes!` (DEX).
- Storage permissions requested at runtime; `AppState.storage_permission_granted` gates scanning.
- System insets (`inset_top`, `inset_bottom`) plumbed through to Slint for edge-to-edge layout.
- `android_logger` routes Rust log output to logcat.

**Differences from Krokiet:**
- No video tools (ffmpeg not available on Android).
- Touch-optimised UI (`cedinia/ui/`); momentum-scroll views, bottom sheets, FAB.
- `flc!` macro (cedinia-specific) in `src/localizer_cedinia.rs`.

**Translation:** `flc!("key")` macro; language files in `cedinia/i18n/<lang-code>/cedinia.ftl`.

---

## czkawka_cli

Thin wrapper around `czkawka_core`. Uses `clap` (derive API) for argument parsing and `indicatif`
for progress bars. No GUI code. Results printed via the tool's `PrintResults` trait.

---

## czkawka_gui

Legacy GTK 4 GUI. **Maintenance mode only** – no new features are added. Bug-fixes that
keep it compatible with core API changes are accepted.

---

## misc/

- `ai_translate/translate.py` – AI-powered batch translation into all supported languages.
- `ai_translate/validate_translations.py` – Checks placeholder consistency across translations.
  Pass `--fix` to automatically remove invalid entries.
- `find_unused_fluent_translations.py` / `find_unused_slint_translations.py` – Dead-code
  detection for translation keys.
- `gen_cedinia_licenses.py` – Generates `THIRD_PARTY_LICENSES.txt` from Cargo metadata.

---

## i18n

All user-visible strings use [Fluent](https://projectfluent.org/) (`.ftl` files).

| Project      | Macro  | File pattern                                |
|--------------|--------|---------------------------------------------|
| krokiet      | `flk!` | `krokiet/i18n/<lang>/krokiet.ftl`           |
| cedinia      | `flc!` | `cedinia/i18n/<lang>/cedinia.ftl`           |
| czkawka_core | `flc!` | `czkawka_core/i18n/<lang>/czkawka_core.ftl` |
| czkawka_gui  | `flg!` | `czkawka_gui/i18n/<lang>/czkawka_gui.ftl`   |

English is the source/fallback language. All other locales are AI-translated and then validated.

**Important:** Only edit the English `.ftl` files (`i18n/en/`) directly in this repository.
All other language files are managed through [Crowdin](https://crowdin.com/) and will be
**overwritten** when translations are pulled from Crowdin. Any manual edits to non-English
`.ftl` files in the repo will be lost on the next `just unpack_translations` run.

**Fork (Simplified Chinese):** This branch maintains **zh-CN only** beyond Crowdin. Use
`just sync-zh-cn` (runs `misc/fill_zh_cn_missing.py` and `misc/sync_fork_i18n.py` for
`zh-CN` / GTK+core fork keys). Krokiet and Cedinia bundle **Noto Sans SC**
(`ui/fonts/NotoSansSC-Regular.otf`, `default-font-family` in `main_window.slint`). After
editing fork strings, validate with
`uv run misc/ai_translate/validate_translations.py <project>/i18n --languages zh-CN`.
Tracked zh-CN files live under `i18n/zh-CN/` (gitignored by default — `git add -f` when
committing).

---

## Slint UI conventions

- **Hidden Text elements for width measurement** – where a layout element must adapt its width to
  translated label text, add off-screen `Text` instances (`x: -10000px; y: -10000px; height: 0`)
  and compute `preferred-width` at runtime (see `LeftSidePanel`, `CompareInfoBar`).
- **Enums over strings** – UI state that takes a fixed set of values should use a Slint `enum`,
  not a `string` (e.g. `ConfirmPopupAction`, `ActiveTool`, `ScanState`).
- **Global state** – Application-wide state lives in Slint `global` blocks (`GuiState`,
  `AppState`, `Settings`, `Translations`, …). Rust reads/writes via `app.global::<GlobalName>()`.

---

## Build profiles (Cargo.toml)

| Profile        | Purpose                                                              |
|----------------|----------------------------------------------------------------------|
| `release`      | Standard release                                                     |
| `fast_release` | Incremental, stripped – fast iteration                               |
| `rdebug`       | Release + full debug symbols (profiling)                             |
| `fastest`      | Max opt, LTO, panic=abort – mostly benchmarks/poc how fast it can be |
| `fastci`       | Small binary, fast CI builds                                         |

---

## justfile quick reference

```
just run krokiet          # debug run
just runr krokiet         # fast_release run
just fix                  # format + clippy + Python checks
just translate            # AI-translate all projects
just validate_translations [--fix]
just sync-zh-cn           # fork: fill zh-CN gaps + GTK/core fork strings (zh-CN only)
just pack_translations    # create i18n_translations.zip for Crowdin
just unpack_translations <path>
just android              # build + install + launch on device
just androidr             # release variant
```

---

## Known Defects / Limitations (this fork)

- **GTK frontend maintenance-only** – `czkawka_gui` receives no new features; all active development targets Krokiet. Upstream has announced it will eventually stop providing GTK binaries.
- **Cedinia is experimental** – the Android frontend lacks video tools (ffmpeg unavailable on Android) and has no stable release.
- **Non-English translations are AI-generated** – only the English `.ftl` files are hand-edited; all other locales are machine-translated via Crowdin and may contain errors or missing entries.
- **Cache incompatibility on upgrade** – the broken-files cache format changed (file type no longer stored); existing cache files are silently regenerated on first run after upgrade, causing a slower first scan.
- **Prehash cache invalidated on upgrade** – the prehash algorithm was updated; all prehash cache entries are invalid after upgrading and will be recomputed.
- **Mac Intel binaries dropped** – upstream no longer provides prebuilt Intel macOS binaries due to CI build times; Intel Mac users must compile from source.
- **`SelectAllExceptHighestQuality` is SimilarImages-only** – the selection mode (spare the highest-quality copy: biggest pixel count, file size as tiebreaker) is only shown for the Similar Images tab; it has no meaning for other tools. Its visibility toggle is persisted in `BasicSettings.select_show_except_highest_quality`.

### Stability investigation (Krokiet)

Findings from auditing fork commits against `upstream/master`:

- **File-protection feature [REDESIGNED]** – protection is now a visible, reversible per-row state instead of a removal. `protect_selected_items`, the post-scan filter, and the per-row context-menu toggle all *mark* rows (`protected = true`) and clear their checkbox in place via `set_row_data`; rows are never removed, so `TOOLS_SELECTION` indices stay valid (this also eliminates the earlier selection-desync panic, since the model is no longer swapped). Protected rows render with an amber tint + accent bar and a disabled checkbox, and `is_file_protected` guards delete, move, hardlink, symlink and rename. The enabled-items counter is recomputed after any in-place checkbox change. Recovering a single file no longer requires "Clear All". Post-scan marking runs in `filter_protected_files_after_scan` (fork-only, wired from `scan_ended`); it must not call `set_row_data` synchronously inside `scan_ended` before `scanning = false` — use `invoke_from_event_loop` (see `file_protection/connect.rs`).
- **"Choose application" after scan (macOS) [ADDRESSED]** – only `open_item_simple` → `open::that` can show that dialog (path is logged at `info!`). Upstream has no `filter_protected_files_after_scan` in `scan_ended`. If the dialog persists with zero protected files in Settings, capture the logged path from the terminal.
- **Simplified Chinese UI [FIXED in fork]** – Krokiet/Cedinia embed Noto Sans SC; zh-CN strings are maintained via `just sync-zh-cn`. System locale tags such as `zh`, `zh-CN`, and `zh-Hans-CN` map to `zh-CN` in `find_the_closest_language_idx_to_system()` (BCP47-aware, not the old alphabetic-stripping matcher).
- **Image preview [PARTIALLY MITIGATED]** – preview decode/crop runs on a background thread with a generation counter so the UI thread is not blocked and stale loads are dropped. A GPU/backend crash on first preview (if any) may still be upstream/`winit_femtovg`; report upstream if it persists after this change.
- **Double-click spawns a second instance / second-instance crash [NOT A FORK REGRESSION]** – the double-click handler routes to `open::that(path)` in `connect_row_selection.rs`, which is byte-identical to upstream. No fork code re-execs the krokiet binary. The "second instance" is an OS file-association side effect of opening a result file, not a fork bug.
- **Environment limitation** – GPU/renderer crashes require a live GUI session to reproduce and capture a backtrace; they cannot be reproduced in a headless CI/agent environment. Root-causing here is static (diff vs upstream) only.
