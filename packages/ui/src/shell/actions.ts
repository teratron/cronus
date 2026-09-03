/**
 * Thin action registry — the minimal command vocabulary this slice needs.
 *
 * A `ShellAction` is a named command independent of how it is triggered (a menu
 * leaf, a keybinding, a palette row, or code). This is deliberately NOT the full
 * context-predicate keymap dispatch of the application-shell concept (AS-6…AS-8):
 * there is no focus tree, no bubbling, no layered keymap merge. It is a flat
 * `id → { label, run, binding }` map that the menu renders from and the command
 * palette sources — enough for the shell frame, replaced by the real dispatch
 * model when a surface needs it.
 */

import type { MessageKey } from "../shared/i18n";

/** One registered command. */
export interface ShellAction {
  /** Stable namespaced id, e.g. `"file.settings"`. */
  id: string;
  /** i18n key for the user-visible label. */
  labelKey: MessageKey;
  /** What the action does. Presentation-only callers pass a no-op or an intent. */
  run: () => void;
  /** Current keybinding, display-only (e.g. `"Ctrl ,"`). */
  binding?: string;
  /** Whether the action is bound to a shipped capability. An unbound action is
   *  hidden from every surface (INV-9) — never rendered as a dead control. */
  bound?: boolean;
}

/** An immutable lookup over registered actions. */
export interface ActionRegistry {
  get(id: string): ShellAction | undefined;
  /** All actions that are bound (INV-9) — the only ones any surface renders. */
  bound(): ShellAction[];
  has(id: string): boolean;
}

/** Build a registry from a list. A later entry with the same id overrides. */
export function createActionRegistry(actions: readonly ShellAction[]): ActionRegistry {
  const byId = new Map<string, ShellAction>();
  for (const a of actions) {
    byId.set(a.id, {
      bound: true,
      ...a,
    });
  }
  return {
    get: (id) => byId.get(id),
    has: (id) => byId.has(id),
    bound: () =>
      [
        ...byId.values(),
      ].filter((a) => a.bound !== false),
  };
}

/** Whether an action id resolves to a bound command (render gate, INV-9). */
export function isBound(registry: ActionRegistry, id: string): boolean {
  return registry.get(id)?.bound !== false && registry.has(id);
}
