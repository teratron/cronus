/**
 * The graphical shell's surface catalog — static presentation data.
 *
 * This is the original surface set that the `Workbench` composer routes over.
 * It is catalog data, not a component, so it lives in the leaf tier alongside
 * the sidebar catalog. Keeping it here is what lets the composer (shell tier)
 * and any surface reference the same identifiers without a lateral edge.
 */

import type { MessageKey } from "./i18n";

/** The five surfaces of the graphical shell. */
export type SurfaceId = "office" | "board" | "chat" | "editor" | "dashboard";

export const SURFACES: SurfaceId[] = [
  "office",
  "board",
  "chat",
  "editor",
  "dashboard",
];

/** i18n key per surface — the label shown in the workbench nav strip. */
export const SURFACE_LABEL: Record<SurfaceId, MessageKey> = {
  office: "surface.office",
  board: "surface.board",
  chat: "surface.chat",
  editor: "surface.editor",
  dashboard: "surface.dashboard",
};
