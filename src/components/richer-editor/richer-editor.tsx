import { EditorContent, useEditor, type JSONContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { cn } from "@/lib/utils";

export interface RicherEditorProps {
  defaultValue?: JSONContent;
  onChange?: (value: JSONContent) => void;
  editable?: boolean;
  autofocus?: boolean;
  className?: string;
  contentClassName?: string;
}

export function RicherEditor({
  defaultValue,
  onChange,
  editable = true,
  autofocus = false,
  className,
  contentClassName,
}: RicherEditorProps) {
  const editor = useEditor({
    extensions: [StarterKit],
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
    <EditorContent
      editor={editor}
      className={cn("h-full overflow-y-auto", className)}
    />
  );
}
