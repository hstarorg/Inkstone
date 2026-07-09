import { useEffect, useState } from "react";
import { useEditorState, type Editor } from "@tiptap/react";
import { ChevronDown, ChevronUp, X } from "lucide-react";
import { searchPluginKey } from "./search";

export function SearchPanel({
  editor,
  onClose,
}: {
  editor: Editor;
  onClose: () => void;
}) {
  const [term, setTerm] = useState("");
  const [replacement, setReplacement] = useState("");

  const { count, active } = useEditorState({
    editor,
    selector: ({ editor }) => {
      const pluginState = searchPluginKey.getState(editor.state);
      return {
        count: pluginState?.matches.length ?? 0,
        active: pluginState?.activeIndex ?? 0,
      };
    },
  });

  useEffect(() => {
    editor.commands.setSearchTerm(term);
  }, [editor, term]);

  useEffect(() => {
    return () => {
      editor.commands.clearSearch();
    };
  }, [editor]);

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Enter") {
      event.preventDefault();
      if (event.shiftKey) {
        editor.commands.findPrevious();
      } else {
        editor.commands.findNext();
      }
    }
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
  };

  return (
    <div
      className="absolute top-2 right-4 z-40 flex items-center gap-1 rounded-lg border bg-popover p-1.5 text-popover-foreground shadow-md"
      onKeyDown={handleKeyDown}
    >
      <div className="flex flex-col gap-1">
        <input
          autoFocus
          value={term}
          onChange={(event) => setTerm(event.target.value)}
          placeholder="Find"
          className="h-7 w-44 rounded-md border bg-transparent px-2 text-sm focus:outline-none"
        />
        <input
          value={replacement}
          onChange={(event) => setReplacement(event.target.value)}
          placeholder="Replace"
          className="h-7 w-44 rounded-md border bg-transparent px-2 text-sm focus:outline-none"
        />
      </div>
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-1">
          <span className="w-12 text-center text-xs tabular-nums text-muted-foreground">
            {count > 0 ? `${active + 1}/${count}` : "0/0"}
          </span>
          <button
            type="button"
            aria-label="Previous match"
            onClick={() => editor.commands.findPrevious()}
            className="rounded-md p-1 hover:bg-accent hover:text-accent-foreground"
          >
            <ChevronUp className="size-4" />
          </button>
          <button
            type="button"
            aria-label="Next match"
            onClick={() => editor.commands.findNext()}
            className="rounded-md p-1 hover:bg-accent hover:text-accent-foreground"
          >
            <ChevronDown className="size-4" />
          </button>
          <button
            type="button"
            aria-label="Close search"
            onClick={onClose}
            className="rounded-md p-1 hover:bg-accent hover:text-accent-foreground"
          >
            <X className="size-4" />
          </button>
        </div>
        <div className="flex items-center gap-1">
          <button
            type="button"
            onClick={() => editor.commands.replaceActive(replacement)}
            className="rounded-md px-2 py-0.5 text-xs hover:bg-accent hover:text-accent-foreground"
          >
            Replace
          </button>
          <button
            type="button"
            onClick={() => editor.commands.replaceAll(replacement)}
            className="rounded-md px-2 py-0.5 text-xs hover:bg-accent hover:text-accent-foreground"
          >
            Replace all
          </button>
        </div>
      </div>
    </div>
  );
}
