# Inkstone 磁盘格式规范

> 状态：文档部分 **M1 评审中**；加密部分 M5 前评审定稿；同步部分 M6 前定稿。
> 本文件是所有落盘格式的唯一权威。任何格式改动必须先修改本文件，再改代码。

## 总则

1. **版本化**：所有落盘 JSON 顶层必须带整数字段 `formatVersion`，从 `1` 开始。
2. **兼容策略**：读到旧版本 → 兼容读取或执行迁移；读到高于当前支持的版本 → 只读打开并提示升级应用，禁止写入。禁止静默丢弃未知字段（读-改-写必须往返保真）。
3. **原子写**：写盘一律"同目录临时文件写入 → flush(fsync) → 原子 rename"。临时文件命名 `<目标名>.tmp-<随机后缀>`；应用启动时清扫 vault 内残留的 `*.tmp-*` 孤儿文件。任何时刻磁盘上不得出现半个文件。
4. **编码**：UTF-8，无 BOM；JSON 不做格式化压缩要求，但键名用 camelCase。
5. **标识符**：文档 ID 用 ULID，落盘与文件名一律**小写**（规避跨平台文件系统大小写敏感性差异）。
6. **时间戳**：一律 RFC 3339 / ISO-8601，UTC（`2026-07-08T03:00:00.000Z`）。
7. **并发假设**：v1 假设同一 vault 同时只被一个应用实例打开（后续用单实例锁强化）；索引 `index.db` 之外的文件不做文件级锁。

## Vault 目录布局

```
MyVault/
├── inkstone.json          # vault 元信息（此文件的存在即 vault 的标志）
├── docs/
│   └── <ulid>.json        # 每文档一个文件，画布场景内联其中
├── assets/
│   └── <hash>.<ext>       # 二进制资源，内容寻址（sha256 前 16 字节 hex）
├── .trash/                # 软删除的文档，保留原文件名（见软删除章节）
└── .inkstone/
    └── index.db           # SQLite 派生索引，可随时删除重建，不参与同步

- `docs/`、`assets/`、`.trash/`、`.inkstone/` 由应用按需创建，缺失不视为错误。
- 应用级状态（最近打开的 vault 路径、窗口尺寸等）存 OS 应用配置目录，**不入 vault**。
```

## inkstone.json

```jsonc
{
  "formatVersion": 1,
  "vaultId": "<ulid>", // vault 唯一标识，创建时生成
  "createdAt": "<ISO-8601>",
  "encryption": "none", // "none" | "v1"（v1 布局见加密章节，M5 定稿）
}
```

## 文档文件 `docs/<ulid>.json`

```jsonc
{
  "formatVersion": 1,
  "id": "<ulid>", // 与文件名一致
  "createdAt": "<ISO-8601>",
  "updatedAt": "<ISO-8601>",
  "tags": ["..."],
  "content": {/* ProseMirror doc JSON（Tiptap 原生格式） */},
}
```

- `id` 必须与文件名（去扩展名）一致，不一致视为损坏文件。
- 文档标题不设独立字段，由 `content` 派生（首个 heading，缺省取首行文本，均空则 "Untitled"）。
- `content` 为 Tiptap/ProseMirror 文档 JSON 原样存储，应用不得改写其内部结构；未知节点类型原样保留（向前兼容未来的块类型）。
- 画布是 `content` 中的自定义节点：`type: "canvas"`，`attrs.scene` 为 Plait 场景 JSON。画布内图片不得内嵌 base64，必须走 assets 引用。
- 读到损坏 JSON：文件保持原样不动，UI 提示损坏并跳过加载；禁止自动"修复"覆盖原文件。

## 软删除 `.trash/`

- 删除文档 = 将 `docs/<ulid>.json` 原样移入 `.trash/<ulid>.json`，内容不做任何修改；删除时间取文件 mtime。
- 恢复 = 移回 `docs/`；ID 冲突（理论上不可能）时拒绝并报错。
- 清空回收站为显式用户操作；应用不做自动清理（自动过期策略留待后续版本）。
- `.trash/` 不参与搜索索引，不参与同步（同步的删除传播用 manifest 墓碑，见同步章节）。

## assets 资源

- 文件名：内容 SHA-256 的前 16 字节 hex + 原扩展名，天然去重。
- 文档/画布中的引用格式：`asset://<hash>`，由渲染层解析为本地路径。
- 垃圾回收：删除文档不立即删 asset；提供全库扫描回收未引用资源的维护操作（待设计）。

## 加密格式 v1（占位，M5 前评审定稿）

待定稿内容：Argon2id 参数选择、主密钥/恢复码双包裹结构、每文档 DEK 信封布局、XChaCha20-Poly1305 nonce 布局、加密文件头（magic + formatVersion + KDF 参数）、文件名混淆方案、测试向量。

**定稿前禁止实现加密落盘代码。**

## 同步 manifest（占位，M6 前定稿）

待定稿内容：加密 manifest 结构（文档版本表）、版本号语义与冲突判定、删除墓碑（tombstone）、远端对象命名。
