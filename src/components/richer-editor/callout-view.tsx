import {
  NodeViewContent,
  NodeViewWrapper,
  type ReactNodeViewProps,
} from "@tiptap/react";
import {
  Info,
  Lightbulb,
  OctagonAlert,
  TriangleAlert,
  type LucideIcon,
} from "lucide-react";

import { CALLOUT_VARIANTS, type CalloutVariant } from "./callout-variants";

const VARIANT_ICONS: Record<CalloutVariant, LucideIcon> = {
  info: Info,
  tip: Lightbulb,
  warn: TriangleAlert,
  danger: OctagonAlert,
};

export function CalloutView({ node, updateAttributes }: ReactNodeViewProps) {
  const variant = node.attrs.variant as CalloutVariant;
  const Icon = VARIANT_ICONS[variant];

  const cycleVariant = () => {
    const index = CALLOUT_VARIANTS.indexOf(variant);
    const next = CALLOUT_VARIANTS[(index + 1) % CALLOUT_VARIANTS.length];
    updateAttributes({ variant: next });
  };

  return (
    <NodeViewWrapper data-type="callout" data-variant={variant}>
      <button
        type="button"
        contentEditable={false}
        onClick={cycleVariant}
        aria-label={`Callout style: ${variant}. Click to change`}
        className="callout-icon"
      >
        <Icon className="size-4" />
      </button>
      <NodeViewContent className="callout-content" />
    </NodeViewWrapper>
  );
}
