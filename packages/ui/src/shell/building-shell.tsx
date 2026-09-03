/**
 * The four-layer building shell — composes L0 frame, L1 floor tabs, L2 sidebar,
 * L3 mechanism strip, the surface router, and the floating overlays (command
 * palette, right file-tree dock, global settings).
 *
 * Presentation only. All domain state (floors, office state, badge counts, the
 * file tree, recent offices) arrives as props from the hosting shell over the
 * IPC bridge; this component holds only view state (which floor/subsystem/facet
 * is active, which overlays are open) and forwards every mutation as an intent.
 *
 * Theming: the two axes (mode × scheme) are applied on the root via
 * `surfaceAttributes` — `data-theme` + `data-scheme` + the Tailwind `dark` class.
 * Switching either is a cosmetic attribute swap; the tree never unmounts.
 */

import { useMemo, useState } from "react";
import { type Locale, translator } from "../shared/i18n";
import type { SidebarTab } from "../shared/navigation";
import { surfaceAttributes, type Theme } from "../shared/theme";
import type { DashboardProjection, OfficeProjection } from "../surfaces";
import { type ActionRegistry, createActionRegistry, type ShellAction } from "./actions";
import { BuildingFrame } from "./building-frame";
import { CommandPalette } from "./command-palette";
import { type FloorTab, FloorTabBar } from "./floor-tab-bar";
import { GlobalSettingsOverlay } from "./global-settings-overlay";
import { MechanismNav } from "./mechanism-nav";
import type { MenuGroupId } from "./menu";
import { type FileNode, RightDock } from "./right-dock";
import { SubsystemSidebar } from "./subsystem-sidebar";
import { SurfaceRouter } from "./surface-router";

export interface BuildingShellProps {
  // theming (mode × scheme)
  theme?: Theme;
  colorScheme?: string;
  systemPrefersDark?: boolean;
  onThemeChange?: (theme: Theme) => void;
  onColorSchemeChange?: (id: string) => void;
  schemes?: readonly {
    id: string;
    name: string;
  }[];
  // navigation state (caller-owned)
  floors: readonly FloorTab[];
  activeFloorId: string;
  activeSubsystem?: SidebarTab;
  onSelectFloor?: (id: string) => void;
  onCreateFloor?: () => void;
  onFloorMenu?: (id: string) => void;
  onSelectSubsystem?: (tab: SidebarTab) => void;
  pinned?: readonly SidebarTab[];
  badges?: Partial<Record<SidebarTab, number>>;
  floorName?: string;
  floorSlug?: string;
  // L0 facilities
  actions?: readonly ShellAction[];
  fileTree?: readonly FileNode[];
  recentOffices?: readonly {
    id: string;
    name: string;
    hint?: string;
  }[];
  // surfaces
  office?: OfficeProjection;
  dashboard?: DashboardProjection;
  locale?: Locale;
}

export function BuildingShell({
  theme = "system",
  colorScheme = "default",
  systemPrefersDark = true,
  onThemeChange,
  onColorSchemeChange,
  schemes,
  floors,
  activeFloorId,
  activeSubsystem = "dashboard",
  onSelectFloor,
  onCreateFloor,
  onFloorMenu,
  onSelectSubsystem,
  pinned = [],
  badges = {},
  floorName,
  floorSlug,
  actions = [],
  fileTree = [],
  recentOffices = [],
  office,
  dashboard,
  locale = "en",
}: BuildingShellProps) {
  const msg = translator(locale);
  const surface = surfaceAttributes(theme, colorScheme, systemPrefersDark);

  // view state only
  const [openGroup, setOpenGroup] = useState<MenuGroupId | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [rightDockOpen, setRightDockOpen] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [activeFacet, setActiveFacet] = useState<string | undefined>(undefined);

  const registry: ActionRegistry = useMemo(() => {
    const openSettings: ShellAction = {
      id: "file.settings",
      labelKey: "menu.file.settings",
      binding: "Ctrl ,",
      run: () => setSettingsOpen(true),
    };
    const newProject: ShellAction = {
      id: "file.new-project",
      labelKey: "menu.file.new-project",
      binding: "Ctrl N",
      run: () => onCreateFloor?.(),
    };
    return createActionRegistry([
      openSettings,
      newProject,
      ...actions,
    ]);
  }, [
    actions,
    onCreateFloor,
  ]);

  const paletteActions = registry.bound().map((a) => ({
    id: a.id,
    label: msg(a.labelKey),
    binding: a.binding,
    run: a.run,
  }));

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: shell-level shortcut listener; the palette is also reachable from the sidebar search button
    <div
      data-testid="building-shell"
      data-theme={surface["data-theme"]}
      data-scheme={surface["data-scheme"]}
      className={`relative flex h-screen flex-col bg-surface-0 text-text-primary ${surface.className}`}
      onKeyDown={(e) => {
        if (e.ctrlKey && e.shiftKey && (e.key === "J" || e.key === "j")) {
          e.preventDefault();
          setPaletteOpen(true);
        }
      }}
    >
      <BuildingFrame
        actions={registry}
        openGroup={openGroup}
        onOpenGroup={setOpenGroup}
        onToggleSidebar={() => setSidebarOpen((v) => !v)}
        onToggleRightDock={() => setRightDockOpen((v) => !v)}
        locale={locale}
      />

      <FloorTabBar
        floors={floors}
        activeFloorId={activeFloorId}
        onSelectFloor={onSelectFloor}
        onCreateFloor={onCreateFloor}
        onFloorMenu={onFloorMenu}
        locale={locale}
      />

      <div className="flex min-h-0 flex-1">
        {sidebarOpen ? (
          <SubsystemSidebar
            active={activeSubsystem}
            onSelect={(tab) => {
              setActiveFacet(undefined);
              onSelectSubsystem?.(tab);
            }}
            pinned={pinned}
            badges={badges}
            floorName={floorName}
            floorSlug={floorSlug}
            onOpenSearch={() => setPaletteOpen(true)}
            locale={locale}
          />
        ) : null}

        <div className="flex min-w-0 flex-1 flex-col">
          <div className="flex h-13.25 flex-none items-center gap-2.5 border-b border-border-subtle px-4">
            <span className="text-sm font-semibold text-text-primary">
              {msg(`nav.${activeSubsystem}` as const)}
            </span>
            <div className="flex-1" />
            <MechanismNav
              subsystem={activeSubsystem}
              activeFacet={activeFacet}
              onSelectFacet={setActiveFacet}
              locale={locale}
            />
          </div>
          <SurfaceRouter
            active={activeSubsystem}
            office={office}
            dashboard={dashboard}
            locale={locale}
          />
        </div>

        <RightDock open={rightDockOpen} floorName={floorName} tree={fileTree} locale={locale} />
      </div>

      <CommandPalette
        open={paletteOpen}
        onClose={() => setPaletteOpen(false)}
        recentOffices={recentOffices}
        onGoToOffice={(id) => {
          onSelectFloor?.(id);
          setPaletteOpen(false);
        }}
        onGoToSubsystem={(tab) => {
          onSelectSubsystem?.(tab);
          setPaletteOpen(false);
        }}
        onOpenSettings={() => {
          setSettingsOpen(true);
          setPaletteOpen(false);
        }}
        actions={paletteActions}
        locale={locale}
      />

      <GlobalSettingsOverlay
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        theme={theme}
        onThemeChange={onThemeChange}
        colorScheme={colorScheme}
        onColorSchemeChange={onColorSchemeChange}
        schemes={schemes}
        locale={locale}
      />
    </div>
  );
}
