# Rolling dependency maintenance

The `all-features` fork tracks the latest **stable** dependencies. Major upgrades are in scope:
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
   Python tool requirements use rolling lower bounds. The updater copies `misc/pyproject.toml`
   to the generated root project and refreshes its local lock with prereleases disallowed;
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
   `origin/all-features`, pull with `--ff-only`, and verify clean tracked/untracked porcelain
   plus zero ahead/behind against the fetched fork branch. Preserve ignored app bundles,
   backups and existing stashes.

These are repository maintenance rules and local commands, not an activated hosted updater.
No CI polling, heartbeat or background monitor is required. Do not modify the upstream
repository, remote default branch, installed apps or release binaries as part of this procedure.

## Reviewed exceptions

| Dependency | Current contract | Reason and exit condition |
|------------|------------------|---------------------------|
| `bincode` | Exact `2.0.1`, serde with legacy variable-integer encoding | Published `3.0.0` is a retirement notice whose source deliberately fails compilation and has no serde feature. Re-evaluate a maintained replacement on each update pass; migrate only with byte-level compatibility fixtures for existing 1.3+ caches, or explicitly version new cache files and document invalidation. |
| Slint family | Shared `~1.18.1` runtime/build requirements; Fontique `0.11.1` series | Permit patch updates without independently changing the font API. Slint 1.18.1 still needs the 0.11 dlopen feature configuration, not Fontique 0.12. The preview reports the independent latest release; the mutating updater excludes it until a coordinated Slint migration. Preserve renderer support and the bundled Chinese font. |
| Android previews | AGP 9.6.0-alpha02 and KGP 2.5.0-Beta2 | Explicitly requested on 2026-10-14. Keep built-in Kotlin enabled and use Android's documented higher-KGP classpath mechanism. Recheck published coordinates and the Gradle warning gate on each Android dependency pass; do not extend this opt-in to other ecosystems or alter app SDK levels without reviewing runtime behavior. |
| Linux release ABI | Explicit Ubuntu 24.04 artifact builders | Updating a release builder can raise the required system-library baseline. Move it only after reviewing supported distributions; normal moving-alias jobs now explicitly use the current hosted Ubuntu 26.04 GA image. |
| macOS Skia | Platform-compatible feature set | Upstream Skia has separate Metal and Vulkan macOS prebuilt artifacts. Validate supported combinations separately; do not combine unsupported backends or remove them from other platforms. |

The bincode retirement notice is documented in the [published source](https://docs.rs/crate/bincode/3.0.0/source/src/lib.rs).
The [Android Gradle plugin compatibility table](https://developer.android.com/build/releases/about-agp)
defines the corresponding Gradle requirements. Recheck both when updating; this document does
not assert that an old exception remains necessary indefinitely.

## Last reviewed: 2026-10-29 (source refresh; CI deferred)

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
