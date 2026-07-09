import { useEffect, useState } from "react";
import { FileText, Trash2, Undo2 } from "lucide-react";
import { vaultApi, type DocMeta } from "@/lib/vault";

export function TrashPanel({
  vault,
  onRestored,
  onClose,
}: {
  vault: string;
  onRestored: () => void;
  onClose: () => void;
}) {
  const [docs, setDocs] = useState<DocMeta[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = () => {
    vaultApi
      .listTrash(vault)
      .then((list) => setDocs(list.docs))
      .catch((listError) => setError(String(listError)));
  };

  useEffect(refresh, [vault]);

  const restore = async (id: string) => {
    setError(null);
    try {
      await vaultApi.restoreDoc(vault, id);
      setDocs((prev) => prev.filter((doc) => doc.id !== id));
      onRestored();
    } catch (restoreError) {
      setError(String(restoreError));
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/30 pt-[15vh]"
      onMouseDown={onClose}
      onKeyDown={(event) => {
        if (event.key === "Escape") onClose();
      }}
    >
      <div
        className="w-full max-w-lg overflow-hidden rounded-xl border bg-popover text-popover-foreground shadow-2xl"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="flex items-center gap-2 border-b px-4 py-3">
          <Trash2 className="size-4 text-muted-foreground" />
          <h2 className="flex-1 text-sm font-medium">Trash</h2>
          <button
            type="button"
            onClick={onClose}
            className="text-xs text-muted-foreground hover:text-foreground"
          >
            Close
          </button>
        </div>
        <div className="max-h-80 overflow-y-auto p-1">
          {docs.length === 0 ? (
            <p className="px-3 py-6 text-center text-sm text-muted-foreground">
              Trash is empty
            </p>
          ) : (
            docs.map((doc) => (
              <div
                key={doc.id}
                className="flex items-center gap-2 rounded-md px-3 py-2"
              >
                <FileText className="size-4 shrink-0 text-muted-foreground" />
                <span className="min-w-0 flex-1 truncate text-sm">
                  {doc.title}
                </span>
                <button
                  type="button"
                  aria-label={`Restore ${doc.title}`}
                  onClick={() => void restore(doc.id)}
                  className="flex items-center gap-1 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                >
                  <Undo2 className="size-3.5" /> Restore
                </button>
              </div>
            ))
          )}
        </div>
        {error && (
          <p className="border-t px-3 py-2 text-xs text-destructive">{error}</p>
        )}
      </div>
    </div>
  );
}
