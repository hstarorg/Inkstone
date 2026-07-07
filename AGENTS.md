# Inkstone Agent 规约

Inkstone（砚台）：本地优先、端到端加密的跨平台桌面个人知识库。Tauri 2 + React 18 + TypeScript + Vite + Tailwind + shadcn/ui；Rust 侧负责文件 IO、SQLite（FTS5）、加密、同步；编辑器 Tiptap v3，画布 Plait。

## 权威文档

- `docs/ROADMAP.md` — 路线图、已决策项、各里程碑验收标准。**范围问题以此为准**：非目标（协同编辑、移动端/Web、AI、插件系统）一律不做，新想法追加到 ROADMAP 而不是直接实现。
- `docs/FORMAT.md` — 磁盘格式规范的唯一权威（vault 布局、文档 JSON、加密、同步 manifest）。

## 硬性规则

- **逐任务确认**：每个任务动手前先向用户说明要做什么并获得确认，一次只推进一个已确认的任务；用户一句"继续推进"不构成对后续任务的批量授权。
- **git 只读**：禁止一切 git 写操作（add / commit / push / reset / branch / stash / tag 等），只允许 status、log、diff 等只读查询。提交永远由用户本人完成。
- **格式先行**：任何落盘格式的新增/修改，必须先更新 `docs/FORMAT.md` 并经用户确认，再写代码；实现须遵守其总则（formatVersion、兼容策略、原子写）。
- **加密**：只用成熟 crate（RustCrypto / libsodium 系），禁止自组合密码学原语；密钥材料不出 Rust 侧、不进日志与错误信息；FORMAT.md 加密章节定稿前禁止实现加密落盘代码。
- **架构边界**：前端不直接访问文件系统/数据库，一切经 Tauri command；Plait 只经画布窄接口访问；`index.db` 是可重建的派生数据，不得作为唯一数据源。
- **跨平台**：改动不得破坏 Windows/Linux 构建；快捷键用 Cmd/Ctrl 抽象，禁止硬编码单平台修饰键。

## 工程约定

- **项目/工具初始化必须用官方 CLI**（如 `pnpm create tauri-app`、`pnpm dlx shadcn@latest init`、`pnpm create vite`），禁止手写脚手架配置文件冒充初始化产物；**执行任何初始化命令前，先向用户列出具体命令并确认**。
- 包管理用 pnpm；提交前 `pnpm check`（lint + typecheck + 前后端测试）必须全绿。
- TypeScript strict 模式；Rust 零 clippy 警告。
- 每个里程碑以 ROADMAP.md 中的验收标准为完成定义，实现时先写能验证该标准的测试。

## 命令

脚手架尚未搭建（M0 进行中）。M0 完成后在此登记：dev 启动、测试、构建、打包命令。
