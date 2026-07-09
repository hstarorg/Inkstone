import { useRef, useState } from "react";
import { Drawnix } from "@drawnix/drawnix";
import "../../node_modules/@plait-board/react-board/index.css";
import "../../node_modules/@plait-board/react-text/index.css";
import "../../node_modules/@drawnix/drawnix/index.css";
import "./canvas-spike.css";
import type { PlaitElement } from "@plait/core";

const STORAGE_KEY = "canvas-spike-value";

function loadInitial(): PlaitElement[] {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return [];
  try {
    return JSON.parse(stored) as PlaitElement[];
  } catch {
    return [];
  }
}

export function CanvasSpike() {
  const [initialValue, setInitialValue] = useState<PlaitElement[]>(loadInitial);
  const valueRef = useRef<PlaitElement[]>(initialValue);
  const [status, setStatus] = useState("ready");
  const [generation, setGeneration] = useState(0);

  const save = () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(valueRef.current));
    setStatus(`saved ${JSON.stringify(valueRef.current).length} bytes`);
  };

  const reload = () => {
    const value = loadInitial();
    valueRef.current = value;
    setInitialValue(value);
    setGeneration((n) => n + 1);
    setStatus("reloaded");
  };

  const clear = () => {
    localStorage.removeItem(STORAGE_KEY);
    valueRef.current = [];
    setInitialValue([]);
    setGeneration((n) => n + 1);
    setStatus("cleared");
  };

  return (
    <div className="flex h-screen flex-col">
      <div className="flex items-center gap-2 border-b p-2 text-sm">
        <strong>Canvas spike (Drawnix)</strong>
        <button className="rounded border px-2 py-0.5" onClick={save}>
          Save JSON
        </button>
        <button className="rounded border px-2 py-0.5" onClick={reload}>
          Reload
        </button>
        <button className="rounded border px-2 py-0.5" onClick={clear}>
          Clear
        </button>
        <span className="text-muted-foreground">{status}</span>
      </div>
      <div className="relative min-h-0 flex-1 overflow-hidden">
        <Drawnix
          key={generation}
          value={initialValue}
          onChange={(data) => {
            valueRef.current = data.children;
          }}
        />
      </div>
    </div>
  );
}
