# Inkstone 磁盘格式规范

> 状态：**草案**。文档部分随 M1 定稿；加密部分 M5 前评审定稿；同步部分 M6 前定稿。
> 本文件是所有落盘格式的唯一权威。任何格式改动必须先修改本文件，再改代码。

## 总则

1. **版本化**：所有落盘 JSON 顶层必须带整数字段 `formatVersion`，从 `1` 开始。
2. **兼容策略**：读到旧版本 → 兼容读取或执行迁移；读到高于当前支持的版本 → 只读打开并提示升级应用，禁止写入。禁止静默丢弃未知字段（读-改-写必须往返保真）。
3. **原子写**：写盘一律"同目录临时文件写入 → flush → 原子 rename"。任何时刻磁盘上不得出现半个文件。
4. **编码**：UTF-8，无 BOM；JSON 不做格式化压缩要求，但键名用 camelCase。

## Vault 目录布局

```
MyVault/
├── inkstone.json          # vault 元信息
├── docs/
│   └── <ulid>.json        # 每文档一个文件，画布场景内联其中
├── assets/
│   └── <hash>.<ext>       # 二进制资源，内容寻址
├── .trash/                # 软删除的文档，保留原文件名
└── .inkstone/
    └── index.db           # SQLite 派生索引，可随时删除重建，不参与同步
```

## inkstone.json

```jsonc
{
  "formatVersion": 1,
  "vaultId": "<ulid>",        // vault 唯一标识，创建时生成
  "createdAt": "<ISO-8601>",
  "encryption": "none"        // "none" | "v1"（v1 布局见加密章节，M5 定稿）
}
```

## 文档文件 `docs/<ulid>.json`

```jsonc
{
  "formatVersion": 1,
  "id": "<ulid>",             // 与文件名一致
  "createdAt": "<ISO-8601>",
  "updatedAt": "<ISO-8601>",
  "tags": ["..."],
  "content": { /* ProseMirror doc JSON（Tiptap 原生格式） */ }
}
```

- 文档标题不设独立字段，由 `content` 派生（首个 heading，缺省取首行文本）。
- 画布是 `content` 中的自定义节点：`type: "canvas"`，`attrs.scene` 为 Plait 场景 JSON。画布内图片不得内嵌 base64，必须走 assets 引用。

## assets 资源

- 文件名：内容 SHA-256 的前 16 字节 hex + 原扩展名，天然去重。
- 文档/画布中的引用格式：`asset://<hash>`，由渲染层解析为本地路径。
- 垃圾回收：删除文档不立即删 asset；提供全库扫描回收未引用资源的维护操作（待设计）。

## 加密格式 v1（占位，M5 前评审定稿）

待定稿内容：Argon2id 参数选择、主密钥/恢复码双包裹结构、每文档 DEK 信封布局、XChaCha20-Poly1305 nonce 布局、加密文件头（magic + formatVersion + KDF 参数）、文件名混淆方案、测试向量。

**定稿前禁止实现加密落盘代码。**

## 同步 manifest（占位，M6 前定稿）

待定稿内容：加密 manifest 结构（文档版本表）、版本号语义与冲突判定、删除墓碑（tombstone）、远端对象命名。
