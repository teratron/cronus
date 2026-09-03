/**
 * Canonical design-token names — the single visual source of truth.
 *
 * Every visual attribute a component renders (colour, type, spacing, radius,
 * motion, elevation) derives from one of these CSS custom properties, directly
 * or through a Tailwind utility mapped to it in `tokens.css`'s `@theme` block.
 * A colour scheme supplies a value for every name below, per mode; a literal
 * colour / font / radius outside this token layer is a craft defect, caught by
 * the craft lint (see `craft-lint.ts`).
 *
 * Semantic-role names only — never a raw hue — so a scheme can recolour the
 * whole shell without a single component changing.
 */

export const CANONICAL_TOKENS = [
  // colour · surface (app ground → raised → overlay)
  "--surface-0",
  "--surface-1",
  "--surface-2",
  "--surface-3",
  // colour · text
  "--text-primary",
  "--text-secondary",
  "--text-muted",
  "--text-inverse",
  // colour · line
  "--border-subtle",
  "--border-strong",
  "--focus-ring",
  // colour · accent
  "--accent",
  "--accent-hover",
  "--accent-contrast",
  // colour · semantic (+ a -subtle background variant each)
  "--success",
  "--success-subtle",
  "--warning",
  "--warning-subtle",
  "--danger",
  "--danger-subtle",
  "--info",
  "--info-subtle",
  // typography · family
  "--font-sans",
  "--font-mono",
  // typography · size scale
  "--text-xs",
  "--text-sm",
  "--text-base",
  "--text-lg",
  "--text-xl",
  "--text-2xl",
  // typography · weight
  "--weight-regular",
  "--weight-medium",
  "--weight-semibold",
  // typography · line-height
  "--leading-tight",
  "--leading-normal",
  "--leading-relaxed",
  // spacing · 4px base scale
  "--space-1",
  "--space-2",
  "--space-3",
  "--space-4",
  "--space-5",
  "--space-6",
  "--space-8",
  // radius
  "--radius-sm",
  "--radius-md",
  "--radius-lg",
  "--radius-pill",
  // motion
  "--duration-fast",
  "--duration-base",
  "--duration-slow",
  "--ease-standard",
  "--ease-emphasized",
  // elevation
  "--shadow-panel",
  "--shadow-overlay",
] as const;

/** One canonical token name. */
export type TokenName = (typeof CANONICAL_TOKENS)[number];
