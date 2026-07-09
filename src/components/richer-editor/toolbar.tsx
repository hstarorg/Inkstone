import type { Editor } from "@tiptap/react";
import { useEditorState } from "@tiptap/react";
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  Ban,
  Redo2,
  Undo2,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { TEXT_COLORS } from "./colors";

const HEADING_OPTIONS = [
  { value: "paragraph", label: "Text" },
  { value: "h1", label: "Heading 1" },
  { value: "h2", label: "Heading 2" },
  { value: "h3", label: "Heading 3" },
];

function currentHeadingValue(editor: Editor): string {
  if (editor.isActive("heading", { level: 1 })) return "h1";
  if (editor.isActive("heading", { level: 2 })) return "h2";
  if (editor.isActive("heading", { level: 3 })) return "h3";
  return "paragraph";
}

export function RicherEditorToolbar({ editor }: { editor: Editor }) {
  const state = useEditorState({
    editor,
    selector: ({ editor }) => ({
      heading: currentHeadingValue(editor),
      canUndo: editor.can().undo(),
      canRedo: editor.can().redo(),
      alignLeft: editor.isActive({ textAlign: "left" }),
      alignCenter: editor.isActive({ textAlign: "center" }),
      alignRight: editor.isActive({ textAlign: "right" }),
    }),
  });

  const setHeading = (value: string) => {
    const chain = editor.chain().focus();
    if (value === "h1" || value === "h2" || value === "h3") {
      chain.setHeading({ level: Number(value[1]) as 1 | 2 | 3 }).run();
    } else {
      chain.setParagraph().run();
    }
  };

  return (
    <div className="flex items-center gap-1 border-b px-2 py-1">
      <button
        type="button"
        aria-label="Undo"
        disabled={!state.canUndo}
        onClick={() => editor.chain().focus().undo().run()}
        className="rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-30"
      >
        <Undo2 className="size-4" />
      </button>
      <button
        type="button"
        aria-label="Redo"
        disabled={!state.canRedo}
        onClick={() => editor.chain().focus().redo().run()}
        className="rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-30"
      >
        <Redo2 className="size-4" />
      </button>
      <div className="mx-1 h-5 w-px bg-border" />
      <select
        aria-label="Text style"
        value={state.heading}
        onChange={(event) => setHeading(event.target.value)}
        className="rounded-md border bg-transparent px-1.5 py-1 text-sm"
      >
        {HEADING_OPTIONS.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      <div className="mx-1 h-5 w-px bg-border" />
      <button
        type="button"
        aria-label="Align left"
        aria-pressed={state.alignLeft}
        onClick={() => editor.chain().focus().setTextAlign("left").run()}
        className={cn(
          "rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground",
          state.alignLeft && "bg-accent text-accent-foreground",
        )}
      >
        <AlignLeft className="size-4" />
      </button>
      <button
        type="button"
        aria-label="Align center"
        aria-pressed={state.alignCenter}
        onClick={() => editor.chain().focus().setTextAlign("center").run()}
        className={cn(
          "rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground",
          state.alignCenter && "bg-accent text-accent-foreground",
        )}
      >
        <AlignCenter className="size-4" />
      </button>
      <button
        type="button"
        aria-label="Align right"
        aria-pressed={state.alignRight}
        onClick={() => editor.chain().focus().setTextAlign("right").run()}
        className={cn(
          "rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground",
          state.alignRight && "bg-accent text-accent-foreground",
        )}
      >
        <AlignRight className="size-4" />
      </button>
      <div className="mx-1 h-5 w-px bg-border" />
      {TEXT_COLORS.map((color) => (
        <button
          key={color.value}
          type="button"
          aria-label={`Text color ${color.name}`}
          onClick={() => editor.chain().focus().setColor(color.value).run()}
          className="size-5 rounded-full border border-black/10"
          style={{ backgroundColor: color.value }}
        />
      ))}
      <button
        type="button"
        aria-label="Remove text color"
        onClick={() => editor.chain().focus().unsetColor().run()}
        className="rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground"
      >
        <Ban className="size-4" />
      </button>
    </div>
  );
}
