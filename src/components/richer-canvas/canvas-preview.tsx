import { Drawnix } from "@drawnix/drawnix";
import "@plait-board/react-board/index.css";
import "@plait-board/react-text/index.css";
import "@drawnix/drawnix/index.css";
import { cn } from "@/lib/utils";
import { CanvasErrorBoundary, CanvasFallback } from "./canvas-error-boundary";
import type { CanvasScene } from "./richer-canvas";
import "./richer-canvas.css";

export interface CanvasPreviewProps {
  scene: CanvasScene;
  className?: string;
}

export function CanvasPreview({ scene, className }: CanvasPreviewProps) {
  return (
    <div className={cn("richer-canvas-preview", className)}>
      <CanvasErrorBoundary
        fallback={<CanvasFallback message="Canvas failed to render" />}
      >
        <Drawnix value={scene} />
      </CanvasErrorBoundary>
    </div>
  );
}
