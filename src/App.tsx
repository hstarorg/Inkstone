import { useState } from "react";
import { Plus } from "lucide-react";
import { RicherEditor, type JSONContent } from "@/components/richer-editor";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface Doc {
  id: string;
  title: string;
  content?: JSONContent;
}

function collectText(node?: JSONContent): string {
  if (!node) return "";
  if (node.type === "text") return node.text ?? "";
  return (node.content ?? []).map(collectText).join("");
}

function deriveTitle(content: JSONContent): string {
  return collectText(content.content?.[0]).trim() || "Untitled";
}

function App() {
  const [docs, setDocs] = useState<Doc[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const activeDoc = docs.find((doc) => doc.id === activeId);

  const createDoc = () => {
    const doc: Doc = { id: crypto.randomUUID(), title: "Untitled" };
    setDocs((prev) => [...prev, doc]);
    setActiveId(doc.id);
  };

  const updateDoc = (id: string, content: JSONContent) => {
    setDocs((prev) =>
      prev.map((doc) =>
        doc.id === id ? { ...doc, content, title: deriveTitle(content) } : doc,
      ),
    );
  };

  return (
    <div className="flex h-screen bg-background text-foreground">
      <aside className="flex w-60 shrink-0 flex-col border-r bg-sidebar">
        <div className="flex items-center gap-2 px-4 py-3">
          <img src="/logo.svg" alt="" className="size-6 rounded" />
          <span className="flex-1 text-sm font-semibold">Inkstone</span>
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
              <button
                key={doc.id}
                onClick={() => setActiveId(doc.id)}
                className={cn(
                  "w-full truncate rounded-md px-2 py-1.5 text-left text-sm",
                  doc.id === activeId
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-sidebar-foreground hover:bg-sidebar-accent/50",
                )}
              >
                {doc.title}
              </button>
            ))
          )}
        </nav>
      </aside>
      <main className="min-w-0 flex-1">
        {activeDoc ? (
          <RicherEditor
            key={activeDoc.id}
            defaultValue={activeDoc.content}
            autofocus
            onChange={(content) => updateDoc(activeDoc.id, content)}
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
