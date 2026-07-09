import { forwardRef, useEffect, useImperativeHandle, useState } from "react";
import type { Editor, Range } from "@tiptap/core";
import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

export interface SlashCommandItem {
  key: string;
  label: string;
  keywords: string;
  icon: LucideIcon;
  run: (editor: Editor, range: Range) => void;
}

export interface SlashListProps {
  items: SlashCommandItem[];
  command: (item: SlashCommandItem) => void;
}

export interface SlashListRef {
  onKeyDown: (event: KeyboardEvent) => boolean;
}

export const SlashList = forwardRef<SlashListRef, SlashListProps>(
  function SlashList({ items, command }, ref) {
    const [selected, setSelected] = useState(0);

    useEffect(() => {
      setSelected(0);
    }, [items]);

    useImperativeHandle(ref, () => ({
      onKeyDown: (event) => {
        if (event.key === "ArrowDown") {
          setSelected((current) => (current + 1) % items.length);
          return true;
        }
        if (event.key === "ArrowUp") {
          setSelected((current) => (current - 1 + items.length) % items.length);
          return true;
        }
        if (event.key === "Enter") {
          if (items[selected]) command(items[selected]);
          return true;
        }
        return false;
      },
    }));

    if (items.length === 0) {
      return (
        <div className="rounded-lg border bg-popover px-3 py-2 text-sm text-muted-foreground shadow-md">
          No results
        </div>
      );
    }

    return (
      <div className="max-h-72 w-48 overflow-y-auto rounded-lg border bg-popover p-1 text-popover-foreground shadow-md">
        {items.map((item, index) => (
          <button
            key={item.key}
            type="button"
            onClick={() => command(item)}
            onMouseEnter={() => setSelected(index)}
            className={cn(
              "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm",
              index === selected && "bg-accent text-accent-foreground",
            )}
          >
            <item.icon className="size-4 shrink-0" />
            {item.label}
          </button>
        ))}
      </div>
    );
  },
);
