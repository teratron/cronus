/**
 * L0 · Building frame — the application-global chrome.
 *
 * Presentation only: renders the title bar (app icon, burger menu, sidebar +
 * file-tree toggles, window controls) and the File/Edit/View/Help dropdowns.
 * Menu-open state and every toggle are caller-owned; a leaf click dispatches the
 * bound action and asks the caller to close the menu. Unbound leaves are already
 * removed by `visibleMenu` (INV-9), so nothing here renders a dead control.
 */

import { type Locale, translator } from "../i18n";
import type { ActionRegistry } from "./actions";
import { type MenuGroupId, visibleMenu } from "./menu";

export interface BuildingFrameProps {
  actions: ActionRegistry;
  /** Which menu group's dropdown is open, if any (caller-owned view state). */
  openGroup?: MenuGroupId | null;
  /** Toggle the burger menu / switch the open group. */
  onOpenGroup?: (group: MenuGroupId | null) => void;
  onToggleSidebar?: () => void;
  onToggleRightDock?: () => void;
  /** Whether the file-tree toggle is offered (a building-wide facility). */
  showRightDockToggle?: boolean;
  locale?: Locale;
}

export function BuildingFrame({
  actions,
  openGroup = null,
  onOpenGroup,
  onToggleSidebar,
  onToggleRightDock,
  showRightDockToggle = true,
  locale = "en",
}: BuildingFrameProps) {
  const msg = translator(locale);
  const groups = visibleMenu(actions);
  const burgerOpen = openGroup !== null;

  return (
    <div
      data-testid="building-frame"
      className="relative z-60 flex h-8.5 flex-none select-none items-center border-b border-border-subtle bg-surface-0"
    >
      <div className="flex w-52.5 flex-none items-center pl-3">
        <img
          src="assets/cronus-icon.png"
          alt={msg("app.title")}
          className="mr-3 h-3.75 w-3.75 rounded-sm"
        />
        <button
          type="button"
          data-testid="burger"
          aria-haspopup="menu"
          aria-expanded={burgerOpen}
          title={msg("frame.menu")}
          className="flex h-6 w-6.5 items-center justify-center rounded-sm text-text-secondary hover:bg-surface-2 hover:text-text-primary aria-expanded:bg-surface-2"
          onClick={() => onOpenGroup?.(burgerOpen ? null : "file")}
        >
          ≡
        </button>
        <div className="ml-auto flex items-center gap-0.5">
          <button
            type="button"
            data-testid="toggle-sidebar"
            title={msg("frame.toggle-sidebar")}
            className="flex h-6 w-6.5 items-center justify-center rounded-sm text-text-secondary hover:bg-surface-2 hover:text-text-primary"
            onClick={() => onToggleSidebar?.()}
          >
            ▮
          </button>
        </div>
      </div>

      <div className="h-full flex-1" />

      {showRightDockToggle ? (
        <div className="mr-1.5 flex items-center gap-0.5">
          <button
            type="button"
            data-testid="toggle-files"
            title={msg("frame.toggle-files")}
            className="flex h-6 w-6.5 items-center justify-center rounded-sm text-text-secondary hover:bg-surface-2 hover:text-text-primary"
            onClick={() => onToggleRightDock?.()}
          >
            ▤
          </button>
        </div>
      ) : null}

      <div className="flex h-full items-stretch">
        <button
          type="button"
          title={msg("frame.minimize")}
          className="w-11 border-none bg-transparent text-text-secondary hover:bg-surface-2"
        >
          —
        </button>
        <button
          type="button"
          title={msg("frame.maximize")}
          className="w-11 border-none bg-transparent text-text-secondary hover:bg-surface-2"
        >
          ▢
        </button>
        <button
          type="button"
          title={msg("frame.close")}
          className="w-11 border-none bg-transparent text-text-secondary hover:bg-danger hover:text-text-primary"
        >
          ✕
        </button>
      </div>

      {burgerOpen ? (
        <>
          <button
            type="button"
            aria-label="close menu"
            data-testid="menu-scrim"
            className="fixed inset-0 z-55 cursor-default border-none bg-transparent"
            onClick={() => onOpenGroup?.(null)}
          />
          <div
            data-testid="menu-bar"
            role="menubar"
            className="absolute top-9.5 left-2.5 z-58 flex min-w-47 flex-col gap-0.5 rounded-lg border border-border-strong bg-surface-2 p-1.5 shadow-overlay"
          >
            {groups.map((group) => (
              <div key={group.id} className="relative">
                <button
                  type="button"
                  role="menuitem"
                  data-testid={`menu-group-${group.id}`}
                  aria-expanded={openGroup === group.id}
                  className="flex w-full items-center rounded-sm px-2.5 py-1.5 text-left text-sm hover:bg-surface-3 aria-expanded:bg-surface-3"
                  onMouseEnter={() => onOpenGroup?.(group.id)}
                  onClick={() => onOpenGroup?.(group.id)}
                >
                  {msg(group.labelKey)}
                  <span className="ml-auto text-text-muted">›</span>
                </button>
                {openGroup === group.id ? (
                  <div
                    data-testid={`menu-submenu-${group.id}`}
                    role="menu"
                    className="absolute -top-1.5 left-full z-59 ml-1.5 flex min-w-55 flex-col gap-0.5 rounded-lg border border-border-strong bg-surface-2 p-1.5 shadow-overlay"
                  >
                    {group.leaves.map((leaf, i) =>
                      leaf === null ? (
                        <div
                          key={`sep-${group.id}-${i}`}
                          className="mx-1.5 my-1 h-px bg-border-subtle"
                        />
                      ) : (
                        <button
                          key={leaf.actionId}
                          type="button"
                          role="menuitem"
                          data-testid={`menu-leaf-${leaf.actionId}`}
                          className="flex items-center rounded-sm px-2.5 py-1.5 text-left text-sm hover:bg-surface-3"
                          onClick={() => {
                            actions.get(leaf.actionId)?.run();
                            onOpenGroup?.(null);
                          }}
                        >
                          {msg(leaf.labelKey)}
                          {leaf.binding ? (
                            <span className="ml-auto pl-6 text-xs text-text-muted">
                              {leaf.binding}
                            </span>
                          ) : null}
                        </button>
                      ),
                    )}
                  </div>
                ) : null}
              </div>
            ))}
          </div>
        </>
      ) : null}
    </div>
  );
}
