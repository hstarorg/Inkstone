export const CALLOUT_VARIANTS = ["info", "tip", "warn", "danger"] as const;
export type CalloutVariant = (typeof CALLOUT_VARIANTS)[number];
