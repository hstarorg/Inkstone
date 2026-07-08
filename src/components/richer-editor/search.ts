import { Extension } from "@tiptap/core";
import { Plugin, PluginKey, TextSelection } from "@tiptap/pm/state";
import type { Node as PMNode } from "@tiptap/pm/model";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

interface Match {
  from: number;
  to: number;
}

interface SearchPluginState {
  term: string;
  matches: Match[];
  activeIndex: number;
  decorations: DecorationSet;
}

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    search: {
      setSearchTerm: (term: string) => ReturnType;
      clearSearch: () => ReturnType;
      findNext: () => ReturnType;
      findPrevious: () => ReturnType;
      replaceActive: (replacement: string) => ReturnType;
      replaceAll: (replacement: string) => ReturnType;
    };
  }
}

export const searchPluginKey = new PluginKey<SearchPluginState>("search");

function findMatches(doc: PMNode, term: string): Match[] {
  const matches: Match[] = [];
  if (!term) return matches;
  const query = term.toLowerCase();
  doc.descendants((node, pos) => {
    if (!node.isText || !node.text) return;
    const text = node.text.toLowerCase();
    let index = text.indexOf(query);
    while (index !== -1) {
      matches.push({ from: pos + index, to: pos + index + query.length });
      index = text.indexOf(query, index + query.length);
    }
  });
  return matches;
}

function buildState(
  doc: PMNode,
  term: string,
  activeIndex: number,
): SearchPluginState {
  const matches = findMatches(doc, term);
  const clamped = matches.length
    ? Math.min(Math.max(activeIndex, 0), matches.length - 1)
    : 0;
  const decorations = DecorationSet.create(
    doc,
    matches.map((match, index) =>
      Decoration.inline(match.from, match.to, {
        class:
          index === clamped
            ? "search-match search-match-active"
            : "search-match",
      }),
    ),
  );
  return { term, matches, activeIndex: clamped, decorations };
}

function jumpToMatch(index: number) {
  return ({
    state,
    dispatch,
  }: {
    state: import("@tiptap/pm/state").EditorState;
    dispatch?: (tr: import("@tiptap/pm/state").Transaction) => void;
  }) => {
    const pluginState = searchPluginKey.getState(state);
    if (!pluginState || pluginState.matches.length === 0) return false;
    const count = pluginState.matches.length;
    const target = ((index % count) + count) % count;
    const match = pluginState.matches[target];
    if (dispatch) {
      const tr = state.tr
        .setMeta(searchPluginKey, { activeIndex: target })
        .setSelection(TextSelection.create(state.doc, match.from, match.to))
        .scrollIntoView();
      dispatch(tr);
    }
    return true;
  };
}

export const Search = Extension.create({
  name: "search",

  addCommands() {
    return {
      setSearchTerm:
        (term) =>
        ({ state, dispatch }) => {
          if (dispatch) {
            dispatch(state.tr.setMeta(searchPluginKey, { term }));
          }
          return true;
        },
      clearSearch:
        () =>
        ({ state, dispatch }) => {
          if (dispatch) {
            dispatch(state.tr.setMeta(searchPluginKey, { term: "" }));
          }
          return true;
        },
      findNext:
        () =>
        ({ state, dispatch }) => {
          const pluginState = searchPluginKey.getState(state);
          if (!pluginState) return false;
          return jumpToMatch(pluginState.activeIndex + 1)({ state, dispatch });
        },
      findPrevious:
        () =>
        ({ state, dispatch }) => {
          const pluginState = searchPluginKey.getState(state);
          if (!pluginState) return false;
          return jumpToMatch(pluginState.activeIndex - 1)({ state, dispatch });
        },
      replaceActive:
        (replacement) =>
        ({ state, dispatch }) => {
          const pluginState = searchPluginKey.getState(state);
          if (!pluginState || pluginState.matches.length === 0) return false;
          const match = pluginState.matches[pluginState.activeIndex];
          if (dispatch) {
            dispatch(state.tr.insertText(replacement, match.from, match.to));
          }
          return true;
        },
      replaceAll:
        (replacement) =>
        ({ state, dispatch }) => {
          const pluginState = searchPluginKey.getState(state);
          if (!pluginState || pluginState.matches.length === 0) return false;
          if (dispatch) {
            let tr = state.tr;
            [...pluginState.matches].reverse().forEach((match) => {
              tr = tr.insertText(replacement, match.from, match.to);
            });
            dispatch(tr);
          }
          return true;
        },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin<SearchPluginState>({
        key: searchPluginKey,
        state: {
          init: (_, state) => buildState(state.doc, "", 0),
          apply: (tr, value, _oldState, newState) => {
            const meta = tr.getMeta(searchPluginKey) as
              { term?: string; activeIndex?: number } | undefined;
            if (meta?.term !== undefined) {
              return buildState(newState.doc, meta.term, 0);
            }
            if (meta?.activeIndex !== undefined) {
              return buildState(newState.doc, value.term, meta.activeIndex);
            }
            if (tr.docChanged && value.term) {
              return buildState(newState.doc, value.term, value.activeIndex);
            }
            return value;
          },
        },
        props: {
          decorations(state) {
            return searchPluginKey.getState(state)?.decorations;
          },
        },
      }),
    ];
  },
});
