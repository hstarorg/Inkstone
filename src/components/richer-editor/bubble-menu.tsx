import { useEffect, useRef, useState } from "react";
import type { Editor } from "@tiptap/react";
import { useEditorState } from "@tiptap/react";
import { BubbleMenu } from "@tiptap/react/menus";
import {
  Bold,
  Check,
  Code,
  Heading1,
  Heading2,
  Heading3,
  Italic,
  Link as LinkIcon,
  List,
  ListOrdered,
  Strikethrough,
  TextQuote,
  Trash2,
  Underline,
  type LucideIcon,
} from "lucide-react";
import { cn } from "@/lib/utils";

interface EditorAction {
  key: string;
  icon: LucideIcon;
  label: string;
  isActive: (editor: Editor) => boolean;
  toggle: (editor: Editor) => void;
}

const BLOCK_ACTIONS: EditorAction[] = [
  {
    key: "h1",
    icon: Heading1,
    label: "Heading 1",
    isActive: (editor) => editor.isActive("heading", { level: 1 }),
    toggle: (editor) =>
      editor.chain().focus().toggleHeading({ level: 1 }).run(),
  },
  {
    key: "h2",
    icon: Heading2,
    label: "Heading 2",
    isActive: (editor) => editor.isActive("heading", { level: 2 }),
    toggle: (editor) =>
      editor.chain().focus().toggleHeading({ level: 2 }).run(),
  },
  {
    key: "h3",
    icon: Heading3,
    label: "Heading 3",
    isActive: (editor) => editor.isActive("heading", { level: 3 }),
    toggle: (editor) =>
      editor.chain().focus().toggleHeading({ level: 3 }).run(),
  },
  {
    key: "bulletList",
    icon: List,
    label: "Bullet list",
    isActive: (editor) => editor.isActive("bulletList"),
    toggle: (editor) => editor.chain().focus().toggleBulletList().run(),
  },
  {
    key: "orderedList",
    icon: ListOrdered,
    label: "Ordered list",
    isActive: (editor) => editor.isActive("orderedList"),
    toggle: (editor) => editor.chain().focus().toggleOrderedList().run(),
  },
  {
    key: "blockquote",
    icon: TextQuote,
    label: "Blockquote",
    isActive: (editor) => editor.isActive("blockquote"),
    toggle: (editor) => editor.chain().focus().toggleBlockquote().run(),
  },
];

const MARK_ACTIONS: EditorAction[] = [
  {
    key: "bold",
    icon: Bold,
    label: "Bold",
    isActive: (editor) => editor.isActive("bold"),
    toggle: (editor) => editor.chain().focus().toggleBold().run(),
  },
  {
    key: "italic",
    icon: Italic,
    label: "Italic",
    isActive: (editor) => editor.isActive("italic"),
    toggle: (editor) => editor.chain().focus().toggleItalic().run(),
  },
  {
    key: "underline",
    icon: Underline,
    label: "Underline",
    isActive: (editor) => editor.isActive("underline"),
    toggle: (editor) => editor.chain().focus().toggleUnderline().run(),
  },
  {
    key: "strike",
    icon: Strikethrough,
    label: "Strikethrough",
    isActive: (editor) => editor.isActive("strike"),
    toggle: (editor) => editor.chain().focus().toggleStrike().run(),
  },
  {
    key: "code",
    icon: Code,
    label: "Inline code",
    isActive: (editor) => editor.isActive("code"),
    toggle: (editor) => editor.chain().focus().toggleCode().run(),
  },
];

function ActionButton({
  action,
  active,
  editor,
}: {
  action: EditorAction;
  active: boolean;
  editor: Editor;
}) {
  return (
    <button
      type="button"
      aria-label={action.label}
      aria-pressed={active}
      onMouseDown={(event) => event.preventDefault()}
      onClick={() => action.toggle(editor)}
      className={cn(
        "rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground",
        active && "bg-accent text-accent-foreground",
      )}
    >
      <action.icon className="size-4" />
    </button>
  );
}

export function RicherEditorBubbleMenu({ editor }: { editor: Editor }) {
  const [linkOpen, setLinkOpen] = useState(false);
  const [url, setUrl] = useState("");
  const linkOpenRef = useRef(false);

  useEffect(() => {
    linkOpenRef.current = linkOpen;
  }, [linkOpen]);

  const state = useEditorState({
    editor,
    selector: ({ editor }) => ({
      blocks: BLOCK_ACTIONS.map((action) => action.isActive(editor)),
      marks: MARK_ACTIONS.map((action) => action.isActive(editor)),
      link: editor.isActive("link"),
    }),
  });

  const openLinkInput = () => {
    setUrl(editor.getAttributes("link").href ?? "");
    setLinkOpen(true);
  };

  const closeLinkInput = () => {
    setLinkOpen(false);
    editor.chain().focus().run();
  };

  const applyLink = () => {
    const href = url.trim();
    const chain = editor.chain().focus().extendMarkRange("link");
    if (href) {
      chain.setLink({ href }).run();
    } else {
      chain.unsetLink().run();
    }
    setLinkOpen(false);
  };

  const removeLink = () => {
    editor.chain().focus().extendMarkRange("link").unsetLink().run();
    setLinkOpen(false);
  };

  return (
    <BubbleMenu
      editor={editor}
      shouldShow={({ editor }) =>
        linkOpenRef.current || !editor.state.selection.empty
      }
      className="flex items-center gap-0.5 rounded-lg border bg-popover p-1 text-popover-foreground shadow-md"
    >
      {linkOpen ? (
        <div className="flex items-center gap-0.5">
          <input
            autoFocus
            value={url}
            onChange={(event) => setUrl(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                applyLink();
              }
              if (event.key === "Escape") {
                event.preventDefault();
                closeLinkInput();
              }
            }}
            placeholder="https://"
            className="h-7 w-56 bg-transparent px-2 text-sm focus:outline-none"
          />
          <button
            type="button"
            aria-label="Apply link"
            onClick={applyLink}
            className="rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground"
          >
            <Check className="size-4" />
          </button>
          <button
            type="button"
            aria-label="Remove link"
            onClick={removeLink}
            className="rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground"
          >
            <Trash2 className="size-4" />
          </button>
        </div>
      ) : (
        <>
          {BLOCK_ACTIONS.map((action, index) => (
            <ActionButton
              key={action.key}
              action={action}
              active={state.blocks[index]}
              editor={editor}
            />
          ))}
          <div className="mx-0.5 h-5 w-px bg-border" />
          {MARK_ACTIONS.map((action, index) => (
            <ActionButton
              key={action.key}
              action={action}
              active={state.marks[index]}
              editor={editor}
            />
          ))}
          <div className="mx-0.5 h-5 w-px bg-border" />
          <button
            type="button"
            aria-label="Edit link"
            aria-pressed={state.link}
            onMouseDown={(event) => event.preventDefault()}
            onClick={openLinkInput}
            className={cn(
              "rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground",
              state.link && "bg-accent text-accent-foreground",
            )}
          >
            <LinkIcon className="size-4" />
          </button>
        </>
      )}
    </BubbleMenu>
  );
}
