/**
 * Localization: externalized UI strings with English fallback.
 *
 * Every user-facing string resolves through a message catalog — components
 * never hardcode visible text. Catalogs ship per locale; a key missing from
 * the active locale falls back to English so the UI never renders a blank.
 */

/** Supported locales; the catalog is extensible. */
export type Locale = "en" | "ru";

export const DEFAULT_LOCALE: Locale = "en";

/** Message keys — typed so a missing key is a compile error, not a blank. */
export type MessageKey =
  | "app.title"
  | "status.connecting"
  | "surface.office"
  | "surface.board"
  | "surface.chat"
  | "surface.editor"
  | "surface.dashboard"
  | "surface.empty"
  | "office.empty"
  | "dashboard.building"
  | "dashboard.offices"
  | "dashboard.active-agents"
  | "dashboard.cards";

type Catalog = Record<MessageKey, string>;

const en: Catalog = {
  "app.title": "Cronus",
  "status.connecting": "connecting…",
  "surface.office": "Office",
  "surface.board": "Board",
  "surface.chat": "Chat",
  "surface.editor": "Editor",
  "surface.dashboard": "Dashboard",
  "surface.empty": "Nothing here yet — the core will fill this surface.",
  "office.empty": "No office staffed yet.",
  "dashboard.building": "Building overview",
  "dashboard.offices": "Offices",
  "dashboard.active-agents": "Active agents",
  "dashboard.cards": "Cards",
};

// Deliberately partial: brand strings (app.title) are not translated and
// exercise the English fallback path.
const ru: Partial<Catalog> = {
  "status.connecting": "подключение…",
  "surface.office": "Офис",
  "surface.board": "Доска",
  "surface.chat": "Чат",
  "surface.editor": "Редактор",
  "surface.dashboard": "Дашборд",
  "surface.empty": "Здесь пока пусто — ядро наполнит эту поверхность.",
  "office.empty": "Офис пока не укомплектован.",
  "dashboard.building": "Обзор здания",
  "dashboard.offices": "Офисы",
  "dashboard.active-agents": "Активные агенты",
  "dashboard.cards": "Карточки",
};

const catalogs: Record<Locale, Partial<Catalog>> = { en, ru };

/** Resolve a message for a locale, falling back to English. */
export function t(locale: Locale, key: MessageKey): string {
  return catalogs[locale][key] ?? en[key];
}

/** Bind a locale once and translate with the shorter `msg(key)` form. */
export function translator(locale: Locale): (key: MessageKey) => string {
  return (key) => t(locale, key);
}
