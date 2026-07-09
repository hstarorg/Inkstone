import { Drawnix } from "@drawnix/drawnix";
import "@plait-board/react-board/index.css";
import "@plait-board/react-text/index.css";
import "@drawnix/drawnix/index.css";
import "./richer-canvas.css";
import { cn } from "@/lib/utils";
import type { PlaitElement } from "@plait/core";
import { CanvasErrorBoundary, CanvasFallback } from "./canvas-error-boundary";
import { IsolatedRoot } from "./isolated-root";

export type CanvasScene = PlaitElement[];

export interface RicherCanvasProps {
  defaultValue?: CanvasScene;
  onChange?: (scene: CanvasScene) => void;
  className?: string;
}

export function RicherCanvas({
  defaultValue = [],
  onChange,
  className,
}: RicherCanvasProps) {
  return (
    <div className={cn("richer-canvas", className)}>
      <IsolatedRoot>
        <CanvasErrorBoundary
          fallback={<CanvasFallback message="Canvas failed to load" />}
        >
          <Drawnix
            value={defaultValue}
            onChange={(data) => {
              onChange?.(data.children);
            }}
          />
        </CanvasErrorBoundary>
      </IsolatedRoot>
    </div>
  );
}
