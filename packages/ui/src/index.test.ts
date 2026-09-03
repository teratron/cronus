import { describe, expect, it } from "vitest";
import * as ui from "./index";

// The package's public value surface, frozen. The four-tier relocation must not
// add, drop, or rename a single export — this list is the neutrality proof.
// A deliberate public-API change updates this list in the same commit.
const PUBLIC_API = [
  "App",
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
  "SURFACES",
  "SelectionSurface",
  "SubsystemSidebar",
  "SurfacePlaceholder",
  "SurfaceRouter",
  "Workbench",
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
