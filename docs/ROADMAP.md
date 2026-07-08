# Inkstone 开发计划

## 目标与非目标

**目标**：跨平台桌面个人知识库。本地文件存储、离线全文检索、文档内混排画布（脑图/流程图/手绘）、静态加密可选、端到端加密的可插拔同步。平台策略：**macOS 一等公民**（深度测试、体验打磨），Windows / Linux 保证可用（CI 构建 + 发布安装包 + 基础冒烟，不逐版本深测）。

**非目标**（明确不做，防止范围蔓延）：

- 协同编辑（无 CRDT、无实时协议）
- 移动端 / Web 端（架构不排斥，但不为其做任何预留工作）
- AI 功能
- 插件系统（画布节点的窄接口除外）

## 基础假设（有异议请在动工前推翻）

- 技术栈：Tauri 2 + React 18 + TypeScript + Vite，包管理 pnpm；UI 组件 shadcn/ui + Tailwind
- License：MIT（已落 LICENSE 文件）
- 跨平台约束从第一天生效：路径处理全走 Rust 侧统一 API（大小写敏感性、路径分隔符、文件名非法字符三平台不同）；快捷键用 Cmd/Ctrl 抽象
- Rust 侧承担：文件 IO、SQLite、加密、同步；前端不直接碰文件系统
- 数据目录（vault）：用户选定的一个文件夹，结构如下：

```
MyVault/
├── inkstone.json          # vault 元信息、格式版本号
├── docs/
│   └── <ulid>.json        # 每文档一个 ProseMirror JSON 文件（画布场景内联其中）
├── assets/                # 图片等二进制资源，内容寻址命名（hash）
└── .inkstone/
    └── index.db           # SQLite 索引，可随时删除重建，不参与同步
```

- 文档标识用 ULID（时间有序、文件名安全）；标题是文档内容的一部分，不是文件名
- 所有磁盘格式（文档 JSON、manifest、加密 blob）从第一天起带 `formatVersion` 字段

## 里程碑

### M0 · 脚手架（预计 0.5 周）

- [x] pnpm + Vite + React + TS 初始化，Tauri 2 接入，`pnpm tauri dev` 跑通（create-tauri-app 官方 CLI；`pnpm build` + `cargo check` 验证通过）
- [x] 品牌 logo（`docs/assets/logo.svg`）与全平台应用图标（`pnpm tauri icon`）、web favicon —— 计划外提前完成
- [x] Tailwind + shadcn/ui 接入，基础布局壳（侧栏 + 主区）（Tailwind v4 Vite 插件 + shadcn CLI，radix 基座 / Nova preset）
- [x] 基础工程化：ESLint + Prettier + rustfmt + clippy，`pnpm check` 一键全查（ESLint 9 flat config + typescript-eslint；`@eslint/create-config` 向导不支持非交互，按官方文档手写）
- [x] GitHub Actions：lint + 前后端测试，macOS / Windows / Linux 三平台矩阵构建（workflow 已就位，待首次 push 后在线验证）

**验收**：新克隆的仓库按 README 三条命令内能跑起空窗口应用；三平台 CI 绿。

### M1 · Vault 与文档 CRUD（预计 1–1.5 周）

- [x] Rust：vault 创建/打开（记住最近路径，存 OS 配置目录）、`inkstone.json` 读写与版本校验
- [x] Rust：文档 list / read / write / trash / restore 的 Tauri command，12 个单元测试全过（含损坏文件、版本超限、未知字段保真、孤儿 tmp 清扫）
- [x] 前端：欢迎页（打开/新建 vault）、文档列表侧栏、新建/删除（重命名 = 改首行，标题派生）
- [x] Tiptap 接入：StarterKit 级别的编辑（M0 后已远超此范围）
- [x] 自动保存：500ms 防抖写盘 + 切换/删除时 flush + 原子写崩溃安全
- [x] 文档 JSON schema 定稿并写文档（`docs/FORMAT.md`，已评审）

**验收**：断网、杀进程、重启后无数据丢失；手工构造损坏 JSON 时应用报错而非崩溃。

### M2 · 编辑器能力（核心，分批实现）

> 编辑器是产品核心。能力基线对标 Notion（块模型/交互）、语雀（中文场景的富内容块）、YouMind（Markdown 优先的干净写作面）。
> 全部实现在 `richer-editor` 模块内，保持业务隔离。AI 辅助与协同类能力按非目标排除。
> 已具备：StarterKit 基础块与行内标记、Markdown 快捷输入、气泡工具栏（块级切换 + 标记 + 链接 popover）、代码块语法高亮（lowlight）、Placeholder、智能排版。

**P0 · 结构与输入基线**

- [x] Slash 菜单：`/` 唤起块插入器，支持关键词过滤、键盘导航（12 种块类型）
- [x] 表格：表头、行列增删（气泡栏表格操作组）；单元格对齐并入 P2 表格进阶
- [x] 任务列表（可勾选，支持嵌套）
- [x] Callout 信息块：info / tip / warn / danger 变体（点击图标循环切换，slash 菜单插入）
- [x] 折叠块（toggle/details，标题 + 可折叠内容，保留开合状态）
- [x] 高亮标记：`==text==` 快捷输入 + 气泡栏 5 色色板
- [x] 图片：粘贴/拖入 → `assets/` 内容寻址去重 → 文档存 `asset://` 引用；拖拽手柄缩放，宽度持久化（说明文字 → P2 打磨）
- [x] 块拖拽手柄：拖动重排、块选中（官方 DragHandle + NodeRange）
- [x] Markdown 互操作：粘贴 Markdown 解析（启发式识别）、复制纯文本即 Markdown（官方 `@tiptap/markdown` + 自定义剪贴板扩展；callout/details 序列化留 M7 导出时补）

**P1 · 写作辅助**

- [x] 查找替换（Cmd/Ctrl+F 唤起，逐个跳转/替换/全部替换，命中高亮）
- [x] 字数统计（右下角常驻徽标，可用 `showCharacterCount` 关闭）
- [x] Slash 菜单中文关键词与拼音别名（如 `/表格`、`/biaoge`）
- [x] 清除格式（气泡栏按钮，`unsetAllMarks`）

**P2 · 体验打磨**

- [ ] 大纲面板：标题层级、点击跳转
- [ ] 文档内 TOC 块
- [ ] 多栏布局（columns）
- [ ] 表格进阶：合并单元格、列宽拖拽
- [ ] 专注/打字机模式
- [ ] 段落对齐（左/中/右，text-align）
- [ ] 文本颜色（气泡栏色板，与高亮同交互）
- [ ] 可选固定工具栏（undo/redo、标题下拉、颜色、对齐——提升可发现性，默认关闭，参考语雀顶栏）
- [ ] 界面文案抽离（全英文，见 AGENTS.md 工程约定；留 i18n 结构）

**P3 · 富内容块（低优先级，按需再排期）**

- [ ] 数学公式：行内 `$x$` + 块级（KaTeX）
- [ ] Mermaid 图表块：源码编辑 + 渲染预览
- [ ] 链接卡片/书签块：贴 URL 转卡片（本地抓取标题/摘要，做成可关闭的隐私开关）
- [ ] 附件块：任意文件落 `assets/`，显示文件名/大小（依赖 M1 vault）
- [ ] 视频/音频：本地文件嵌入播放
- [ ] Emoji 选择器（`:` 触发）

**明确排除或归属其他里程碑**

- AI 写作/润色、AI 生成图片 → 非目标
- 评论、协同光标、分享发布 → 非目标
- 数据库/多维表（Notion database、语雀数据表）→ 超出个人文档定位，不做
- 格式刷、行高/段落缩进 → 不做（保持文档语义化，不引入排版属性；Notion 亦无）
- Status 标签块 → 不做（低频，Callout 可替代）
- 文档历史版本（语雀 latest version）→ 非编辑器职责，属存储层，M6 后评估
- 画布/白板块 → M3（Plait）
- 双链 `[[`、反链、标签 → M4
- Markdown/HTML 全库导出 → M7

**验收**：P0 完成后——把一篇含图片/表格/代码/callout 的真实笔记从 Notion 或语雀手工搬入，编辑体验无明显缺口；P1 完成后——含公式和 Mermaid 的技术笔记可完整表达。

### M3 · Plait 画布节点（预计 1.5–2 周，含技术验证）

- [ ] **技术 spike（先做，1–2 天）**：独立 demo 验证 Plait React 组件的嵌入、JSON 序列化/反序列化、PNG 导出。**手感不达标则触发备选方案评审（Excalidraw / 自研），不要带病进主线**
- [ ] 定义画布窄接口：`load(json) / save(): json / exportPNG()`，Plait 只是接口后的实现
- [ ] Tiptap 自定义节点 + NodeView：文档内插入画布，行内只读预览 + 全屏编辑两种模式
- [ ] 存储策略：画布场景 JSON 始终内联在文档节点属性里（单文件、单路径）；画布内插入的图片一律存 `assets/`（内容寻址），场景里只存引用，保证内联 JSON 是纯矢量数据
- [ ] 画布内脑图、流程图、手绘三类元素混排可用；撤销/重做与文档编辑器互不干扰

**验收**：一篇文档内嵌两个画布，混排编辑、保存、重启恢复、导出 PNG 全部正常。

### M4 · 搜索与知识库（预计 1 周）

- [x] SQLite FTS5 索引：保存/新建/删除/恢复时增量更新，打开 vault 时全量重建（rusqlite bundled）
- [x] **中文分词**：trigram tokenizer；≥3 字符走 FTS MATCH，1–2 字符（常见中文双字词）回退 LIKE 扫描；测试覆盖中英文长短词
- [x] 全局搜索面板（Cmd/Ctrl+K：标题 + 全文，命中片段高亮，键盘导航）
- [ ] 标签（文档属性面板）与标签筛选
- [ ] 双链：`[[` 引用其他文档、反向链接面板（画布内文本暂不参与索引，记为已知限制）

**验收**：1000 篇文档规模下搜索响应 < 100ms；删除 `index.db` 后重启自动重建且结果一致。

### M5 · 静态加密（预计 1.5 周）

- [ ] 密钥体系：主密码 → Argon2id → 主密钥；每文档随机密钥，XChaCha20-Poly1305 信封加密
- [ ] 恢复码：创建加密 vault 时生成一次性恢复码（第二把包裹主密钥的钥匙），强制用户确认已保存
- [ ] Vault 级加密开关：仅在创建时选择（明文 vault 可整体迁移为加密 vault，反向导出为明文属于"导出"功能）
- [ ] 锁定/解锁 UX：启动解锁、手动锁定、闲置自动锁定；锁定时内存密钥清零（zeroize）
- [ ] 加密文件格式定稿写入 `docs/FORMAT.md`（含 formatVersion、KDF 参数、nonce 布局）
- [ ] 威胁模型文档：明确防什么（设备失窃、云端窥探）不防什么（解锁状态下的内存取证、键盘记录）

**验收**：加密 vault 的所有文件用 `xxd` 检视无明文泄漏（含文件名）；错误密码有明确提示；改主密码不重写文档本体（只重包裹密钥）。

### M6 · 同步（预计 2 周）

- [ ] `SyncBackend` trait 定稿：`list / get / put(带 base_version) / delete`，版本检查防覆盖
- [ ] 同步引擎：加密 manifest（文档版本向量表）、增量推拉、删除传播
- [ ] 冲突策略：同文档双端修改 → 落冲突副本文件，UI 内并列展示由用户处置
- [ ] WebDAV 后端实现（用 Nextcloud 和坚果云各测一遍，两者 WebDAV 实现差异不小）
- [ ] 端到端测试：本机两个 vault 副本模拟双设备，脚本化跑「改-同步-冲突-解决」全流程
- [ ] 同步状态 UI：上次同步时间、进行中、错误明示（网络错误可重试 vs 版本冲突需处置）

**验收**：服务端抓包/看文件只见密文与不可关联的文件名；断网中途同步可安全重入；冲突永不静默丢数据。

### M7 · 打磨与发布（预计 1.5 周）

- [ ] 导出：单篇/全库导出 Markdown（画布导出为 PNG 引用）与 HTML
- [ ] 三平台打包进 CI：macOS dmg（签名 + 公证）、Windows msi/nsis、Linux AppImage + deb
- [ ] 自动更新（Tauri updater，三平台）
- [ ] 手工冒烟：macOS 全量清单（编辑、画布、搜索、加密解锁、中文输入法）；Windows / Linux 基础清单（安装启动、基本编辑、画布可用、中文输入法可输入）
- [ ] 首次启动引导：建 vault → 是否加密 → 示例文档
- [ ] README 补齐截图与下载链接，打 v0.1.0 tag

**验收**：macOS 干净机器从下载 dmg 到写下第一篇图文混排笔记 ≤ 3 分钟、无终端操作；Windows / Linux 安装包能装、能写、能画。

## 主要风险与对冲

| 风险                                     | 对冲                                                                                  |
| ---------------------------------------- | ------------------------------------------------------------------------------------- |
| Plait 成熟度不足、文档少                 | M3 spike 前置做 go/no-go 决策；画布窄接口保证可整体替换；MIT 可 fork                  |
| FTS5 中文检索质量                        | trigram 起步 + 基准用例集，预留 jieba/tantivy 升级路径                                |
| 加密格式返工代价高                       | M5 前先写格式文档评审再写代码；所有格式带版本号                                       |
| WebDAV 服务端行为不一致                  | 至少两家真实服务端做集成测试；`put` 的乐观锁语义降级方案（ETag 缺失时退回整库锁文件） |
| 单人项目范围蔓延                         | 非目标清单 + 每里程碑验收标准；新想法一律进 backlog 不插队                            |
| Linux WebView（webkit2gtk）渲染/IME 怪癖 | CI 三平台构建从 M0 开始；M3 画布 spike 与 M7 冒烟均覆盖 Linux                         |

## 已决策

1. License：MIT
2. UI 组件：shadcn/ui + Tailwind
3. 平台：macOS 一等公民，Windows / Linux 保证可用
4. 画布存储：始终内联文档 JSON，不做 sidecar；画布内图片走 `assets/` 引用（若实测出现性能问题再引入 sidecar）

## 待决策

暂无。
