import { mergeAttributes, Node } from "@tiptap/core";
import { ReactNodeViewRenderer } from "@tiptap/react";
import { CalloutView } from "./callout-view";
import type { CalloutVariant } from "./callout-variants";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    callout: {
      setCallout: (attributes?: { variant: CalloutVariant }) => ReturnType;
      toggleCallout: (attributes?: { variant: CalloutVariant }) => ReturnType;
    };
  }
}

export const Callout = Node.create({
  name: "callout",
  group: "block",
  content: "block+",
  defining: true,

  addAttributes() {
    return {
      variant: {
        default: "info",
        parseHTML: (element) => element.getAttribute("data-variant"),
        renderHTML: (attributes) => ({ "data-variant": attributes.variant }),
      },
    };
  },

  parseHTML() {
    return [{ tag: 'div[data-type="callout"]' }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "div",
      mergeAttributes(HTMLAttributes, { "data-type": "callout" }),
      0,
    ];
  },

  addNodeView() {
    return ReactNodeViewRenderer(CalloutView);
  },

  addCommands() {
    return {
      setCallout:
        (attributes) =>
        ({ commands }) =>
          commands.wrapIn(this.name, attributes),
      toggleCallout:
        (attributes) =>
        ({ commands }) =>
          commands.toggleWrap(this.name, attributes),
    };
  },
});
