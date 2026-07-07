# Inkstone · 砚台

> A local-first, end-to-end encrypted knowledge base where writing and drawing live on the same page.
>
> 本地优先、端到端加密的个人知识库，写作与画图在同一页面自由混排。

Inkstone is a personal knowledge base that keeps your thoughts where they belong — on your own machine. Write rich documents, then sketch freely alongside them: mind maps, flowcharts, and freehand drawings all live on one canvas, embedded right in your notes. Everything is stored as plain local files, searchable offline, and end-to-end encrypted before it ever leaves your device. Sync is optional, pluggable, and blind — whichever backend you choose only ever sees ciphertext.

砚台（Inkstone）把数据留在你自己的设备上：所有内容以本地文件存储、离线全文检索；思维导图、流程图、自由手绘混排在同一张画布上，直接嵌进笔记；在离开你的设备之前，一切都已端到端加密，同步后端全程只见密文。

## Design principles

- **Local-first** — documents are plain files on disk; the app works fully offline and forever without an account.
- **E2EE by construction** — encryption happens on-device (Argon2id key derivation, XChaCha20-Poly1305 envelope encryption); sync backends are blind relays.
- **Writing and drawing are one medium** — the canvas (mind map / flowchart / freehand) is a first-class node inside documents, not a separate app.
- **No lock-in** — one document, one file; reliable export to Markdown/HTML; the search index is disposable and rebuildable.

## Tech stack

| Layer | Choice |
|---|---|
| Desktop shell | Tauri 2 (Rust backend) |
| Editor | Tiptap v3 + React, documents stored as ProseMirror JSON files |
| Canvas | Plait (mind map + flowchart + freehand), wrapped as a Tiptap node behind a narrow interface |
| Search & metadata | SQLite (FTS5), rebuildable from source files |
| Encryption | Argon2id + XChaCha20-Poly1305 (envelope encryption, per-document keys) |
| Sync | Pluggable `SyncBackend` trait (WebDAV first; S3-compatible and self-hosted later), versioned + conflict copies |

Single-user by design: no collaborative editing, no CRDT, no sync server with plaintext access.

## Roadmap

1. Tauri + Tiptap shell: local JSON document read/write, file tree
2. Plait canvas node: mixed mind map / flowchart / freehand, inline preview + fullscreen edit
3. SQLite FTS5 full-text search, tags and backlinks
4. At-rest encryption (optional toggle)
5. `SyncBackend` interface + WebDAV implementation

## License

[MIT](./LICENSE)
