/**
 * Theming: system / light / dark over design tokens.
 *
 * The theme choice is cosmetic only and applies via a `data-theme` attribute
 * plus Tailwind's `dark` class on the surface root — tokens react to those;
 * components never hardcode literal colors for themed surfaces.
 */

/** The persisted theme choice. */
export type Theme = "system" | "light" | "dark";

/** What a theme choice renders as, once the OS preference is known. */
export type ResolvedTheme = "light" | "dark";

/** Resolve `system` against the OS preference; explicit choices pass through. */
export function resolveTheme(theme: Theme, systemPrefersDark: boolean): ResolvedTheme {
  if (theme === "system") {
    return systemPrefersDark ? "dark" : "light";
  }
  return theme;
}

/** Token attributes for a resolved theme, applied on the surface root. */
export function themeAttributes(resolved: ResolvedTheme): {
  "data-theme": ResolvedTheme;
  className: string;
} {
  return {
    "data-theme": resolved,
    className: resolved === "dark" ? "dark" : "",
  };
}
