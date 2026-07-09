import { Extension } from "@tiptap/core";
import { Plugin, PluginKey, type EditorState } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

interface FocusModeState {
  enabled: boolean;
  decorations: DecorationSet;
}

export const focusModePluginKey = new PluginKey<FocusModeState>("focusMode");

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    focusMode: {
      setFocusMode: (enabled: boolean) => ReturnType;
    };
  }
}

function buildDecorations(state: EditorState): DecorationSet {
  const { from, to } = state.selection;
  const decorations: Decoration[] = [];
  state.doc.forEach((node, offset) => {
    const nodeFrom = offset;
    const nodeTo = offset + node.nodeSize;
    const isCurrent = from <= nodeTo && to >= nodeFrom;
    if (!isCurrent) {
      decorations.push(
        Decoration.node(nodeFrom, nodeTo, { class: "focus-dim" }),
      );
    }
  });
  return DecorationSet.create(state.doc, decorations);
}

export const FocusMode = Extension.create({
  name: "focusMode",

  addCommands() {
    return {
      setFocusMode:
        (enabled) =>
        ({ state, dispatch }) => {
          if (dispatch) {
            dispatch(state.tr.setMeta(focusModePluginKey, { enabled }));
          }
          return true;
        },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin<FocusModeState>({
        key: focusModePluginKey,
        state: {
          init: () => ({ enabled: false, decorations: DecorationSet.empty }),
          apply(tr, value, _oldState, newState) {
            const meta = tr.getMeta(focusModePluginKey) as
              { enabled: boolean } | undefined;
            const enabled = meta?.enabled ?? value.enabled;
            if (!enabled) {
              return { enabled, decorations: DecorationSet.empty };
            }
            if (!meta && !tr.docChanged && !tr.selectionSet) {
              return value;
            }
            return { enabled, decorations: buildDecorations(newState) };
          },
        },
        props: {
          decorations(state) {
            return focusModePluginKey.getState(state)?.decorations;
          },
        },
      }),
    ];
  },
});
