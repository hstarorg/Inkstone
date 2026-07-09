# Inkstone 磁盘格式规范

> 状态：文档部分 **已定稿**（M1）；加密部分 **已定稿**（M5）；同步部分 M6 前定稿。
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

## 加密格式 v1（M5，已定稿）

> 三个决策点已确认：索引用 SQLCipher 加密、assets 保留内容寻址（accept 文件名侧信道的窄残余风险）、Argon2id 参数 `memory=64MiB/iterations=3/parallelism=4`。可以开始实现。

### 密钥体系（三层信封）

```
主密码 ──Argon2id──→ KEK_password ─┐
                                    ├─ 各自包裹 → 主密钥 MK（32 随机字节，仅生成一次）
恢复码 ──HKDF-SHA256──→ KEK_recovery ┘
                                    │
MK ── 包裹 → 每文档独立随机 DEK（32 字节）── 加密 → 文档内容
MK ── 派生（HKDF，独立 info 标签）→ 索引加密密钥（用于 SQLCipher，见下）
```

- **MK（Master Key）**：vault 创建时用 CSPRNG 生成一次，32 字节，此后永不改变（除非用户主动"重置加密"，属未来功能）。
- **密码路径**：`KEK_password = Argon2id(password, salt_password)`，参数：`memory=65536 KiB (64 MiB)`、`iterations=3`、`parallelism=4`、输出 32 字节。这组参数存在 `inkstone.json` 里（不是硬编码），未来新建 vault 可调整默认值而不需要格式版本升级。
- **恢复码路径**：`KEK_recovery = HKDF-SHA256(ikm=recovery_code_bytes, salt=salt_recovery, info="inkstone-recovery-kek-v1")`。恢复码本身是 20 字节 CSPRNG 随机数（160 位熵），无需慢哈希——它不是需要防暴力破解的低熵密码，HKDF 只是把它规范映射成密钥。展示给用户时编码为 Crockford Base32（无歧义字符 I/O/L/U），分组显示为 `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`（32 字符）。
- **每文档 DEK**：创建文档时 CSPRNG 生成一次，32 字节；用 MK 包裹（wrap）后存在该文档自己的文件头里（见下）。改主密码只需重新用新 `KEK_password` 包裹 MK，不touchDEK、不碰任何文档正文——这是选择三层而非两层信封的原因。
- **AEAD**：全程 XChaCha20-Poly1305（RustCrypto `chacha20poly1305` crate），24 字节随机 nonce，每次加密独立生成，不复用。
- **RNG**：`getrandom` crate（直接读取操作系统 CSPRNG），用于所有 salt/nonce/MK/DEK/恢复码/文件名 token 的生成。未用 `rand` crate——其 `rand_core` 版本与 RustCrypto 系依赖（argon2/chacha20poly1305/hkdf）当前解析到的版本冲突，`getrandom` 更底层、无此问题。

### `inkstone.json`（加密 vault）

```jsonc
{
  "formatVersion": 1,
  "vaultId": "<ulid>",
  "createdAt": "<ISO-8601>",
  "encryption": "v1",
  "keys": {
    "formatVersion": 1,
    "password": {
      "kdf": "argon2id",
      "kdfParams": { "memoryKib": 65536, "iterations": 3, "parallelism": 4 },
      "salt": "<base64, 16 bytes>",
      "wrappedKey": { "nonce": "<base64, 24 bytes>", "ciphertext": "<base64>" },
    },
    "recovery": {
      "kdf": "hkdf-sha256",
      "salt": "<base64, 16 bytes>",
      "wrappedKey": { "nonce": "<base64, 24 bytes>", "ciphertext": "<base64>" },
    },
  },
}
```

盐、KDF 参数、包裹后的密文本身都不敏感，明文存放不泄露信息（这正是信封加密的设计目的）。

### 加密文档文件 `docs/<token>.enc`

- **文件名 = 文档 `id`**：与明文 vault 的不变式保持一致（id 必须等于文件名去扩展名），只是**加密 vault 的 `id` 生成方式换成 16 字节随机 token 的十六进制（32 位小写字符），不用 ULID**。
  - 原因：ULID 前 48 位是时间戳，若加密 vault 仍用 ULID 做 id/文件名，即使内容加密，文件名本身也会泄露"这篇文档创建于何时"——这是明确要堵住的元数据泄露。换成随机 token 后，id 和文件名同时失去时间信息，且不需要额外维护一张"id → 文件名"映射表。
  - 明文 vault 的 id 生成方式不变（仍是 ULID）。
- **二进制布局**（固定头 + 密文尾）：

```
偏移       长度      内容
0          4 字节    magic "INKD"（ASCII）
4          1 字节    formatVersion（当前 = 1）
5          24 字节   DEK 包裹 nonce
29         48 字节   DEK 包裹密文（32 字节 DEK + 16 字节 Poly1305 tag），用 MK 加密
77         24 字节   正文 nonce
101        剩余      正文密文（JSON 序列化的文档，字段与明文文档 schema 完全一致）+ 16 字节 tag
```

- 解密后的 JSON 内容与明文 vault 的 `docs/<ulid>.json` **schema 完全相同**（id/formatVersion/createdAt/updatedAt/tags/content），只是整体被当作一段字节串加密，不做字段级加密。
- 损坏处理规则不变：解不开（tag 校验失败）或反序列化失败 → 文件原样保留，UI 报告该文档损坏，不影响其他文档。

### `.trash/` 与 assets（加密 vault）

- `.trash/` 内文件同样是 `<token>.enc`，原样移动，不解密不重加密。
- assets：**沿用现有内容寻址方案**（`assets/<sha256前16字节hex>.<ext>.enc`，hash 基于明文字节），文件内容套用与文档相同的头部布局（DEK 包裹 + 正文密文，只是"正文"是原始二进制而非 JSON）。这里有一个已知、刻意接受的取舍——见下方决策点 2。

### 索引 `.inkstone/index.db`（加密 vault）

- `index.db` 统一用 `rusqlite` 的 `bundled-sqlcipher-vendored-openssl` feature 编译（OpenSSL 从源码内置编译，不依赖系统安装，Windows CI 亦可构建）。SQLCipher 编译出的 SQLite 对明文 DB 完全透明兼容——**不是编译期二选一**，而是运行时决定：加密 vault 才在打开连接后执行 `PRAGMA key`，明文 vault 不执行，链接的是同一份二进制。
- 页加密密钥 = `HKDF-SHA256(ikm=MK, salt=vaultId, info="inkstone-index-key-v1")`，派生 32 字节，vault 解锁后传给 SQLCipher 的 `PRAGMA key`。
- 明文 vault 的 `index.db` 不变（仍是普通明文 SQLite，`bundled` feature）。
- 加密 vault 未解锁时，`index.db` 完全不可用（无法列出文档、无法搜索）——这与"文档本体也需要 MK 才能读"一致，不构成新的功能倒退。
- 索引本质仍是可丢弃的派生数据：删除 `index.db` 后，vault 解锁状态下可全量重建（重建时逐个解密文档取标题/正文再入库，与明文 vault 重建逻辑相同，只是库文件本身加了密）。

### 锁定与内存

- MK 只存在于 Rust 侧内存，解锁时派生/解包，`zeroize` 在锁定或应用退出时清零。
- 每文档 DEK **不做批量预解包缓存**；每次读写具体某篇文档时才临时解包对应 DEK，用完即弃——保持"锁定"语义干净：清掉 MK 后，之前解包过的 DEK 不会遗留在内存里被继续使用。
- 解锁失败（密码错/恢复码错）：AEAD tag 校验失败即报错，不做模糊提示，不做次数限制（本地应用，无网络暴力破解面）。

### 测试向量

不在本文档内手算字节序列；随 Rust 实现在单元测试里固化（构造一个已知 MK/DEK/明文，验证输出的密文/tag 可复现），作为 CI 回归防线。

## 同步 manifest（占位，M6 前定稿）

待定稿内容：加密 manifest 结构（文档版本表）、版本号语义与冲突判定、删除墓碑（tombstone）、远端对象命名。
