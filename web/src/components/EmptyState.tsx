/** Shown on stats pages until an export has been imported. */
export default function EmptyState() {
  return (
    <div className="bg-card rounded-xl border px-6 py-10">
      <h2 className="font-display text-2xl font-semibold tracking-tight">No listening data yet</h2>
      <p className="text-muted-foreground mt-2 max-w-prose">
        Request your data from Spotify (Account privacy → Download your data), then import each
        account&apos;s export with the CLI. Import each person&apos;s folders into their own account.
      </p>
      <pre className="bg-muted mt-4 overflow-x-auto rounded-lg p-4 text-sm">
        {`ortzi install spotify
ortzi spotify accounts add me --name "Your name"
ortzi spotify import ~/Downloads/my_spotify_data --account me`}
      </pre>
    </div>
  );
}
