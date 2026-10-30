# Test compilation speed size
This project is used to get the compilation speed and size of generated rust binaries for different configurations.

## How to use?
- install app `cargo install --path .`
- configure json config, just like `test.json` or `krokiet.json` - allowed values you can find in `src/model.rs`
- go to your project root(if project is in workspace, you need to go to the workspace root)
- run `test_compilation_speed_size config.json`
- fix compilation errors if any happens(mixing some compilation flags can cause compilation errors)
- use the Python dependency group declared in `misc/pyproject.toml`; `uv` installs pandas, matplotlib and tabulate without changing system Python packages
- from the Czkawka repository root, generate charts with `uv run --project misc --extra plots python misc/test_compilation_speed_size/generate_md_and_plots.py`; the command reads `compilation_results.txt` from the current directory
- generated md and png files will be in `charts` folder
- charts prefer an installed Noto Sans font and otherwise use Matplotlib's bundled DejaVu Sans, without requiring a system font installation
