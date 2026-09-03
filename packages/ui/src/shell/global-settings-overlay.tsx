/**
 * L0 · Global settings — a full-screen overlay above the workbench.
 *
 * Opened from File ▸ Settings and from the Settings tab's Global tier. It has
 * its own title bar, a left settings nav, and a scrolled content pane. Closing
 * returns to the prior surface without disturbing floor / subsystem state
 * (the caller keeps that state; this component only toggles its own visibility).
 *
 * The Appearance section hosts the two theming-axis pickers (mode × scheme).
 */

import { type Locale, translator } from "../shared/i18n";
import type { Theme } from "../shared/theme";

export interface GlobalSettingsOverlayProps {
  open: boolean;
  onClose?: () => void;
  /** Current mode-axis value. */
  theme: Theme;
  onThemeChange?: (theme: Theme) => void;
  /** Current colour-scheme-axis value. */
  colorScheme: string;
  onColorSchemeChange?: (id: string) => void;
  /** Available scheme ids (built-in + registered). */
  schemes?: readonly {
    id: string;
    name: string;
  }[];
  locale?: Locale;
}

const MODES: readonly Theme[] = [
  "system",
  "light",
  "dark",
];

export function GlobalSettingsOverlay({
  open,
  onClose,
  theme,
  onThemeChange,
  colorScheme,
  onColorSchemeChange,
  schemes = [
    {
      id: "default",
      name: "Cronus",
    },
  ],
  locale = "en",
}: GlobalSettingsOverlayProps) {
  const msg = translator(locale);
  if (!open) return null;

  return (
    <div
      data-testid="global-settings-overlay"
      className="absolute inset-0 z-120 flex flex-col bg-surface-0"
    >
      <div className="flex h-8.5 flex-none items-center border-b border-border-subtle px-2.5">
        <img src="assets/cronus-icon.png" alt="" className="h-3.75 w-3.75 rounded-sm" />
        <span className="ml-2.5 text-xs text-text-muted">{msg("settings.title")}</span>
        <div className="flex-1" />
        <button
          type="button"
          data-testid="settings-close"
          title={msg("frame.close")}
          className="flex h-full w-11 items-center justify-center border-none bg-transparent text-text-secondary hover:bg-danger hover:text-text-primary"
          onClick={() => onClose?.()}
        >
          ✕
        </button>
      </div>

      <div className="flex min-h-0 flex-1">
        <div className="flex w-66 flex-none flex-col overflow-y-auto border-r border-border-subtle p-3">
          <button
            type="button"
            data-testid="settings-back"
            className="flex items-center gap-2.5 rounded-md px-2.5 py-2 text-left text-sm text-text-secondary hover:bg-surface-1 hover:text-text-primary"
            onClick={() => onClose?.()}
          >
            ‹ {msg("settings.back")}
          </button>
          <div className="px-2.5 pt-4 pb-1.5 text-xs font-semibold tracking-wide text-text-muted">
            {msg("settings.tier.global")}
          </div>
          <a
            href="#appearance"
            className="rounded-md bg-surface-2 px-2.5 py-2 text-sm text-text-primary no-underline"
          >
            {msg("settings.appearance")}
          </a>
        </div>

        <div className="min-w-0 flex-1 overflow-y-auto p-10">
          <section id="appearance" className="max-w-180">
            <h1 className="mb-1.5 text-2xl font-semibold text-text-primary">
              {msg("settings.appearance")}
            </h1>
            <div className="mt-6 rounded-lg border border-border-subtle bg-surface-1 p-4">
              <div className="flex items-center justify-between border-b border-border-subtle py-4">
                <span className="text-sm text-text-primary">{msg("settings.appearance.mode")}</span>
                <div
                  data-testid="mode-picker"
                  className="flex gap-1 rounded-md border border-border-subtle bg-surface-0 p-0.5"
                >
                  {MODES.map((mode) => (
                    <button
                      key={mode}
                      type="button"
                      data-testid={`mode-${mode}`}
                      aria-pressed={theme === mode}
                      className="rounded-sm px-3 py-1 text-xs text-text-secondary aria-pressed:bg-surface-3 aria-pressed:text-text-primary"
                      onClick={() => onThemeChange?.(mode)}
                    >
                      {msg(`settings.mode.${mode}`)}
                    </button>
                  ))}
                </div>
              </div>
              <div className="flex items-center justify-between py-4">
                <span className="text-sm text-text-primary">
                  {msg("settings.appearance.scheme")}
                </span>
                <select
                  data-testid="scheme-picker"
                  value={colorScheme}
                  className="rounded-md border border-border-subtle bg-surface-0 px-3 py-1 text-xs text-text-secondary outline-none"
                  onChange={(e) => onColorSchemeChange?.(e.target.value)}
                >
                  {schemes.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.name}
                    </option>
                  ))}
                </select>
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
