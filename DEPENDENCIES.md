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
| Slint family | Matched exact runtime/build releases; compatible fontique | Update pins together when a newer stable Slint is published. Confirm the prior Wayland window-size fix, renderer support and bundled Chinese font remain functional. |
| macOS Skia | Platform-compatible feature set | Upstream Skia has separate Metal and Vulkan macOS prebuilt artifacts. Validate supported combinations separately; do not combine unsupported backends or remove them from other platforms. |

The bincode retirement notice is documented in the [published source](https://docs.rs/crate/bincode/3.0.0/source/src/lib.rs).
The [Android Gradle plugin compatibility table](https://developer.android.com/build/releases/about-agp)
defines the corresponding Gradle requirements. Recheck both when updating; this document does
not assert that an old exception remains necessary indefinitely.

## Last reviewed: 2026-10-03

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
