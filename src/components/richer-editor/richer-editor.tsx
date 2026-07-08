import { EditorContent, useEditor, type JSONContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { Placeholder } from "@tiptap/extensions";
import Typography from "@tiptap/extension-typography";
import { CodeBlockLowlight } from "@tiptap/extension-code-block-lowlight";
import { common, createLowlight } from "lowlight";
import { cn } from "@/lib/utils";
import { RicherEditorBubbleMenu } from "./bubble-menu";
import "./richer-editor.css";

const lowlight = createLowlight(common);

export interface RicherEditorProps {
  defaultValue?: JSONContent;
  onChange?: (value: JSONContent) => void;
  editable?: boolean;
  autofocus?: boolean;
  placeholder?: string;
  className?: string;
  contentClassName?: string;
}

export function RicherEditor({
  defaultValue,
  onChange,
  editable = true,
  autofocus = false,
  placeholder = "Start writing…",
  className,
  contentClassName,
}: RicherEditorProps) {
  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        codeBlock: false,
        link: { openOnClick: false },
      }),
      CodeBlockLowlight.configure({ lowlight }),
      Typography,
      Placeholder.configure({ placeholder }),
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
    <>
      {editor && editable && <RicherEditorBubbleMenu editor={editor} />}
      <EditorContent
        editor={editor}
        className={cn("h-full overflow-y-auto", className)}
      />
    </>
  );
}
