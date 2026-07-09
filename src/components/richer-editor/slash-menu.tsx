import { Extension } from "@tiptap/core";
import { ReactRenderer } from "@tiptap/react";
import Suggestion, {
  type SuggestionKeyDownProps,
  type SuggestionProps,
} from "@tiptap/suggestion";
import {
  Code,
  Heading1,
  Heading2,
  Heading3,
  Info,
  List,
  ListCollapse,
  ListOrdered,
  ListTodo,
  Minus,
  Pilcrow,
  Shapes,
  Table,
  TextQuote,
} from "lucide-react";
import {
  SlashList,
  type SlashCommandItem,
  type SlashListProps,
  type SlashListRef,
} from "./slash-list";

const COMMANDS: SlashCommandItem[] = [
  {
    key: "text",
    label: "Text",
    keywords: "paragraph plain 正文 文本 zhengwen wenben",
    icon: Pilcrow,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).setParagraph().run(),
  },
  {
    key: "h1",
    label: "Heading 1",
    keywords: "h1 title 一级标题 标题 biaoti yiji",
    icon: Heading1,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).setHeading({ level: 1 }).run(),
  },
  {
    key: "h2",
    label: "Heading 2",
    keywords: "h2 subtitle 二级标题 标题 biaoti erji",
    icon: Heading2,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).setHeading({ level: 2 }).run(),
  },
  {
    key: "h3",
    label: "Heading 3",
    keywords: "h3 三级标题 标题 biaoti sanji",
    icon: Heading3,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).setHeading({ level: 3 }).run(),
  },
  {
    key: "bulletList",
    label: "Bullet list",
    keywords: "ul unordered 无序列表 列表 liebiao wuxu",
    icon: List,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).toggleBulletList().run(),
  },
  {
    key: "orderedList",
    label: "Ordered list",
    keywords: "ol numbered 有序列表 列表 liebiao youxu",
    icon: ListOrdered,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).toggleOrderedList().run(),
  },
  {
    key: "taskList",
    label: "Task list",
    keywords: "todo checkbox check 任务 待办 清单 renwu daiban",
    icon: ListTodo,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).toggleTaskList().run(),
  },
  {
    key: "table",
    label: "Table",
    keywords: "grid rows columns 表格 biaoge bg",
    icon: Table,
    run: (editor, range) =>
      editor
        .chain()
        .focus()
        .deleteRange(range)
        .insertTable({ rows: 3, cols: 3, withHeaderRow: true })
        .run(),
  },
  {
    key: "quote",
    label: "Quote",
    keywords: "blockquote citation 引用 yinyong",
    icon: TextQuote,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).toggleBlockquote().run(),
  },
  {
    key: "codeBlock",
    label: "Code block",
    keywords: "snippet pre fence 代码 代码块 daima",
    icon: Code,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).toggleCodeBlock().run(),
  },
  {
    key: "canvas",
    label: "Canvas",
    keywords: "draw board whiteboard mind 画布 白板 脑图 手绘 huaban baiban",
    icon: Shapes,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).insertCanvas().run(),
  },
  {
    key: "callout",
    label: "Callout",
    keywords:
      "info tip warning danger note admonition 信息块 提示 xinxikuai tishi",
    icon: Info,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).toggleCallout().run(),
  },
  {
    key: "details",
    label: "Toggle",
    keywords: "details collapse fold 折叠 zhedie",
    icon: ListCollapse,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).setDetails().run(),
  },
  {
    key: "divider",
    label: "Divider",
    keywords: "hr horizontal rule separator 分割线 分隔线 fengexian fgx",
    icon: Minus,
    run: (editor, range) =>
      editor.chain().focus().deleteRange(range).setHorizontalRule().run(),
  },
];

function filterCommands(query: string): SlashCommandItem[] {
  const q = query.toLowerCase().trim();
  if (!q) return COMMANDS;
  return COMMANDS.filter(
    (item) => item.label.toLowerCase().includes(q) || item.keywords.includes(q),
  );
}

function positionElement(
  element: HTMLElement,
  clientRect: (() => DOMRect | null) | null | undefined,
) {
  const rect = clientRect?.();
  if (!rect) return;
  element.style.position = "fixed";
  element.style.zIndex = "50";
  element.style.left = `${rect.left}px`;
  const spaceBelow = window.innerHeight - rect.bottom;
  if (spaceBelow < 320) {
    element.style.top = "auto";
    element.style.bottom = `${window.innerHeight - rect.top + 4}px`;
  } else {
    element.style.bottom = "auto";
    element.style.top = `${rect.bottom + 4}px`;
  }
}

export const SlashCommand = Extension.create({
  name: "slashCommand",

  addProseMirrorPlugins() {
    return [
      Suggestion<SlashCommandItem>({
        editor: this.editor,
        char: "/",
        command: ({ editor, range, props }) => props.run(editor, range),
        items: ({ query }) => filterCommands(query),
        render: () => {
          let component: ReactRenderer<SlashListRef, SlashListProps> | null =
            null;

          const destroy = () => {
            component?.element.remove();
            component?.destroy();
            component = null;
          };

          return {
            onStart: (props: SuggestionProps<SlashCommandItem>) => {
              component = new ReactRenderer(SlashList, {
                props: {
                  items: props.items,
                  command: (item: SlashCommandItem) => props.command(item),
                },
                editor: props.editor,
              });
              document.body.appendChild(component.element);
              positionElement(
                component.element as HTMLElement,
                props.clientRect,
              );
            },
            onUpdate: (props: SuggestionProps<SlashCommandItem>) => {
              component?.updateProps({
                items: props.items,
                command: (item: SlashCommandItem) => props.command(item),
              });
              if (component) {
                positionElement(
                  component.element as HTMLElement,
                  props.clientRect,
                );
              }
            },
            onKeyDown: (props: SuggestionKeyDownProps) => {
              if (props.event.key === "Escape") {
                destroy();
                return true;
              }
              return component?.ref?.onKeyDown(props.event) ?? false;
            },
            onExit: destroy,
          };
        },
      }),
    ];
  },
});

export type { SlashCommandItem };
