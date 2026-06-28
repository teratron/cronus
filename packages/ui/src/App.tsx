/**
 * Root application surface.
 *
 * Presentation only: it renders from props and holds no business logic — all
 * domain state arrives from the core over the shell bridge (wired in a later
 * task). This is the scaffold root the desktop shell mounts.
 */
export interface AppProps {
  /** Core status line, supplied by the shell bridge. */
  status?: string;
}

export function App({ status }: AppProps) {
  return (
    <main className="flex h-screen flex-col items-center justify-center gap-2 bg-neutral-950 text-neutral-100">
      <h1 className="text-2xl font-semibold">Cronus</h1>
      <p className="text-sm text-neutral-400" data-testid="status">
        {status ?? "connecting…"}
      </p>
    </main>
  );
}
