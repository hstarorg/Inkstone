import {
  NodeViewContent,
  NodeViewWrapper,
  type ReactNodeViewProps,
} from "@tiptap/react";

export function CodeBlockView({
  node,
  updateAttributes,
  extension,
}: ReactNodeViewProps) {
  const languages = (
    extension.options.lowlight.listLanguages() as string[]
  ).sort();
  const language = (node.attrs.language as string | null) ?? "auto";

  return (
    <NodeViewWrapper className="code-block">
      <select
        contentEditable={false}
        value={language}
        aria-label="Code language"
        onChange={(event) =>
          updateAttributes({
            language: event.target.value === "auto" ? null : event.target.value,
          })
        }
      >
        <option value="auto">auto</option>
        {languages.map((lang) => (
          <option key={lang} value={lang}>
            {lang}
          </option>
        ))}
      </select>
      <pre spellCheck={false}>
        <NodeViewContent<"code"> as="code" />
      </pre>
    </NodeViewWrapper>
  );
}
