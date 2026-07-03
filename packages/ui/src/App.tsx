/**
 * Root application surface.
 *
 * Presentation only: renders from props and holds no business logic — all
 * domain state arrives from the core over the shell bridge. The only local
 * state is view state (which surface is active); everything visible resolves
 * through the i18n catalog and the theme tokens.
 */

import { useState } from "react";
import type { Locale } from "./i18n";
import { type SurfaceId, Workbench } from "./surfaces";
import type { Theme } from "./theme";

export interface AppProps {
  /** Core status line, supplied by the shell bridge. */
  status?: string;
  /** Active locale; defaults to English. */
  locale?: Locale;
  /** Theme choice; `system` follows the OS preference. */
  theme?: Theme;
  /** OS dark-mode preference, injected by the hosting shell. */
  systemPrefersDark?: boolean;
}

export function App({
  status,
  locale = "en",
  theme = "system",
  systemPrefersDark,
}: AppProps) {
  // View state only: which surface the user is looking at.
  const [active, setActive] = useState<SurfaceId>("office");

  return (
    <Workbench
      active={active}
      onSelect={setActive}
      status={status}
      locale={locale}
      theme={theme}
      systemPrefersDark={systemPrefersDark}
    />
  );
}
