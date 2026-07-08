import { Fragment, useEffect, useRef, useState } from "react";
import { FileText, SearchIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { vaultApi, type SearchHit } from "@/lib/vault";

const HIGHLIGHT_OPEN = "";
const HIGHLIGHT_CLOSE = "";

function Snippet({ text }: { text: string }) {
  const parts = text.split(HIGHLIGHT_OPEN);
  return (
    <>
      {parts.map((part, index) => {
        if (index === 0) return <Fragment key={index}>{part}</Fragment>;
        const [hit, rest] = part.split(HIGHLIGHT_CLOSE);
        return (
          <Fragment key={index}>
            <mark className="rounded-sm bg-transparent font-semibold text-foreground">
              {hit}
            </mark>
            {rest}
          </Fragment>
        );
      })}
    </>
  );
}

export function SearchPalette({
  vault,
  onOpen,
  onClose,
}: {
  vault: string;
  onOpen: (id: string) => void;
  onClose: () => void;
}) {
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[]>([]);
  const [selected, setSelected] = useState(0);
  const debounce = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (debounce.current) clearTimeout(debounce.current);
    debounce.current = setTimeout(() => {
      vaultApi
        .searchDocs(vault, query)
        .then((results) => {
          setHits(results);
          setSelected(0);
        })
        .catch(() => setHits([]));
    }, 150);
    return () => {
      if (debounce.current) clearTimeout(debounce.current);
    };
  }, [vault, query]);

  const openHit = (hit: SearchHit | undefined) => {
    if (!hit) return;
    onOpen(hit.id);
    onClose();
  };

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setSelected((current) => Math.min(current + 1, hits.length - 1));
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setSelected((current) => Math.max(current - 1, 0));
    }
    if (event.key === "Enter") {
      event.preventDefault();
      openHit(hits[selected]);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/30 pt-[15vh]"
      onMouseDown={onClose}
    >
      <div
        className="w-full max-w-lg overflow-hidden rounded-xl border bg-popover text-popover-foreground shadow-2xl"
        onMouseDown={(event) => event.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        <div className="flex items-center gap-2 border-b px-3">
          <SearchIcon className="size-4 text-muted-foreground" />
          <input
            autoFocus
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search documents…"
            className="h-11 flex-1 bg-transparent text-sm focus:outline-none"
          />
        </div>
        <div className="max-h-80 overflow-y-auto p-1">
          {hits.length === 0 ? (
            <p className="px-3 py-6 text-center text-sm text-muted-foreground">
              {query.trim() ? "No results" : "Type to search your vault"}
            </p>
          ) : (
            hits.map((hit, index) => (
              <button
                key={hit.id}
                type="button"
                onClick={() => openHit(hit)}
                onMouseEnter={() => setSelected(index)}
                className={cn(
                  "flex w-full items-start gap-2 rounded-md px-3 py-2 text-left",
                  index === selected && "bg-accent text-accent-foreground",
                )}
              >
                <FileText className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                <span className="min-w-0">
                  <span className="block truncate text-sm font-medium">
                    {hit.title}
                  </span>
                  <span className="block truncate text-xs text-muted-foreground">
                    <Snippet text={hit.snippet} />
                  </span>
                </span>
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
