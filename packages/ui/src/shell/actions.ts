/**
 * The action registry — the command vocabulary (AS-6).
 *
 * A `ShellAction` is a named command independent of how it is triggered (a menu
 * leaf, a keybinding, a palette row, or code). A control invokes an action by id
 * and never holds the behaviour, so the same action is renderable in a menu, the
 * palette, and the keymap surface without bespoke wiring. Two gates decide
 * whether a surface may show an action: `bound` (is the capability shipped —
 * INV-9) and `when` (is it live in the current context — AS-7).
 *
 * This module is still narrow: it carries the vocabulary and the two gates. The
 * keystroke resolution that consumes `when` lives in `../shared/keymap`.
 */

import type { MessageKey } from "../shared/i18n";
import { always, type ContextPredicate, type ContextStack } from "../shared/keymap";

/** One registered command. */
export interface ShellAction {
  /** Stable namespaced id, e.g. `"file.settings"`. */
  id: string;
  /** i18n key for the user-visible label. Mandatory (AS-6) — an action is always describable. */
  labelKey: MessageKey;
  /** What the action does. Presentation-only callers pass a no-op or an intent. */
  run: () => void;
  /** Current keybinding, display-only (e.g. `"Ctrl ,"`). */
  binding?: string;
  /** Whether the action is bound to a shipped capability. An unbound action is
   *  hidden from every surface (INV-9) — never rendered as a dead control. */
  bound?: boolean;
  /** Where the action is live (AS-7). Absent means everywhere. A predicate that
   *  is false for the current context hides the action and drops its binding. */
  when?: ContextPredicate;
}

/** An immutable lookup over registered actions. */
export interface ActionRegistry {
  get(id: string): ShellAction | undefined;
  /** All actions that are bound (INV-9) — the only ones any surface may render. */
  bound(): ShellAction[];
  /** The bound actions whose `when` predicate holds over `stack` (AS-7). */
  live(stack: ContextStack): ShellAction[];
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
  const boundActions = () =>
    [
      ...byId.values(),
    ].filter((a) => a.bound !== false);
  return {
    get: (id) => byId.get(id),
    has: (id) => byId.has(id),
    bound: boundActions,
    live: (stack) => boundActions().filter((a) => (a.when ?? always)(stack)),
  };
}

/** Whether an action id resolves to a bound command (render gate, INV-9). */
export function isBound(registry: ActionRegistry, id: string): boolean {
  return registry.get(id)?.bound !== false && registry.has(id);
}
