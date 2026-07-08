import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import type { MarkdownManager } from "@tiptap/markdown";

const MARKDOWN_PATTERNS = [
  /^#{1,6}\s/m,
  /^\s*[-*+]\s/m,
  /^\s*\d+\.\s/m,
  /^\s*>\s/m,
  /```/,
  /\*\*[^*]+\*\*/,
  /\[[^\]]+\]\([^)]+\)/,
  /^\s*[-*_]{3,}\s*$/m,
  /^\|.*\|/m,
];

function looksLikeMarkdown(text: string): boolean {
  return MARKDOWN_PATTERNS.some((pattern) => pattern.test(text));
}

export const MarkdownClipboard = Extension.create({
  name: "markdownClipboard",

  addProseMirrorPlugins() {
    const editor = this.editor;
    const getManager = (): MarkdownManager | undefined =>
      editor.storage.markdown?.manager;

    return [
      new Plugin({
        key: new PluginKey("markdownClipboard"),
        props: {
          handlePaste: (_view, event) => {
            const clipboard = event.clipboardData;
            if (!clipboard) return false;
            if (clipboard.getData("text/html")) return false;
            if (editor.isActive("codeBlock")) return false;

            const text = clipboard.getData("text/plain");
            if (!text || !looksLikeMarkdown(text)) return false;

            const manager = getManager();
            if (!manager) return false;

            try {
              const doc = manager.parse(text);
              const content = doc.content;
              if (!content || content.length === 0) return false;
              editor.commands.insertContent(content);
              return true;
            } catch {
              return false;
            }
          },
          clipboardTextSerializer: (slice) => {
            const manager = getManager();
            const content = slice.content.toJSON();
            if (!manager || !content) {
              return slice.content.textBetween(0, slice.content.size, "\n\n");
            }
            try {
              return manager.serialize({ type: "doc", content });
            } catch {
              return slice.content.textBetween(0, slice.content.size, "\n\n");
            }
          },
        },
      }),
    ];
  },
});
