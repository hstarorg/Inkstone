import { invoke } from "@tauri-apps/api/core";
import type { JSONContent } from "@/components/richer-editor";

export interface VaultInfo {
  path: string;
  vaultId: string;
  formatVersion: number;
  encryption: string;
}

export interface VaultCreateResult {
  info: VaultInfo;
  recoveryCode: string | null;
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

export interface SearchHit {
  id: string;
  title: string;
  snippet: string;
}

export const vaultApi = {
  create: (path: string, password?: string) =>
    invoke<VaultCreateResult>("vault_create", { path, password }),
  open: (path: string) => invoke<VaultInfo>("vault_open", { path }),
  unlock: (path: string, password: string) =>
    invoke<VaultInfo>("vault_unlock", { path, password }),
  unlockWithRecoveryCode: (path: string, recoveryCode: string) =>
    invoke<VaultInfo>("vault_unlock_with_recovery_code", {
      path,
      recoveryCode,
    }),
  lock: (path: string) => invoke<void>("vault_lock", { path }),
  isUnlocked: (path: string) => invoke<boolean>("vault_is_unlocked", { path }),
  changePassword: (
    path: string,
    currentPassword: string,
    newPassword: string,
  ) =>
    invoke<void>("vault_change_password", {
      path,
      currentPassword,
      newPassword,
    }),
  recent: () => invoke<string | null>("vault_recent"),
  listDocs: (vault: string) => invoke<DocList>("doc_list", { vault }),
  listTrash: (vault: string) => invoke<DocList>("doc_list_trash", { vault }),
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
  searchDocs: (vault: string, query: string) =>
    invoke<SearchHit[]>("search_docs", { vault, query }),
};
