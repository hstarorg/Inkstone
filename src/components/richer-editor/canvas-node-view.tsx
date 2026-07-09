import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { NodeViewWrapper, type ReactNodeViewProps } from "@tiptap/react";
import { Pencil, Shapes, Trash2 } from "lucide-react";
import {
  CanvasPreview,
  RicherCanvas,
  type CanvasScene,
} from "@/components/richer-canvas";
import { cn } from "@/lib/utils";

export function CanvasNodeView({
  node,
  updateAttributes,
  deleteNode,
  selected,
  editor,
}: ReactNodeViewProps) {
  const scene = node.attrs.scene as CanvasScene;
  const [editing, setEditing] = useState(false);
  const draft = useRef<CanvasScene>(scene);
  const overlayRef = useRef<HTMLDivElement>(null);

  const [tracked, setTracked] = useState({ scene, version: 0 });
  if (tracked.scene !== scene) {
    setTracked({ scene, version: tracked.version + 1 });
  }
  const previewVersion = tracked.version;

  // The underlying Tiptap editor must fully release DOM focus before Plait's
  // board mounts — otherwise ProseMirror's own keydown handling on its still
  // -focused contenteditable races Plait's deferred (setTimeout) focus calls
  // for entering a mind node's text-edit mode, and reliably wins, so the
  // node's first Enter press silently fails to start editing.
  useEffect(() => {
    if (editing) {
      overlayRef.current?.focus();
    }
  }, [editing]);

  const openEditor = () => {
    if (!editor.isEditable) return;
    draft.current = scene;
    editor.commands.blur();
    setEditing(true);
  };

  const commit = () => {
    updateAttributes({ scene: draft.current });
    setEditing(false);
    editor.commands.focus();
  };

  const discard = () => {
    setEditing(false);
    editor.commands.focus();
  };

  return (
    <NodeViewWrapper data-type="canvas">
      <div
        className={cn(
          "canvas-frame group/canvas",
          selected && "canvas-frame-selected",
        )}
        onDoubleClick={openEditor}
      >
        {scene.length > 0 ? (
          <CanvasPreview key={previewVersion} scene={scene} />
        ) : (
          <div className="flex h-full flex-col items-center justify-center gap-2 text-muted-foreground">
            <Shapes className="size-6" />
            <span className="text-sm">Empty canvas — double-click to draw</span>
          </div>
        )}
        {editor.isEditable && (
          <div className="canvas-toolbar flex items-center gap-1">
            <button
              type="button"
              aria-label="Edit canvas"
              onClick={openEditor}
              className="rounded-md border bg-popover p-1.5 text-popover-foreground shadow-sm hover:bg-accent hover:text-accent-foreground"
            >
              <Pencil className="size-3.5" />
            </button>
            <button
              type="button"
              aria-label="Delete canvas"
              onClick={() => deleteNode()}
              className="rounded-md border bg-popover p-1.5 text-popover-foreground shadow-sm hover:bg-destructive hover:text-destructive-foreground"
            >
              <Trash2 className="size-3.5" />
            </button>
          </div>
        )}
      </div>
      {editing &&
        createPortal(
          <div
            ref={overlayRef}
            tabIndex={-1}
            className="fixed inset-0 z-50 flex flex-col bg-background outline-none"
          >
            <div className="flex items-center gap-2 border-b px-4 py-2">
              <Shapes className="size-4 text-muted-foreground" />
              <span className="flex-1 text-sm font-medium">Canvas</span>
              <button
                type="button"
                onClick={discard}
                className="rounded-md px-3 py-1 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              >
                Discard
              </button>
              <button
                type="button"
                onClick={commit}
                className="rounded-md bg-primary px-3 py-1 text-sm text-primary-foreground hover:bg-primary/90"
              >
                Done
              </button>
            </div>
            <div
              className="min-h-0 flex-1"
              onKeyDown={(event) => {
                if (event.key === "Escape") {
                  event.preventDefault();
                  commit();
                }
              }}
            >
              <RicherCanvas
                defaultValue={scene}
                onChange={(value) => {
                  draft.current = value;
                }}
              />
            </div>
          </div>,
          document.body,
        )}
    </NodeViewWrapper>
  );
}
