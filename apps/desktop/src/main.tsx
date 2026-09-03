import {
  BuildingShell,
  createCoreClient,
  type FloorTab,
  schemeCatalog,
  type Theme,
} from "@cronus/ui";
import "@cronus/ui/styles.css";
import { invoke } from "@tauri-apps/api/core";
import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";

// The UI package stays shell-agnostic; the desktop shell injects Tauri's invoke.
const core = createCoreClient(invoke);

// The registered colour schemes, mapped to the picker's shape.
const SCHEMES = schemeCatalog().map((m) => ({
  id: m.id,
  name: m.name,
}));

// Until the core exposes a floor/office projection over IPC, the shell is
// mounted with only the pinned Home floor. Every subsystem surface renders as
// an explicit placeholder (INV-9) rather than fabricated data.
const HOME: FloorTab = {
  id: "home",
  name: "Home",
  kind: "home",
  state: "idle",
};

function Root() {
  // View state the shell does not own itself: the two theming axes. These will
  // move to core-persisted settings once an IPC command exposes them.
  const [theme, setTheme] = useState<Theme>("system");
  const [colorScheme, setColorScheme] = useState("default");
  const [systemPrefersDark] = useState(
    () =>
      typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches,
  );

  useEffect(() => {
    // The bridge is live; nothing consumes it yet. Surface it in the console so
    // the wiring is verifiable without a fake status bar.
    core
      .status()
      .then((s) => console.info("[cronus] core:", s))
      .catch(() => console.info("[cronus] core unavailable"));
  }, []);

  return (
    <BuildingShell
      theme={theme}
      colorScheme={colorScheme}
      systemPrefersDark={systemPrefersDark}
      onThemeChange={setTheme}
      onColorSchemeChange={setColorScheme}
      schemes={SCHEMES}
      floors={[
        HOME,
      ]}
      activeFloorId="home"
      locale="en"
    />
  );
}

const container = document.getElementById("root");
if (container) {
  createRoot(container).render(
    <StrictMode>
      <Root />
    </StrictMode>,
  );
}
