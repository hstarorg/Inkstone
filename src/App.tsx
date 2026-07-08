import { useCallback, useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { FolderOpen, FolderPlus, Plus, Trash2 } from "lucide-react";
import { RicherEditor, type JSONContent } from "@/components/richer-editor";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { vaultApi, type DocMeta } from "@/lib/vault";

interface ActiveDoc {
  id: string;
  content: JSONContent;
}

function collectText(node?: JSONContent): string {
  if (!node) return "";
  if (node.type === "text") return node.text ?? "";
  return (node.content ?? []).map(collectText).join("");
}

function deriveTitle(content: JSONContent): string {
  return collectText(content.content?.[0]).trim() || "Untitled";
}

function vaultName(path: string): string {
  return path.split(/[/\\]/).filter(Boolean).pop() ?? path;
}

const EXT_BY_MIME: Record<string, string> = {
  "image/png": "png",
  "image/jpeg": "jpg",
  "image/gif": "gif",
  "image/webp": "webp",
  "image/svg+xml": "svg",
  "image/avif": "avif",
};

function toBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
}

const ASSET_PREFIX = "asset://";

function App() {
  const [vault, setVault] = useState<string | null>(null);
  const [docs, setDocs] = useState<DocMeta[]>([]);
  const [activeDoc, setActiveDoc] = useState<ActiveDoc | null>(null);
  const [error, setError] = useState<string | null>(null);

  const pendingWrite = useRef<{
    timer: ReturnType<typeof setTimeout>;
    id: string;
    content: JSONContent;
  } | null>(null);

  const flushPendingWrite = useCallback(async () => {
    const pending = pendingWrite.current;
    if (!pending || !vault) return;
    clearTimeout(pending.timer);
    pendingWrite.current = null;
    try {
      const meta = await vaultApi.writeDoc(vault, pending.id, pending.content);
      setDocs((prev) => prev.map((doc) => (doc.id === meta.id ? meta : doc)));
    } catch (writeError) {
      setError(String(writeError));
    }
  }, [vault]);

  const loadVault = useCallback(async (path: string) => {
    const info = await vaultApi.open(path);
    const list = await vaultApi.listDocs(info.path);
    setVault(info.path);
    setDocs(list.docs);
    setActiveDoc(null);
    setError(list.warnings[0] ?? null);
  }, []);

  useEffect(() => {
    vaultApi
      .recent()
      .then((recent) => (recent ? loadVault(recent) : undefined))
      .catch(() => {});
  }, [loadVault]);

  const pickVault = async (mode: "open" | "create") => {
    setError(null);
    try {
      const path = await openDialog({ directory: true });
      if (typeof path !== "string") return;
      if (mode === "create") {
        await vaultApi.create(path);
      }
      await loadVault(path);
    } catch (pickError) {
      setError(String(pickError));
    }
  };

  const selectDoc = async (id: string) => {
    if (activeDoc?.id === id) return;
    await flushPendingWrite();
    if (!vault) return;
    try {
      const doc = await vaultApi.readDoc(vault, id);
      setActiveDoc({ id: doc.id, content: doc.content });
    } catch (readError) {
      setError(String(readError));
    }
  };

  const createDoc = async () => {
    await flushPendingWrite();
    if (!vault) return;
    try {
      const doc = await vaultApi.createDoc(vault);
      setDocs((prev) => [
        {
          id: doc.id,
          title: "Untitled",
          createdAt: doc.createdAt,
          updatedAt: doc.updatedAt,
        },
        ...prev,
      ]);
      setActiveDoc({ id: doc.id, content: doc.content });
    } catch (createError) {
      setError(String(createError));
    }
  };

  const trashDoc = async (id: string) => {
    if (!vault) return;
    if (pendingWrite.current?.id === id) {
      clearTimeout(pendingWrite.current.timer);
      pendingWrite.current = null;
    }
    try {
      await vaultApi.trashDoc(vault, id);
      setDocs((prev) => prev.filter((doc) => doc.id !== id));
      if (activeDoc?.id === id) setActiveDoc(null);
    } catch (trashError) {
      setError(String(trashError));
    }
  };

  const uploadImage = useCallback(
    async (file: File) => {
      if (!vault) throw new Error("no vault open");
      const ext = EXT_BY_MIME[file.type] ?? file.name.split(".").pop() ?? "bin";
      const name = await vaultApi.saveAsset(
        vault,
        toBase64(await file.arrayBuffer()),
        ext,
      );
      return `${ASSET_PREFIX}${name}`;
    },
    [vault],
  );

  const resolveAssetSrc = useCallback(
    (src: string) => {
      if (!vault || !src.startsWith(ASSET_PREFIX)) return src;
      return convertFileSrc(
        `${vault}/assets/${src.slice(ASSET_PREFIX.length)}`,
      );
    },
    [vault],
  );

  const handleChange = (id: string, content: JSONContent) => {
    setDocs((prev) =>
      prev.map((doc) =>
        doc.id === id ? { ...doc, title: deriveTitle(content) } : doc,
      ),
    );
    if (pendingWrite.current) clearTimeout(pendingWrite.current.timer);
    pendingWrite.current = {
      id,
      content,
      timer: setTimeout(() => {
        void flushPendingWrite();
      }, 500),
    };
  };

  if (!vault) {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-4 bg-background text-foreground">
        <img src="/logo.svg" alt="Inkstone" className="size-20 rounded-2xl" />
        <h1 className="text-lg font-semibold">Inkstone</h1>
        <p className="max-w-sm text-center text-sm text-muted-foreground">
          A vault is a folder on your disk that holds your documents. Open an
          existing one or create a new one.
        </p>
        <div className="flex gap-2">
          <Button onClick={() => pickVault("open")}>
            <FolderOpen /> Open vault
          </Button>
          <Button variant="secondary" onClick={() => pickVault("create")}>
            <FolderPlus /> New vault
          </Button>
        </div>
        {error && (
          <p className="max-w-md text-center text-sm text-destructive">
            {error}
          </p>
        )}
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-background text-foreground">
      <aside className="flex w-60 shrink-0 flex-col border-r bg-sidebar">
        <div className="flex items-center gap-2 px-4 py-3">
          <img src="/logo.svg" alt="" className="size-6 rounded" />
          <span className="flex-1 truncate text-sm font-semibold">
            {vaultName(vault)}
          </span>
          <Button
            variant="ghost"
            size="icon"
            aria-label="New document"
            onClick={createDoc}
          >
            <Plus />
          </Button>
        </div>
        <nav className="flex-1 overflow-y-auto px-2 py-1">
          {docs.length === 0 ? (
            <p className="px-2 py-8 text-center text-xs text-muted-foreground">
              No documents yet
            </p>
          ) : (
            docs.map((doc) => (
              <div
                key={doc.id}
                className={cn(
                  "group flex items-center rounded-md",
                  doc.id === activeDoc?.id
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-sidebar-foreground hover:bg-sidebar-accent/50",
                )}
              >
                <button
                  onClick={() => void selectDoc(doc.id)}
                  className="min-w-0 flex-1 truncate px-2 py-1.5 text-left text-sm"
                >
                  {doc.title}
                </button>
                <button
                  aria-label="Delete document"
                  onClick={() => void trashDoc(doc.id)}
                  className="mr-1 hidden rounded p-1 text-muted-foreground hover:text-destructive group-hover:block"
                >
                  <Trash2 className="size-3.5" />
                </button>
              </div>
            ))
          )}
        </nav>
        {error && (
          <p className="border-t px-3 py-2 text-xs text-destructive">{error}</p>
        )}
      </aside>
      <main className="min-w-0 flex-1">
        {activeDoc ? (
          <RicherEditor
            key={activeDoc.id}
            defaultValue={activeDoc.content}
            autofocus
            onChange={(content) => handleChange(activeDoc.id, content)}
            uploadImage={uploadImage}
            resolveAssetSrc={resolveAssetSrc}
          />
        ) : (
          <div className="flex h-full flex-col items-center justify-center gap-4">
            <img
              src="/logo.svg"
              alt="Inkstone"
              className="size-16 rounded-2xl"
            />
            <p className="text-sm text-muted-foreground">
              Select a document from the sidebar, or start here
            </p>
            <Button onClick={createDoc}>New document</Button>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
