/**
 * The shell's icon set — transcribed from the desktop design, one entry per
 * glyph it actually uses.
 *
 * A single registry rather than a component per glyph: the frame draws ~35 of
 * these and the only thing that varies between them is the path data, so a
 * component apiece would be 35 near-identical wrappers. Every icon inherits
 * `currentColor`, so colour is the caller's (and therefore the token layer's).
 *
 * Stroke icons share a 24×24 box and a 1.6 default weight; the few solid glyphs
 * (play/pause/stop, the overflow dots) declare `filled`.
 */

import type { JSX } from "react";

interface Glyph {
  /** Path/shape markup for the 24×24 box. */
  d: JSX.Element;
  /** Solid rather than stroked. */
  filled?: boolean;
  /** Per-glyph stroke weight where the design deviates from the 1.6 default. */
  weight?: number;
}

const G = {
  burger: {
    d: <path d="M4 7h16M4 12h16M4 17h16" />,
    weight: 1.7,
  },
  panelLeft: {
    d: (
      <>
        <rect x="3" y="4.5" width="18" height="15" rx="2.5" />
        <path d="M9.5 4.5v15" />
      </>
    ),
  },
  panelRight: {
    d: (
      <>
        <rect x="3" y="4" width="18" height="16" rx="2" />
        <path d="M15 4v16" />
      </>
    ),
  },
  chevronLeft: {
    d: <path d="M14.5 6 8.5 12l6 6" />,
    weight: 1.8,
  },
  chevronRight: {
    d: <path d="m9.5 6 6 6-6 6" />,
    weight: 1.8,
  },
  caret: {
    d: <path d="m9 6 6 6-6 6" />,
    weight: 1.7,
  },
  minimize: {
    d: <path d="M5 12.5h14" />,
    weight: 1.8,
  },
  maximize: {
    d: <rect x="5" y="5" width="14" height="14" rx="1.5" />,
    weight: 1.8,
  },
  close: {
    d: <path d="m6 6 12 12M18 6 6 18" />,
    weight: 1.8,
  },
  home: {
    d: (
      <>
        <path d="M4 11.5 12 4l8 7.5" />
        <path d="M6 10v9h12v-9" />
      </>
    ),
    weight: 1.7,
  },
  dots: {
    d: (
      <>
        <circle cx="12" cy="5" r="1.5" />
        <circle cx="12" cy="12" r="1.5" />
        <circle cx="12" cy="19" r="1.5" />
      </>
    ),
    filled: true,
  },
  plus: {
    d: <path d="M12 5v14M5 12h14" />,
    weight: 1.7,
  },
  search: {
    d: (
      <>
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-3.2-3.2" />
      </>
    ),
    weight: 1.8,
  },
  dashboard: {
    d: (
      <>
        <rect x="3.5" y="3.5" width="7" height="8" rx="1.3" />
        <rect x="13.5" y="3.5" width="7" height="5" rx="1.3" />
        <rect x="13.5" y="11.5" width="7" height="9" rx="1.3" />
        <rect x="3.5" y="14.5" width="7" height="6" rx="1.3" />
      </>
    ),
  },
  chat: {
    d: (
      <path d="M4 6.5A2.5 2.5 0 0 1 6.5 4h11A2.5 2.5 0 0 1 20 6.5v7a2.5 2.5 0 0 1-2.5 2.5H10l-4.5 4v-4A2.5 2.5 0 0 1 4 13.5Z" />
    ),
  },
  activity: {
    d: <path d="M3 12h4l2-5 4 10 2-5h6" />,
  },
  pulse: {
    d: <path d="M3 12h4l2-6 4 12 2-6h6" />,
  },
  inbox: {
    d: (
      <>
        <path d="M4 13h4l2 3h4l2-3h4" />
        <path d="M4 13 6.5 5.5A2 2 0 0 1 8.4 4h7.2a2 2 0 0 1 1.9 1.5L20 13v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2Z" />
      </>
    ),
  },
  office: {
    d: (
      <>
        <rect x="5" y="3.5" width="14" height="17" rx="1.5" />
        <path d="M9 7.5h2m4 0h-2M9 11.5h2m2 0h2M10 20.5v-2h4v2" />
      </>
    ),
  },
  employees: {
    d: (
      <>
        <circle cx="9" cy="8" r="3" />
        <path d="M3.5 20c0-3 2.7-5 5.5-5s5.5 2 5.5 5" />
        <path d="M16 5.6a3 3 0 0 1 0 5.8M17.5 15.2c2.3.5 4 2.2 4 4.8" />
      </>
    ),
  },
  schedule: {
    d: (
      <>
        <rect x="4" y="5" width="16" height="15" rx="2" />
        <path d="M4 9h16M8 3v4m8-4v4" />
        <path d="M12 12.5v3l2 1" strokeWidth="1.4" />
      </>
    ),
  },
  calendar: {
    d: (
      <>
        <rect x="4" y="5" width="16" height="15" rx="2" />
        <path d="M4 9h16M8 3v4m8-4v4" />
      </>
    ),
    weight: 1.7,
  },
  kanban: {
    d: (
      <>
        <rect x="3.5" y="4" width="5" height="16" rx="1.2" />
        <rect x="9.8" y="4" width="5" height="10" rx="1.2" />
        <rect x="16" y="4" width="5" height="13" rx="1.2" />
      </>
    ),
  },
  automation: {
    d: (
      <>
        <circle cx="6" cy="6" r="2.4" />
        <circle cx="6" cy="18" r="2.4" />
        <circle cx="18" cy="12" r="2.4" />
        <path d="M8.3 7.1 15.6 11M8.3 16.9 15.6 13" />
      </>
    ),
  },
  memory: {
    d: (
      <>
        <path d="M9 5a3 3 0 0 0-3 3 3 3 0 0 0-1 5.5A3 3 0 0 0 8 19a2.5 2.5 0 0 0 4-1V6.5A2.5 2.5 0 0 0 9 5Z" />
        <path d="M15 5a3 3 0 0 1 3 3 3 3 0 0 1 1 5.5A3 3 0 0 1 16 19a2.5 2.5 0 0 1-4-1" />
      </>
    ),
  },
  wiki: {
    d: (
      <>
        <path d="M5 5a2 2 0 0 1 2-2h11v16H7a2 2 0 0 0-2 2Z" />
        <path d="M5 19V5" />
        <path d="M9 7h6M9 10h6" strokeWidth="1.4" />
      </>
    ),
  },
  channels: {
    d: <path d="M10 4 8 20M16 4l-2 16M5 9h15M4 15h15" />,
  },
  security: {
    d: (
      <>
        <path d="M12 3.5 19 6v6c0 4.5-3 7-7 8.5C8 19 5 16.5 5 12V6Z" />
        <path d="m9 12 2 2 4-4" strokeWidth="1.4" />
      </>
    ),
  },
  settings: {
    d: (
      <>
        <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
        <circle cx="12" cy="12" r="3" />
      </>
    ),
  },
  help: {
    d: (
      <>
        <circle cx="12" cy="12" r="8.5" />
        <path d="M9.8 9.5a2.2 2.2 0 1 1 3.2 2c-.8.5-1 1-1 1.8" />
        <circle cx="12" cy="16.5" r="0.5" fill="currentColor" />
      </>
    ),
  },
  clock: {
    d: (
      <>
        <circle cx="12" cy="12" r="8.5" />
        <path d="M12 7.5V12l3 1.8" />
      </>
    ),
  },
  check: {
    d: <path d="m5 12 5 5 9-10" />,
    weight: 1.8,
  },
  filter: {
    d: <path d="M4 6h16M7 12h10m-7 6h4" />,
    weight: 1.7,
  },
  list: {
    d: <path d="M4 6h16M4 12h16M4 18h10" />,
    weight: 1.7,
  },
  refresh: {
    d: (
      <>
        <path d="M21 12a9 9 0 1 1-2.64-6.36" />
        <path d="M21 4v4.5h-4.5" />
      </>
    ),
    weight: 1.7,
  },
  archive: {
    d: (
      <>
        <rect x="3" y="4" width="18" height="4" rx="1" />
        <path d="M5 8v11a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V8" />
        <path d="M10 12h4" />
      </>
    ),
  },
  save: {
    d: (
      <>
        <path d="M5 4h11l3 3v13a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1z" />
        <path d="M8 4v5h7" />
        <rect x="8" y="13" width="8" height="6" rx="1" />
      </>
    ),
  },
  bolt: {
    d: <path d="M13 2 4 14h6l-1 8 9-12h-6z" />,
    filled: true,
  },
  play: {
    d: <path d="M8 5v14l11-7z" />,
    filled: true,
  },
  pause: {
    d: (
      <>
        <rect x="6" y="5" width="4" height="14" rx="1" />
        <rect x="14" y="5" width="4" height="14" rx="1" />
      </>
    ),
    filled: true,
  },
  stop: {
    d: <rect x="6" y="6" width="12" height="12" rx="2" />,
    filled: true,
  },
} satisfies Record<string, Glyph>;

/** Every glyph the shell can draw. */
export type IconName = keyof typeof G;

export interface IconProps {
  name: IconName;
  /** Rendered box, px. The design ranges 11–15. */
  size?: number;
  /** Override the glyph's stroke weight (ignored for solid glyphs). */
  strokeWidth?: number;
  className?: string;
}

export function Icon({ name, size = 15, strokeWidth, className }: IconProps) {
  const glyph: Glyph = G[name];
  if (glyph.filled) {
    return (
      <svg
        aria-hidden="true"
        focusable="false"
        width={size}
        height={size}
        viewBox="0 0 24 24"
        className={className}
        fill="currentColor"
      >
        {glyph.d}
      </svg>
    );
  }
  return (
    <svg
      aria-hidden="true"
      focusable="false"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth ?? glyph.weight ?? 1.6}
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {glyph.d}
    </svg>
  );
}
