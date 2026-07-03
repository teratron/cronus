import { App, createCoreClient } from "@cronus/ui";
import "@cronus/ui/styles.css";
import { invoke } from "@tauri-apps/api/core";
import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";

// The UI package stays shell-agnostic; the desktop shell injects Tauri's invoke.
const core = createCoreClient(invoke);

function Root() {
  const [status, setStatus] = useState<string | undefined>(undefined);

  useEffect(() => {
    let mounted = true;
    core
      .status()
      .then((value) => {
        if (mounted) setStatus(value);
      })
      .catch(() => {
        if (mounted) setStatus("core unavailable");
      });
    return () => {
      mounted = false;
    };
  }, []);

  return <App status={status} />;
}

const container = document.getElementById("root");
if (container) {
  createRoot(container).render(
    <StrictMode>
      <Root />
    </StrictMode>,
  );
}
