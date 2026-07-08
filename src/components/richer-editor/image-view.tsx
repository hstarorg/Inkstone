import { useRef } from "react";
import { NodeViewWrapper, type ReactNodeViewProps } from "@tiptap/react";
import { cn } from "@/lib/utils";
import type { ResolveAssetSrc } from "./asset-image";

const MIN_WIDTH = 80;

export function ImageView({
  node,
  extension,
  selected,
  updateAttributes,
}: ReactNodeViewProps) {
  const resolveSrc = extension.options.resolveSrc as ResolveAssetSrc;
  const imgRef = useRef<HTMLImageElement>(null);
  const width = node.attrs.width as number | null;

  const startResize = (event: React.PointerEvent) => {
    event.preventDefault();
    event.stopPropagation();
    const img = imgRef.current;
    if (!img) return;
    const startX = event.clientX;
    const startWidth = img.getBoundingClientRect().width;
    const widthAt = (clientX: number) =>
      Math.max(MIN_WIDTH, Math.round(startWidth + (clientX - startX)));

    const onMove = (moveEvent: PointerEvent) => {
      img.style.width = `${widthAt(moveEvent.clientX)}px`;
    };
    const onUp = (upEvent: PointerEvent) => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      img.style.width = "";
      updateAttributes({ width: widthAt(upEvent.clientX) });
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  };

  return (
    <NodeViewWrapper className="asset-image" data-drag-handle>
      <div className={cn("image-frame", selected && "is-selected")}>
        <img
          ref={imgRef}
          src={resolveSrc(node.attrs.src as string)}
          alt={(node.attrs.alt as string | null) ?? ""}
          width={width ?? undefined}
          draggable={false}
        />
        <span
          role="separator"
          aria-label="Resize image"
          className="resize-handle"
          onPointerDown={startResize}
        />
      </div>
    </NodeViewWrapper>
  );
}
