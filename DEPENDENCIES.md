# Rolling dependency maintenance

The fork's sole branch, `master`, tracks the latest **stable** dependencies. Major upgrades are in scope:
adapt the implementation and tests instead of indefinitely freezing a dependency. Version
numbers alone are not proof of an upgrade: the manifest, resolved lockfile and enabled feature
paths must agree. Prereleases require a separate explicit request.

## Update procedure

1. Inspect the index, HEAD, worktree, remotes and any hidden index flags. Preserve existing
   edits and stashes; never clear them to create a clean baseline.
2. Install `cargo-edit` if missing (`cargo install cargo-edit --locked`). Run `just upgrade-check`
   for a non-mutating preview, then `just upgrade` to update workspace and standalone crates.
   The preview includes major releases, transitive updates, Python tools and current registry
   releases for excluded contract exceptions. Discovery and lock refresh ignore the old Rust
   minimum so an outdated MSRV cannot silently hide a required migration; review the resulting MSRV.
   Python tool requirements use rolling lower bounds. The updater refreshes the `misc` project
   lock, copies `misc/pyproject.toml` to the generated root quality project and refreshes its
   lock as well, with prereleases disallowed in both;
   ordinary `just fix` can then sync that reviewed lock without installing updates on every run.
3. Check release notes and primary package metadata for breaking changes, MSRV changes,
   security notices and retired packages. If a usable latest release needs newer stable Rust,
   raise the workspace's minimum consistently and verify with that toolchain. Review exact
   pins explicitly; they are compatibility contracts, not permanent freezes.
4. Check Python tools in `misc/pyproject.toml` (including exact pins), Android Gradle plugin
   and its Gradle/JDK requirements, GitHub Actions releases and
   container bases. Pin Actions to full release commit SHAs and images to verified official
   digests. Use the latest stable base, not a development image with a larger version number.
   `misc/add_icon_exe/Cargo.toml` is a manifest template without a source target; check its
   `editpe` version manually instead of passing it to Cargo's workspace updater.
5. Adapt changed APIs without dropping scanners, renderers, optional image decoders, file
   protection or other fork features. Update both Slint runtimes and build dependencies
   together through `[workspace.dependencies]`; keep direct fontique compatible with Slint's
   resolved fontique. Slint permits patch updates within the reviewed minor series; its newer
   minor/major releases remain discoverable with `--incompatible allow`.
6. Run `just fix`, `just clip` and `cargo test --locked --workspace`. Check changed standalone
   crates with their own manifests. Validate native decoder features where the platform
   supports them; desktop checks do not prove Android, Linux, Windows or live GUI behavior.
7. Review the complete diff and `git diff --check`. Record updates and exceptions in the
   fork changelog. When publication is authorized, explicitly stage intended files, commit with an `[AI]` title, push
   only `origin/master`, pull with `--ff-only`, and verify clean tracked/untracked porcelain
   plus zero ahead/behind against the fetched fork branch. Retired branches are not maintenance
   targets. Preserve ignored app bundles,
   backups and existing stashes.

These are repository maintenance rules and local commands, not an activated hosted updater.
No CI polling, heartbeat or background monitor is required. Do not modify the upstream
repository, remote default branch, installed apps or release binaries as part of this procedure.

## Reviewed exceptions

| Dependency | Current contract | Reason and exit condition |
|------------|------------------|---------------------------|
| `bincode` | Exact `2.0.1`, serde with legacy variable-integer encoding | Published `3.0.0` is a retirement notice whose source deliberately fails compilation and has no serde feature. Re-evaluate a maintained replacement on each update pass; migrate only with byte-level compatibility fixtures for existing 1.3+ caches, or explicitly version new cache files and document invalidation. |
| Slint family | Shared `~1.18.1` runtime/build requirements; Fontique `0.11.1` series | Permit patch updates without independently changing the font API. Slint 1.18.1 still needs the 0.11 dlopen feature configuration, not Fontique 0.12. The preview reports the independent latest release; the mutating updater excludes it until a coordinated Slint migration. Preserve renderer support and the bundled Chinese font. |
| `little_exif` | Exact upstream revision [`d3dd078e`](https://github.com/TechnikTobi/little_exif/commit/d3dd078ec333b119d7daa74a0805ca97fe38a09d) | Published `0.6.23` still resolves the vulnerable `quick-xml 0.37.5`. The upstream fix uses `0.41.0`. `just upgrade-check` reports new registry releases. Return to the registry and remove the updater exclusion when a release includes the fix and the locked audit and synthetic EXIF tests pass. |
| Android previews | AGP 9.6.0-alpha02 and KGP 2.5.0-Beta2 | Explicitly requested on 2026-10-14. Keep built-in Kotlin enabled and use Android's documented higher-KGP classpath mechanism. Recheck published coordinates and the Gradle warning gate on each Android dependency pass; do not extend this opt-in to other ecosystems or alter app SDK levels without reviewing runtime behavior. |
| Linux release ABI | Explicit Ubuntu 24.04 artifact builders | Updating a release builder can raise the required system-library baseline. Move it only after reviewing supported distributions; normal moving-alias jobs now explicitly use the current hosted Ubuntu 26.04 GA image. |
| macOS Skia | Platform-compatible feature set | Upstream Skia has separate Metal and Vulkan macOS prebuilt artifacts. Validate supported combinations separately; do not combine unsupported backends or remove them from other platforms. |

The bincode retirement notice is documented in the [published source](https://docs.rs/crate/bincode/3.0.0/source/src/lib.rs).
The [Android Gradle plugin compatibility table](https://developer.android.com/build/releases/about-agp)
defines the corresponding Gradle requirements. Recheck both when updating; this document does
not assert that an old exception remains necessary indefinitely.

## Last reviewed: 2026-11-06

- Merged upstream follow-up commits `dae637a1`, `bf4ceda4` and `74f4ca94` after the dependency
  refresh below. They do not change Cargo/Python requirements or Action pins. Preserved the
  Python 3.15+ baseline, latest reviewed requirements, four Python regressions and all Exact
  File Names selection/removal/export safeguards. The new shared-string row pipeline retains
  protected-row state, reference headers, independent tool models and 8192 hash parameters.
- Post-merge validation passed: `just fix`, `just clip`, four Python regressions, strict typing
  for all 18 Python sources, zh-CN validation and locked workspace tests (480 passed, zero
  failed, six existing ignored cases). New row conversion/8192 checks and disk-usage arc and
  stale-render cases passed. Fedora update handling passed seven isolated command-stub cases;
  no package-manager or privilege changes were performed. Live GUI/GPU encoder/device
  acceptance and hosted CI remain unrun; the merge uses `[skip ci]`.
- Rechecked all 14 direct Python requirements against primary PyPI metadata for current stable,
  non-yanked releases and Python 3.15 compatibility. Updated Ruff to 0.17.4, ty to 0.0.98 and
  pandas to 3.1.1. The other 11 direct requirements remain current. The refreshed all-extras
  environment contains 44 installed packages, including updated pydantic and python-dateutil.
- Retained `requires-python = ">=3.15"`, Ruff `py315`, mypy `3.15` and stable-only uv resolution.
  The screenshot helper's embedded Python now selects this project too; its isolated home
  preparation also avoids an unbound theme variable under Bash `set -u`.
- Updated ZIP to 9.4.1, lopdf to 0.47.0, sevenz-rust2 to 0.24.2, toml to 1.2,
  log to 0.4.35, trash to 5.3.0 and ashpd to 0.14.0, together with resolved transitive packages.
  All five standalone Cargo projects were reviewed, including the upstream X11 e2e helper;
  the rolling preview/updater now includes its manifest and refreshes transitive lockfile
  packages for every standalone project. Its graphical tests require Linux/X11.
- Rechecked 93 distinct direct registry dependency names against the primary Cargo sparse index.
  Published Slint/slint-build remain 1.18.1; slint-build still requires Fontique 0.11.1, so the
  independent 0.12 requirement is not used. bincode 3.0.0 still deliberately fails compilation;
  retain the reviewed 2.0.1 cache-byte contract. Declared Rust minima are metadata evidence,
  separate from compilation on the minimum toolchain.
- Updated the verified full Action pins to [setup-java 6.2.0](https://github.com/actions/setup-java/releases/tag/v6.2.0)
  and [action-gh-release 3.0.4](https://github.com/softprops/action-gh-release/releases/tag/v3.0.4).
  The other four active Actions, existing Android previews, NDK r30 and Ubuntu 26.10 digest
  remain current. No Gradle wrapper is tracked; the inspected installed Gradle 9.8.2 is current.
- Re-fetched and merged upstream through `74f4ca94`; the former three-commit lag beyond
  `cc5cc718` is resolved by the integration follow-up.
- Validation on CPython 3.15.2 passed for the 17 maintenance scripts with warnings as errors,
  strict mypy for all 18 Python source files in both base-only and all-extras environments, optional image/plot/API operations,
  actual temporary chart exports and the screenshot helper's dark/light configuration step.
  An intentionally invalid Pillow assignment still fails with installed library types.
  Both refreshed Python locks resolve the same 44 packages plus the virtual project record.
- Fixed literal-newline detection and automatic repair mistaking absolute Windows path examples
  for escaped line breaks. English-source checks, translation checks and repair share the same
  path-aware matcher. Four persistent standard-library unittests cover path preservation,
  adjacent/chained paths, genuine escapes, repair idempotence and source rejection; `just fix`
  runs them without adding a test dependency.
- `just fix` passed without Rust warnings. The updated ashpd portal path passed a locked
  `x86_64-unknown-linux-gnu` core check with `xdg_portal_trash`. Full all-feature metadata
  contains 972 packages with no declared Rust minimum above 1.94.1; this does not establish
  compilation with Rust 1.94.1. Workflow lint passed after supplying the installed linter's
  missing [official Ubuntu 26.04 runner label](https://github.com/actions/runner-images#available-images)
  through a temporary external label-table configuration, with no diagnostics disabled.
- Final `just fix` (including the four Python regressions), `just clip` and locked workspace
  tests passed on stable Rust 1.99.1: 473 passed, zero failed, six existing ignored cases.
  All 18 Python source files compile under CPython 3.15.2 with warnings as errors. All five
  standalone projects passed locked all-target checks; the X11 e2e helper was checked for
  `x86_64-unknown-linux-gnu` without execution. Krokiet zh-CN translation and character checks
  passed. Linux retains the real non-UTF-8 filesystem fixture; the new path-only test also
  checks invalid parent components on macOS without requiring APFS to create invalid names.
- Linux core test-source compilation also passed with installed Zig 0.17.1 as the explicit
  GNU/Linux x86-64 C compiler/archiver, after the default attempt reported missing cross-GCC.
  The generated C object was verified as ELF64 x86-64. Only task-scoped compiler wrappers were
  used; no global tool configuration or installation changed. This is compilation evidence,
  not Linux test execution. Hosted CI, live GUI, APK/device, actual screenshots, container builds
  and real Flatpak generation were not run; later Python minor releases remain untested.

## Previous review: 2026-10-31

- Python 3.15+ follow-up: declared `>=3.15`, aligned Ruff/mypy and routed maintenance callers
  through the misc project. A later upstream fetch found one new commit, `74c24a13`, after the
  dependency review below. Its Slint, cache and application migration is not merged by this
  Python-only update; the last merged upstream base remains `eb8b91db`.
- Verified the latest stable Python releases against primary PyPI publication metadata. Raised
  base requirements to mypy 2.4.1, ty 0.0.95, pandas-stubs 3.1.0.261029, Ollama 0.6.3 and
  fonttools 4.68.0. Ruff 0.17.3, fluent.syntax 0.19.0 and OpenCC 0.1.7 remain current.
  Requirements stay as rolling lower bounds; optional runtime libraries are declared below.
- Refreshed compatible Rust lockfile packages, including encoding_rs 0.8.44 and the current
  wasm-bindgen family. All 940 registry package records declare a Rust minimum no higher than
  1.94.1; this metadata check is not a minimum-toolchain build. Slint, Fontique and bincode
  retain their reviewed compatibility contracts.
- Verified the refreshed official Ubuntu 26.10 multi-platform index digest by matching both
  the registry header and the SHA-256 of the returned body:
  `sha256:262df65c0ec6bf8d9dcc87454f4330a05fae4b97825f45854af3b136666584c2`.
  Android previews and all six active Action release SHAs remain current.
- Installed and checked all 45 resolved Python packages, including every optional extra, on
  Python 3.15.2 / macOS ARM64. Download transport stalled, so official wheel files were retrieved
  separately and checked against every lockfile SHA-256 and size before hash-enforced offline
  installation. The frozen offline environment check passed; system Python packages and global
  network settings were not changed.
- Strict mypy passed all 17 scripts. Temporary-input checks verified the pandas string fix,
  four real PNG chart exports and Markdown, font fallback without missing-font diagnostics,
  PNG/AVIF/HEIF roundtrips, CairoSVG rendering, offline Ollama responses, Fluent parsing,
  OpenCC conversion and reading the bundled Chinese font. Flatpak failure-status and temporary
  cleanup regressions passed without downloading or running the real generator.
- Locked workspace tests passed on stable Rust 1.99.1: 383 passed, zero failed, six existing
  ignored cases. `just fix` and `just clip` passed without Rust warnings, including the
  platform-compatible native decoder/rendering features and the software-only configuration.
  All-target locked compilation checks passed for ci_tester and all three standalone benchmark
  tools. Re-fetched upstream master at eb8b91db with zero upstream lag. Hosted CI,
  live GUI, APK/device, container builds and actual Flatpak generation were not run.

## Python script dependencies

There are 18 Python source files: 17 maintenance scripts (12 in `misc/`, four translation
helpers and one benchmark plotter) plus one standard-library validator regression test module.
Four maintenance scripts import third-party libraries; the remaining 14 sources use only the
standard library or local modules. The app itself remains Rust-based.
In addition, `misc/screens.sh` embeds one standard-library Python JSON configuration step;
it also runs through the maintained Python 3.15+ uv project.

`misc/pyproject.toml` is the maintained source for the generated root quality-tool project.
The fork's maintenance baseline is **Python 3.15+** (`requires-python = ">=3.15"`), with no
upper bound or exact interpreter patch pin. Ruff and mypy target the 3.15 baseline, while uv
selects a compatible interpreter and excludes prerelease dependencies by default. This admits
newer stable Python releases; each new minor release still needs dependency and runtime checks
before claiming it has been tested. Current local validation uses CPython 3.15.2 on macOS ARM64.
Interpreter requests follow [uv's Python version selection](https://docs.astral.sh/uv/concepts/python-versions/);
the formatter baseline uses [Ruff's target-version setting](https://docs.astral.sh/ruff/settings/#target-version).

From a fresh clone, install a compatible interpreter if needed and use the maintained project:

```bash
uv python install '>=3.15'
uv sync --project misc
uv run --project misc python --version
uv run --project misc python misc/ai_translate/validate_translations.py krokiet/i18n --languages zh-CN
```

The Python-backed version, license, translation and dead-code recipes explicitly use this
project, including when the ignored root quality files have not yet been generated by `just fix`.
Direct `python3 <script>` or executable shebang calls use the caller's interpreter; use Python
3.15+ or the uv commands above. The generated root project inherits the same baseline and tool
settings. Python is required for maintenance scripts, not for running the Rust applications.

Its eight base dependencies include quality tools, pandas typing stubs and retained manual
Fluent/OpenCC/font maintenance tools. `pandas-stubs` does not install runtime pandas.
Three optional extras declare six additional runtime libraries without making image encoders
or plotting libraries mandatory for routine source checks:

| Extra | Script | Reviewed stable dependencies (2026-11-06) |
|-------|--------|------------------------------------------|
| `plots` | `misc/test_compilation_speed_size/generate_md_and_plots.py` | pandas 3.1.1, matplotlib 3.11.2, tabulate 0.10.0 |
| `icons` | `misc/gen_android_icons.py` | CairoSVG 2.9.1; system Cairo is required, or use the existing Inkscape fallback |
| `test-assets` | `misc/gen_test_assets.py` | Pillow 13.0.0, pillow-heif 1.8.1; the AV1 probe also uses system FFmpeg |

For example, from the repository root:

```bash
uv run --project misc --extra plots python misc/test_compilation_speed_size/generate_md_and_plots.py
uv run --project misc --extra icons python misc/gen_android_icons.py
uv run --project misc --extra test-assets python misc/gen_test_assets.py
```

These commands generate output; run them only when intentionally updating charts, icons or
probe assets. Dependency validation uses synthetic inputs and temporary output directories.
The translator uses Ollama Python 0.6.3 and requires a separately running local Ollama service;
upgrading its Python client does not authorize sending translation requests or downloading models.

`misc/flathub.sh` downloads the upstream Flatpak Cargo generator, which now declares its own
PEP 723 dependencies (`aiohttp`, `PyYAML`, `tomlkit`). Run it through isolated
`uv run --no-project --python '>=3.15' --upgrade --prerelease disallow --script`, rather than replacing the
project environment with Python 3.11 and manually installing the obsolete dependency list.
The reviewed compatible stable versions are aiohttp 3.14.5, PyYAML 6.0.3 and tomlkit 0.15.2.

## Previous review: 2026-10-29 (source refresh; CI deferred)

- Official `master` remains `eb8b91db`; no upstream merge is required. Existing fork features,
  cache bytes, optional decoders, GTK removal and 8192 hash sizes are unchanged.
- Replaced the three `ubuntu-latest` workflow sites with explicit `ubuntu-26.04`. The notice is
  GitHub's [October 19-November 19 image migration announcement](https://github.com/actions/runner-images/issues/14748),
  not a compiler warning. Kept the Linux artifact ABI baseline and all quality checks intact.
- Normal CI Rust setups now select `stable` instead of freezing every job at `1.94.1`;
  the explicitly named minimum-Rust job remains at the declared minimum. Workspace metadata
  resolved 943 packages without a declared Rust minimum above `1.94.1`; this is not an MSRV build.
- Retained ZIP `9.1.3`, lopdf `0.46.0`, usvg `0.49.0`, handsome_logger `0.10.0` and refreshed
  compatible lockfile packages. Updated standalone logger requirements and adapted the shared
  logger's `set_write_once` call to `set_single_write`, preserving whole-record writes.
- Centralized the Slint family in the workspace, removed unnecessary exact patch pins and kept
  Fontique `0.11.1` aligned with Slint `1.18.1`. Metadata assertions confirmed inherited disabled
  default features and a single shared Fontique instance with `fontconfig-dlopen` enabled.
- Python tools now use `ruff>=0.17.3`, `mypy>=2.4.0` and `ty>=0.0.93`. Preview/update recipes
  refresh Python resolutions as well as Rust dependencies, without activating a bot or CI run.
- Adopted the published AGP `9.6.0-alpha02` / KGP `2.5.0-Beta2` previews, without changing SDK
  levels, NDK, permissions or built-in Kotlin. Maven publication is verified; new AGP POM
  requests also encountered TLS EOF, so successful Gradle resolution is not claimed.
- Retired the old Docker verification exception: the official Ubuntu `26.10` registry header
  digest and SHA-256 of the returned OCI index body both matched the selected
  `sha256:9c3a42b7c0016cbf87ceb71fd72d26c0e73b37e532e2c6071707f124c489c118`.
  A later repeat request was interrupted; no container build was performed. Active Actions
  already match current official releases, so their full security/reproducibility SHAs remain.
- Validation is limited to workflow syntax, Just recipe parsing, dependency previews/metadata,
  source API review and diff checks. Full `just fix`/`just clip`, builds, tests, Gradle and hosted
  CI remain deferred under the requested source-update/CI-budget constraint. Publication was
  subsequently authorized; this maintenance commit uses `[skip ci]` to avoid a new push-triggered
  CI run. No checks were removed, and no manual dispatch, rerun or CI monitoring was performed.

## Previous review: 2026-10-19

- Official master is still `eb8b91db`; the fork contains every official commit. Kept sevenz-rust2
  `0.24.1`, AGP `9.5.0-alpha10` and setup-java `6.1.0` at its verified release SHA.
- Corrected the existing independent Fontique `0.12.0` edit to `0.11.1`: Slint `1.18.1` still
  depends on the `0.11` package, so enabling dlopen on `0.12` would not configure Slint's fonts.
  Bincode, renderer/decoder features and Android SDK levels remain unchanged.
- Gradle `help`, `buildEnvironment` and all app task registration passed together with
  `--warning-mode fail` on Gradle `9.8.1` / JDK `27`. Android build helpers now retain javac/D8
  diagnostics instead of hiding successful-tool warnings; this is not APK/device acceptance.
- Preserved the six pre-existing dependency edits in the non-clearing local stash
  `pre-warning-hardening dependency edits 2026-10-19` before integration. Retained the verified
  Docker image for the registry-verification reason above; no old stash or app bundle was removed.
- `just upgrade-check` found no additional stable direct updates outside the reviewed exceptions.
  Final `just fix` and `just clip` passed without warnings on Rust `1.94.1`; locked workspace tests
  passed 383 cases with zero failures and six existing ignored cases. Five standalone fixture tests,
  strict standalone clippy and Krokiet zh-CN validation passed. This does not establish APK/device,
  live GUI, container-build or archive-driven CLI integration acceptance.

## Previous review: 2026-10-14

- Fresh official Git refs and GitHub commit metadata agree on `upstream/master` at
  `eb8b91dbb4d25dff416202674ee0de84ee4b5e5c`. The fork contains it; no newer merge was needed.
- Adopted AGP `9.5.0-alpha09` and explicitly upgraded its embedded KGP from `2.2.10` to the
  latest published preview `2.5.0-Beta1`. A traced `Configuration.setVisible` warning came
  from KGP, not project code. `gradle help --warning-mode fail --no-daemon` passed on Gradle
  `9.8.1` / JDK `27` without that warning. No warning suppression or built-in Kotlin opt-out
  is used. See [Android's higher-KGP upgrade instructions](https://developer.android.com/build/releases/agp-9-0-0-release-notes#upgrade-to-a-higher-kgp-version).
- Android Rust libraries remain at the latest compatible releases: android-activity `0.6.1`,
  jni-high `0.1.0`, android_logger `0.15.1`, android-build `0.1.4` and matched Slint `1.18.1`.
  The `backend-android-activity-06` type boundary is retained.
- Updated the Android workflow's NDK from `27.2.12479018` to stable r30 LTS
  `30.0.16248370`, verified in Google's SDK repository stable channel. App SDK levels,
  signing and runtime permission behavior are unchanged. The local host has no Android
  SDK/NDK or Rust Android target, so no APK build or device verification is claimed.
- Preserved existing stable dependency updates, including ZIP `9.0.3`, sevenz-rust2 `0.24.0`,
  rodio `0.23.0` and rust-embed `8.13.0`, and updated standalone tools. Fontique stays aligned
  at `0.11.1`; bincode and GTK-removal invariants are unchanged. Docker keeps the previously
  verified Ubuntu `26.04` digest; a newer digest could not be fetched due to registry TLS errors.
- Removed the two macOS conditional-dependency warnings: the Linux-only portal dependencies
  now use target-specific declarations, and the fallback RAW dependency remains explicit
  when LibRaw is selected. `just fix` passed without warnings; a Linux-target dependency
  graph check still resolves ashpd through the portal feature.
- Locked workspace tests passed: 377 including doctests, zero failures and six existing
  ignored cases. Exact filename/reference-folder scans, protected Space selection and
  bincode-1-compatible cache byte fixtures passed. `just clip` passed without warnings and
  Krokiet zh-CN translations validated successfully.
- Both changed standalone tools passed all-target compilation after adapting the regression
  tester's ZIP 9 filename result to fail with archive/entry context on invalid names.
  Android `help`, all app task registration and resolved build dependencies passed with
  `--warning-mode fail`. These checks do not replace an APK build, device test or live GUI
  acceptance; no installed app bundle was replaced and hosted CI was not monitored.

## Previous review: 2026-10-03

- Workspace direct dependencies are at their latest stable requirements, except the documented
  bincode retirement. Slint/slint-build remain at the current stable `1.18.1`.
- Updated native HEIF integration to `libheif-rs 3.0.0`, retaining the optional `v1_17` ABI floor.
- Updated audio tags, document/archive/markup validation and RAW image dependencies without
  changing the cache wire format or Krokiet's extended hash-size configuration.
- Android Gradle plugin is `9.4.1`; it requires Gradle `9.6.0` or newer and JDK `17` or newer.
  There is no checked-in Gradle wrapper. Use a current compatible stable Gradle installation.
- Docker uses official Ubuntu `26.04` with a verified multi-platform digest. Ubuntu `26.10`
  and AGP `9.5.0-alpha08` were not adopted because they are prereleases at this review date.
- Updated active GitHub Actions to verified current stable release SHAs. Hosted CI and release
  artifacts are outside the local validation claim.
- Python quality tools use Ruff `0.16.8`, mypy `2.4.0` and ty `0.0.84`. The minimum remains
  Python `3.13`, without excluding newer stable interpreters. Refresh these pins on each pass
  and validate the scripts with the new versions rather than treating tool pins as freezes.

### Local validation (macOS ARM64)

- `just fix` passed with the new Ruff/mypy versions; `just clip` passed the platform-compatible
  features and software-only configuration. The two existing manifest unused-dependency
  warnings for ashpd/rawler in the macOS all-features combination remain; no code warnings
  were introduced and no feature was removed to hide them.
- Locked workspace tests passed: 377 tests including doctests, zero failures and six existing
  ignored cases. Legacy-cache byte fixtures and Exact File Names regressions passed.
- The full workspace/all-targets check and a core/all-targets check with latest stable Rust
  `1.99.0` passed. Changed standalone tools compiled; the two benchmark warning fixes were
  rechecked without warnings.
- This is source-level local validation, not hosted CI, a Docker image build, an Android
  package/device test or manually clicked GUI acceptance. Existing app bundles were untouched.
