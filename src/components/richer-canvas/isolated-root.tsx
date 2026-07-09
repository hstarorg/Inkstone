import { useEffect, useRef, type ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";

/**
 * Plait's imperative board setup (global keydown listener, selection
 * bookkeeping) misbehaves under React StrictMode's dev-only double-invoke of
 * mount effects — e.g. a mind node needs two Enter presses to enter edit
 * mode. A separate React root sidesteps StrictMode entirely, since
 * double-invoke only applies within the enclosing <StrictMode> boundary's
 * own reconciler tree.
 */
export function IsolatedRoot({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const rootRef = useRef<Root | null>(null);

  useEffect(() => {
    if (!hostRef.current) return;
    rootRef.current = createRoot(hostRef.current);
    return () => {
      rootRef.current?.unmount();
      rootRef.current = null;
    };
  }, []);

  useEffect(() => {
    rootRef.current?.render(children);
  });

  return (
    <div
      ref={hostRef}
      className={className}
      style={{ width: "100%", height: "100%" }}
    />
  );
}
