/**
 * Typed client for the shell ↔ core IPC bridge.
 *
 * Pure marshalling: each method forwards to one shell IPC command that binds a
 * core capability (the same surface the CLI/TUI bind). The invoke function is
 * injected by the hosting shell, so this package stays shell-agnostic and
 * testable without a Tauri runtime. No business logic lives here.
 */

/** Shape of the shell's IPC invoke function (injected by the host app). */
export type InvokeFn = <T>(
  command: string,
  args?: Record<string, unknown>,
) => Promise<T>;

/** Typed view of the core capability surface exposed over IPC. */
export interface CoreClient {
  /** Core/product version string. */
  version(): Promise<string>;
  /** Human-readable core status line (already masked by the core). */
  status(): Promise<string>;
}

/** Wrap a shell invoke function into the typed core client. */
export function createCoreClient(invoke: InvokeFn): CoreClient {
  return {
    version: () => invoke<string>("capability_version"),
    status: () => invoke<string>("capability_status"),
  };
}
