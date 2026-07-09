import { useState } from "react";
import {
  EditorContent,
  ReactNodeViewRenderer,
  useEditor,
  useEditorState,
  type Editor,
  type JSONContent,
} from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { CharacterCount, Placeholder } from "@tiptap/extensions";
import Typography from "@tiptap/extension-typography";
import { CodeBlockLowlight } from "@tiptap/extension-code-block-lowlight";
import { TableKit } from "@tiptap/extension-table";
import { TaskItem, TaskList } from "@tiptap/extension-list";
import Highlight from "@tiptap/extension-highlight";
import {
  Details,
  DetailsContent,
  DetailsSummary,
} from "@tiptap/extension-details";
import NodeRange from "@tiptap/extension-node-range";
import { Markdown } from "@tiptap/markdown";
import { DragHandle } from "@tiptap/extension-drag-handle-react";
import { GripVertical } from "lucide-react";
import { common, createLowlight } from "lowlight";
import { cn } from "@/lib/utils";
import {
  createAssetImage,
  createImageDropPaste,
  type ResolveAssetSrc,
  type UploadImage,
} from "./asset-image";
import { RicherEditorBubbleMenu } from "./bubble-menu";
import { Callout } from "./callout";
import { CanvasNode } from "./canvas-node";
import { CodeBlockView } from "./code-block-view";
import { MarkdownClipboard } from "./markdown-clipboard";
import { Search } from "./search";
import { SearchPanel } from "./search-panel";
import { SlashCommand } from "./slash-menu";
import "./richer-editor.css";

const lowlight = createLowlight(common);

const CodeBlock = CodeBlockLowlight.extend({
  addNodeView() {
    return ReactNodeViewRenderer(CodeBlockView);
  },
}).configure({ lowlight });

export interface RicherEditorProps {
  defaultValue?: JSONContent;
  onChange?: (value: JSONContent) => void;
  editable?: boolean;
  autofocus?: boolean;
  placeholder?: string;
  showCharacterCount?: boolean;
  uploadImage?: UploadImage;
  resolveAssetSrc?: ResolveAssetSrc;
  className?: string;
  contentClassName?: string;
}

function CharacterCountBadge({ editor }: { editor: Editor }) {
  const characters = useEditorState({
    editor,
    selector: ({ editor }) => editor.storage.characterCount.characters(),
  });

  return (
    <div className="pointer-events-none absolute right-4 bottom-2 z-30 rounded-md bg-background/80 px-1.5 py-0.5 text-xs text-muted-foreground">
      {characters} characters
    </div>
  );
}

export function RicherEditor({
  defaultValue,
  onChange,
  editable = true,
  autofocus = false,
  placeholder = "Start writing…",
  showCharacterCount = true,
  uploadImage,
  resolveAssetSrc = (src) => src,
  className,
  contentClassName,
}: RicherEditorProps) {
  const [searchOpen, setSearchOpen] = useState(false);

  const editor = useEditor({
    extensions: [
      createAssetImage(resolveAssetSrc),
      createImageDropPaste(uploadImage),
      StarterKit.configure({
        codeBlock: false,
        link: { openOnClick: false },
      }),
      CodeBlock,
      TableKit,
      TaskList,
      TaskItem.configure({ nested: true }),
      Highlight.configure({ multicolor: true }),
      Details.configure({ persist: true }),
      DetailsSummary,
      DetailsContent,
      Typography,
      Placeholder.configure({ placeholder }),
      CharacterCount,
      Search,
      Callout,
      CanvasNode,
      SlashCommand,
      NodeRange,
      Markdown,
      MarkdownClipboard,
    ],
    content: defaultValue,
    editable,
    autofocus: autofocus ? "end" : false,
    editorProps: {
      attributes: {
        class: cn(
          "prose prose-neutral dark:prose-invert mx-auto max-w-3xl px-8 py-10 focus:outline-none",
          contentClassName,
        ),
      },
    },
    onUpdate: ({ editor }) => {
      onChange?.(editor.getJSON());
    },
  });

  return (
    <div
      className={cn("relative h-full", className)}
      onKeyDown={(event) => {
        if (
          (event.metaKey || event.ctrlKey) &&
          event.key.toLowerCase() === "f"
        ) {
          event.preventDefault();
          setSearchOpen(true);
        }
      }}
    >
      {editor && editable && <RicherEditorBubbleMenu editor={editor} />}
      {editor && editable && (
        <DragHandle editor={editor}>
          <div className="mr-1 cursor-grab rounded-md p-0.5 text-popover-foreground/40 hover:bg-accent hover:text-accent-foreground">
            <GripVertical className="size-4" />
          </div>
        </DragHandle>
      )}
      {editor && searchOpen && (
        <SearchPanel
          editor={editor}
          onClose={() => {
            setSearchOpen(false);
            editor.commands.focus();
          }}
        />
      )}
      <EditorContent editor={editor} className="h-full overflow-y-auto" />
      {editor && showCharacterCount && <CharacterCountBadge editor={editor} />}
    </div>
  );
}
