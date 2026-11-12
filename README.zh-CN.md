# Czkawka / Krokiet

[English](README.md) · **简体中文**

Czkawka 是本地文件扫描工具。Krokiet 是桌面图形界面，Cedinia 是 Android 界面，`czkawka_cli` 提供命令行操作。它们共用 Rust 扫描核心。可以扫描重复文件、相似图片和视频、空文件夹、损坏文件、错误扩展名等。先选择扫描目录和工具，核对结果后再执行移动、删除、改名或链接操作。

## 本 fork 的桌面功能

- **精确文件名**独立于「重复文件」工具，按包含扩展名且区分大小写的文件名分组。同名文件可能内容不同，改名后重新扫描可发现新匹配。
- **文件保护**可对选中结果或单个文件启用。保护记录跨重启保留，受保护条目仍可查看，应用会阻止对它们执行删除、移动、改名和链接操作。
- **相似图片**支持镜像、翻转和可选的 90 度旋转匹配，也可按字节大小和大小比例筛选。组内选择可保留像素数最高的图片，像素数相同时比较文件大小。修改几何匹配选项后会重新生成相应缓存。
- **简体中文**可在 Krokiet 的语言设置中使用。首次运行会将 `zh`、`zh-CN` 或 `zh-Hans-CN` 映射到简体中文。Krokiet 随包提供 Noto Sans SC。

## 获取与构建

按设备平台和 CPU 架构选择安装包。上游发行说明和资产位于 [Czkawka Releases](https://github.com/qarmin/czkawka/releases)。需要本 fork 的独立精确文件名和保护功能时，使用本 fork 对应的构建或从本仓库 `master` 编译。各平台的依赖和构建参数见[英文构建说明](README.md)。

维护时以当前 `master` 和提交的 `Cargo.lock` 为基线。先用 `just upgrade-check` 核对候选更新，执行 `just fix`、`just clip` 和 `cargo test --locked --workspace`。Slint 与 Fontique 需要沿用相互兼容的系列。Android 工具链的预览依赖须单独验收。 [依赖维护说明](DEPENDENCIES.md)

## 翻译维护

现有界面使用 Fluent 资源。同步简体中文时运行 `just sync-zh-cn`，并核对变量、选择表达式和缺失键。扫描和导出验证使用合成文件或临时目录，翻译中避免加入设备路径、文件清单或操作日志。 [贡献与翻译检查](AGENTS.md)
