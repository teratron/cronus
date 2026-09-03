import { describe, expect, it } from "vitest";
import * as ui from "./index";

// The package's public value surface, frozen. A deliberate public-API change
// updates this list in the same commit; anything else is a regression.
// Phase 26 T-26A01 dropped `App`, `Workbench`, `SURFACES` (46 -> 43): the two
// retired composer roots and the Phase-8 surface catalog they alone used —
// R-1, exactly one exported application root (`BuildingShell`).
const PUBLIC_API = [
  "BuildingFrame",
  "BuildingShell",
  "CANONICAL_TOKENS",
  "CommandPalette",
  "DEFAULT_LOCALE",
  "DEFAULT_SCHEME_ID",
  "DashboardPanel",
  "FloorTabBar",
  "GlobalSettingsOverlay",
  "L3_FACETS",
  "MENU",
  "MechanismNav",
  "NAV_LAYERS",
  "OfficeViewPanel",
  "RightDock",
  "SIDEBAR_PRIMARY",
  "SIDEBAR_TABS",
  "SIDEBAR_UTILITY",
  "SelectionSurface",
  "SubsystemSidebar",
  "SurfacePlaceholder",
  "SurfaceRouter",
  "commandPaletteDelegate",
  "composeSidebar",
  "createActionRegistry",
  "createCoreClient",
  "hasMechanismNav",
  "isBound",
  "isCanonicalOrder",
  "isChildLayer",
  "isClosable",
  "isUnloadable",
  "registerScheme",
  "resolveScheme",
  "resolveTheme",
  "schemeCatalog",
  "settingsTier",
  "shouldLoad",
  "surfaceAttributes",
  "t",
  "themeAttributes",
  "translator",
  "visibleMenu",
];

describe("public API freeze", () => {
  it("exports exactly the frozen symbol set", () => {
    expect(Object.keys(ui).sort()).toEqual(PUBLIC_API);
  });
});
