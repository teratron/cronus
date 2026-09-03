/**
 * Typed client for the shell ↔ core IPC bridge.
 *
 * Pure marshalling: each method forwards to one shell IPC command that binds a
 * core capability (the same surface the CLI/TUI bind), or opens one push channel
 * the core emits on. The `invoke` and `listen` functions are injected by the
 * hosting shell, so this package stays shell-agnostic and testable without a
 * Tauri runtime. No business logic lives here.
 */

/** Shape of the shell's IPC invoke function (injected by the host app). */
export type InvokeFn = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

/**
 * Shape of the host's event-listen function (injected alongside `invoke`).
 * Resolves to a function that detaches the listener; rejects if the channel
 * cannot be opened.
 */
export type ListenFn = <T>(
  channel: string,
  handler: (event: { payload: T }) => void,
) => Promise<() => void>;

/**
 * What the core emits on a push channel: a message, or a one-shot close the
 * host frames itself. After `closed` no further messages arrive on that
 * subscription — the caller re-subscribes to resume, which is a fresh request,
 * never a mid-stream continuation.
 */
export type ChannelEvent<T> =
  | {
      type: "message";
      data: T;
    }
  | {
      type: "closed";
      reason: string;
    };

/**
 * The shell-facing slice of the host's settings store. Host-owned configuration
 * the shell reads and writes as marshalling, not logic (admission rule §4.3,
 * 1.0.1 — host-owned facility). `layout` is opaque here: the frontend owns the
 * `LayoutRecord` schema and its field-wise restore.
 */
export interface ShellSettings {
  theme: string;
  colorScheme: string;
  layout: unknown;
  keymapUser: Record<string, string>;
}

/** Typed view of the core capability surface exposed over IPC. */
export interface CoreClient {
  /** Core/product version string. */
  version(): Promise<string>;
  /** Human-readable core status line (already masked by the core). */
  status(): Promise<string>;
  /**
   * Open a push channel (AS-3). `onMessage` gets each payload; `onClose` fires
   * exactly once — if the channel fails to open, or the host reports it closed —
   * after which the subscription is dead. Returns a function that detaches it
   * (AS-4). Never retries: reconnection follows the host's connection lifecycle,
   * not a frontend timer.
   */
  subscribe<T>(
    channel: string,
    onMessage: (payload: T) => void,
    onClose?: (reason: string) => void,
  ): () => void;
  /** Host-owned settings the shell persists through (AS-12). */
  settings: {
    /** Read the current shell-facing settings slice. */
    get(): Promise<ShellSettings>;
    /** Write a partial update; only the given fields change. */
    set(patch: Partial<ShellSettings>): Promise<void>;
  };
}

/** Wrap a shell invoke (and optional listen) function into the typed core client. */
export function createCoreClient(invoke: InvokeFn, listen?: ListenFn): CoreClient {
  return {
    version: () => invoke<string>("capability_version"),
    status: () => invoke<string>("capability_status"),
    settings: {
      get: () => invoke<ShellSettings>("capability_settings_get"),
      set: (patch) =>
        invoke<void>("capability_settings_set", {
          patch,
        }),
    },
    subscribe: <T>(
      channel: string,
      onMessage: (payload: T) => void,
      onClose?: (reason: string) => void,
    ) => {
      let live = true;
      let detach: (() => void) | null = null;

      const close = (reason: string) => {
        if (!live) {
          return;
        }
        live = false;
        detach?.();
        detach = null;
        onClose?.(reason);
      };

      if (!listen) {
        // No event transport on this host — the channel cannot open.
        queueMicrotask(() => close("no event transport"));
        return () => {
          live = false;
        };
      }

      listen<ChannelEvent<T>>(channel, ({ payload }) => {
        if (!live) {
          return;
        }
        if (payload.type === "closed") {
          close(payload.reason);
        } else {
          onMessage(payload.data);
        }
      }).then(
        (unlisten) => {
          if (live) {
            detach = unlisten;
          } else {
            unlisten();
          }
        },
        (error: unknown) => {
          close(error instanceof Error ? error.message : "channel failed to open");
        },
      );

      return () => {
        live = false;
        detach?.();
        detach = null;
      };
    },
  };
}
