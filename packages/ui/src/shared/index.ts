/**
 * Shared tier — the leaf declaration.
 *
 * Every module here is a leaf: it names no surface, no shell component, and no
 * composition-root module, in an import or a type position. Shared modules may
 * depend on each other as long as the result stays acyclic. This barrel is the
 * tier's public surface; importers name `./shared`, not a file inside it.
 */

export * from "./bridge";
export * from "./canvas";
export * from "./i18n";
export * from "./navigation";
export * from "./surface-catalog";
export * from "./theme";
export * from "./tokens";
