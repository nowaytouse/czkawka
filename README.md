<div align="center"><img src="https://github.com/user-attachments/assets/f5e4b290-d001-4cf4-9f52-dab65a30e441" alt="krokiet_logo" width="600" /></div>
     
**Krokiet** ((IPA: [ˈkrɔcɛt]), "croquette" in Polish) new generation GUI frontend, simple, multiplatform, fast and free app to remove unnecessary files from your computer.


<div align="center"><img src="https://github.com/user-attachments/assets/ed6dfeea-a984-49e8-a621-8d6ae521c760" alt="cedinia_logo" width="600" /></div>

**Cedinia** - Android touch friendly GUI frontend for Czkawka Core, built with Slint.

## Features

- **Written in memory-safe Rust** - almost 100% unsafe code free
- **Amazingly fast** - due multithreading and efficient algorithms
- **Free, Open Source without any ads**
- **Multiplatform** - runs on Linux, Windows, macOS, FreeBSD, x86, ARM, RISC-V and even Android
- **Cache support** - second and further scans should be much faster than the first one
- **Easy to run, easy to compile** - minimal runtime and build dependencies, portable version available
- **CLI frontend** - for easy automation
- **GUI frontend** - Krokiet uses Slint
- **Core library** - allows to reuse functionality in other apps
- **Android app** - touch-friendly frontend for Android devices
- **No spying** - Czkawka does not have access to the Internet, nor does it collect any user information or statistics
- **Multilingual** - support multiple languages like Polish, English or Italian
- **Multiple tools to use**:
    - **Duplicates** - Finds duplicates based on file name, size or hash
    - **Empty Folders** - Finds empty folders with the help of an advanced algorithm
    - **Big Files** - Finds the provided number of the biggest files in given location
    - **Empty Files** - Looks for empty files across the drive
    - **Temporary Files** - Finds temporary files
    - **Similar Images** - Finds images which are not exactly the same (different resolution, watermarks)
    - **Similar Videos** - Looks for visually similar videos
    - **Same Music** - Searches for similar music by tags or by reading content and comparing it
    - **Invalid Symbolic Links** - Shows symbolic links which point to non-existent files/directories
    - **Broken Files** - Finds files that are invalid or corrupted
    - **Bad Extensions** - Lists files whose content not match with their extension
    - **Exif Remover** - Removes Exif metadata from various file types
    - **Video Optimizer** - Crops from static parts and converts videos to more efficient formats
    - **Bad Names** - Finds files with names that may be not wanted (e.g., containing special characters)

![Krokiet](https://github.com/user-attachments/assets/3cc7ec6a-3d6a-42cb-9d33-4b0f0c547af6)

Changelog about each version can be found in [CHANGELOG.md](Changelog.md).

New releases can be found in [Github releases](https://github.com/qarmin/czkawka/releases) and nightly builds also in [Nightly releases](https://github.com/qarmin/czkawka/releases/tag/Nightly)

You can read more about the 12.0 release, its new features, and the issues that were fixed in the following articles:
- English article - https://medium.com/@qarmin/krokiet-czkawka-12-0-6fa09c43c3b9
- Polish article - https://medium.com/@qarmin/krokiet-czkawka-12-0-c5dad2116793

## Fork-specific features (all-features branch)

This branch tracks upstream [`qarmin/czkawka`](https://github.com/qarmin/czkawka) `master` (last merged: `105a520b`, release `12.0.1`) and adds:

### From upstream (merged regularly)

- **Geometric invariance (Similar Images)** - match mirrored/flipped images; optional 90-degree rotation (`Off` / `Mirror + Flip` / `Mirror + Flip + Rotate 90`). Similar-images cache files change when this setting changes; the cache is regenerated automatically.
- Other upstream fixes and features from the 11.0.x line (see [Changelog.md](Changelog.md)).

### Fork-only

- **File protection** - mark any result as protected so it is never deleted, moved, hardlinked, symlinked or renamed by the app. Protected files stay visible in the results with an amber marker and a disabled checkbox, and are remembered across scans and restarts (stored in `protected_files.json`). Protect/unprotect a whole selection with the toolbar buttons, or a single file from the right-click context menu; clear the whole set from Settings.
- **Similar Images extras** - hash sizes up to `8192`, **only same size** filter, **size ratio** filter, and exact byte size in results (on top of upstream similarity settings).
- **Select all except highest quality** - selection mode in Similar Images that spares the largest pixel count in each group (file size breaks ties).
- **Modernized Krokiet UI** - updated color system, clearer active and hover states, improved spacing, and refreshed popup and list styling.
- **Simplified Chinese (zh-CN)** - Noto Sans SC is bundled in Krokiet; maintain it with `just sync-zh-cn`; system locales such as `zh`, `zh-CN`, and `zh-Hans-CN` map to zh-CN on first run.
- **Krokiet-only fork policy** - the legacy `czkawka_gui` GTK source, packaging, and launchers are intentionally removed. CLI and Cedinia follow upstream behavior; shared core changes are limited to capabilities required by Krokiet and cache compatibility.

### Dependency stack (fork maintenance)

The fork keeps **bincode 2** with the legacy wire format so existing cache binaries remain readable. Other dependency versions follow the merged upstream baseline unless Krokiet requires a targeted change. See [Changelog.md](Changelog.md) under *Fork Modifications*.

### Maintaining the fork

```bash
git fetch upstream
git merge upstream/master   # preserve the GTK deletion and Krokiet-only features
just fix
```

See [AGENTS.md](AGENTS.md) for i18n (`just sync-zh-cn`) and architecture notes.

## Usage, installation, compilation, requirements, license

Each tool uses different technologies, so you can find instructions for each of them in the appropriate file:

- [Krokiet GUI (Slint frontend)](krokiet/README.md)</br>
- [Czkawka CLI](czkawka_cli/README.md)</br>
- [Czkawka Core](czkawka_core/README.md)</br>
- [Cedinia](cedinia/README.md)</br>

## Other apps

There are many similar applications to Czkawka on the Internet, which do some things better and some things worse:

### GUI

- [DupeGuru](https://github.com/arsenetar/dupeguru) - Many options to customize
- [FSlint](https://github.com/pixelb/fslint) - A little outdated, but still have some tools not available in Czkawka
- [AntiDupl.NET](https://github.com/ermig1979/AntiDupl) - Shows a lot of metadata of compared images
- [Video Duplicate Finder](https://github.com/0x90d/videoduplicatefinder) - Finds similar videos(surprising, isn't it)

### CLI

Due to limited time, the biggest emphasis is on the GUI version so if you are looking for really good and feature-packed
console apps, then take a look at these:

- [Fclones](https://github.com/pkolaczk/fclones) - One of the fastest tools to find duplicates; it is written also in
  Rust
- [Rmlint](https://github.com/sahib/rmlint) - Nice console interface and also is feature packed
- [RdFind](https://github.com/pauldreik/rdfind) - Fast, but written in C++ ¯\\\_(ツ)\_/¯


## Projects using Czkawka

Czkawka exposes its common functionality through a crate called **`czkawka_core`**, which can be reused by other projects.

It is written in Rust and is used by the Czkawka CLI, Krokiet, and Cedinia.

It is also used by external projects, such as:

- **Czkawka Tauri** - https://github.com/shixinhuang99/czkawka-tauri - A Tauri-based GUI frontend for Czkawka.
- **page-dewarp** - https://github.com/lmmx/page-dewarp - A library for dewarping document images using a cubic sheet model.

Bindings are also available for:

- **Python** - https://pypi.org/project/czkawka/

Some projects work as wrappers around `czkawka_cli`. Without directly depending on `czkawka_core`, they allow simple scanning and retrieving results in JSON format:

- **Schluckauf** - https://github.com/fadykuzman/schluckauf

## Thanks

Big thanks to Pádraig Brady, creator of fantastic FSlint, because without his work I wouldn't create this tool.

Thanks also to all the people who contributed to the project in every possible way

Also, I really appreciate work of people that create crates on which Czkawka is based and for that I try to report bugs to make it even better.

## How to help?

- **Creating issues** - Mainly related to bugs, oddly behaving functionality, etc. As you can see from the issue tracker, there are plenty of ideas for new features, but most of them are either difficult to implement or not aligned with the vision of the project, which evolves slightly over time.
- **Creating pull requests** - Bug fixes are of course very welcome. Regarding new features, it is best to consult with me before implementing them to confirm they align with the project vision. A POC implemented in Rust as external script/project would be useful, especially for more complex features, to ensure there are no technical limitations.
- **Updating translations** - The project uses the Crowdin platform, where translations can be created and updated. In the case of a new release and missing translations, I use machine translation, which is often inaccurate, so updating translations is highly appreciated.
- **Creating packages for various platforms** - Due to the difficulties related to adding and maintaining support for each new platform, such as learning package formats like deb or rpm, creating installers and packages, I decided to mainly focus on providing prebuilt binaries. However, having the project available in distribution repositories or in projects such as Chocolatey, Homebrew or Winget would be beneficial for users who prefer centralized repositories.
- **Creating articles, videos, tutorials, etc.** - Any material that helps people better understand this program and its capabilities is welcome.
- **Recommending it to friends, family, coworkers, etc.** - This is probably the simplest way to help the project become even more popular, which gives me motivation to continue developing the program. Here are a few example ways to naturally mention this program in a regular conversation:

**S** - Someone  
**Y** - You  

### Situation 1:

- **S** - Hey Anon, I have a lot of junk on my disk, what should I do?
- **Y** - Download Krokiet/Czkawka. They are completely free and works on almost every system.
- **S** - Thanks man!

### Situation 2:

- **S** - I am so thirsty...
- **Y** - Have you heard about Krokiet/Czkawka?
- **S** - Wait, what?
- **Y** - Krokiet and Czkawka, in case you did not know, let you clean unnecessary files from your disk. They are completely free...
- **S** - That is nice, but I am thirsty...
- **Y** - ...they work on Windows, Linux and macOS, and some people even port them to FreeBSD and Android...


## AI Policy
The vast majority of the code in this project was written by me (qarmin) without using AI. However, as AI tools have improved and can significantly simplify development and reduce boilerplate, I see no reason to forbid their use. I have also added a AGENTS.md file to the repo to make it easier to provide AI tools with context about the project’s style and code structure.

That said, every pull request, whether created with AI or not, must meet proper quality standards. The author must be able to clearly explain what the code does, without relying on AI for that explanation. I manually review every PR and test each change, so the risk of incorrect code slipping through is low. Still, to avoid wasting time, please refrain from submitting AI Slop PRs.

## Officially Supported Projects
This fork publishes and validates Krokiet and the Czkawka CLI. Cedinia is kept at the upstream baseline.

Czkawka does not have an official website, so do not trust any sites that claim to be the official one.  

If you use packages from unofficial sources, make sure they are safe.

## License

All images and audio files are licensed under the [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) license.

Czkawka Core and CLI are licensed under the [MIT](https://mit-license.org/) license. Krokiet and Cedinia are licensed under [GPL-3.0-only](https://www.gnu.org/licenses/gpl-3.0.en.html) due to Slint license requirements.

## Donations

If you are using the app, I would appreciate a donation for its further development, which can be
done [here](https://github.com/sponsors/qarmin).
