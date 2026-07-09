import { mergeAttributes, Node } from "@tiptap/core";
import { ReactNodeViewRenderer } from "@tiptap/react";
import { CanvasNodeView } from "./canvas-node-view";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    canvas: {
      insertCanvas: () => ReturnType;
    };
  }
}

export const CanvasNode = Node.create({
  name: "canvas",
  group: "block",
  atom: true,
  selectable: true,

  addAttributes() {
    return {
      scene: {
        default: [],
        parseHTML: (element) => {
          try {
            return JSON.parse(element.getAttribute("data-scene") ?? "[]");
          } catch {
            return [];
          }
        },
        renderHTML: (attributes) => ({
          "data-scene": JSON.stringify(attributes.scene),
        }),
      },
    };
  },

  parseHTML() {
    return [{ tag: 'div[data-type="canvas"]' }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["div", mergeAttributes(HTMLAttributes, { "data-type": "canvas" })];
  },

  addNodeView() {
    return ReactNodeViewRenderer(CanvasNodeView);
  },

  addCommands() {
    return {
      insertCanvas:
        () =>
        ({ commands }) =>
          commands.insertContent({ type: this.name }),
    };
  },
});
