import { invoke } from "@tauri-apps/api/core";
import type { JSONContent } from "@/components/richer-editor";

export interface VaultInfo {
  path: string;
  vaultId: string;
  formatVersion: number;
  encryption: string;
}

export interface DocMeta {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
}

export interface DocList {
  docs: DocMeta[];
  warnings: string[];
}

export interface DocFile {
  id: string;
  createdAt: string;
  updatedAt: string;
  tags: string[];
  content: JSONContent;
}

export const vaultApi = {
  create: (path: string) => invoke<VaultInfo>("vault_create", { path }),
  open: (path: string) => invoke<VaultInfo>("vault_open", { path }),
  recent: () => invoke<string | null>("vault_recent"),
  listDocs: (vault: string) => invoke<DocList>("doc_list", { vault }),
  createDoc: (vault: string) => invoke<DocFile>("doc_create", { vault }),
  readDoc: (vault: string, id: string) =>
    invoke<DocFile>("doc_read", { vault, id }),
  writeDoc: (vault: string, id: string, content: JSONContent) =>
    invoke<DocMeta>("doc_write", { vault, id, content }),
  trashDoc: (vault: string, id: string) =>
    invoke<void>("doc_trash", { vault, id }),
  restoreDoc: (vault: string, id: string) =>
    invoke<void>("doc_restore", { vault, id }),
  saveAsset: (vault: string, dataBase64: string, ext: string) =>
    invoke<string>("asset_save", { vault, data: dataBase64, ext }),
};
