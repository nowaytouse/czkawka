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
   The preview includes major releases and exact pins. It ignores the old Rust minimum when
   discovering releases so an outdated MSRV cannot silently hide a required migration.
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
   together; keep direct fontique compatible with Slint's resolved fontique.
6. Run `just fix`, `just clip` and `cargo test --locked --workspace`. Check changed standalone
   crates with their own manifests. Validate native decoder features where the platform
   supports them; desktop checks do not prove Android, Linux, Windows or live GUI behavior.
7. Review the complete diff and `git diff --check`. Record updates and exceptions in the
   fork changelog. Explicitly stage intended files, commit with an `[AI]` title, push
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
| Slint family | Matched exact runtime/build releases; compatible fontique | Update pins together when a newer stable Slint is published. The updater excludes independent fontique bumps: Slint 1.18.1 exposes the fontique-011 API and needs the 0.11.1 dlopen feature configuration. Revisit when the Slint family changes; preserve renderer support and the bundled Chinese font. |
| Android previews | AGP 9.5.0-alpha09 and KGP 2.5.0-Beta1 | Explicitly requested on 2026-10-14. Keep built-in Kotlin enabled and use Android's documented higher-KGP classpath mechanism. Recheck the latest preview coordinates and Gradle warning gate on each Android dependency pass; do not extend this opt-in to other ecosystems or alter app SDK levels without reviewing runtime behavior. |
| macOS Skia | Platform-compatible feature set | Upstream Skia has separate Metal and Vulkan macOS prebuilt artifacts. Validate supported combinations separately; do not combine unsupported backends or remove them from other platforms. |

The bincode retirement notice is documented in the [published source](https://docs.rs/crate/bincode/3.0.0/source/src/lib.rs).
The [Android Gradle plugin compatibility table](https://developer.android.com/build/releases/about-agp)
defines the corresponding Gradle requirements. Recheck both when updating; this document does
not assert that an old exception remains necessary indefinitely.

## Last reviewed: 2026-10-14

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
