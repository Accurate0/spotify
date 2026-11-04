// Detail pages keep the current filter query string so their stats match the page linked from.

export function itemPath(uri: string): string {
  return `/track/${encodeURIComponent(uri)}${window.location.search}`;
}

export function artistPath(name: string): string {
  return `/artist/${encodeURIComponent(name)}${window.location.search}`;
}
