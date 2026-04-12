const acceptPlain = { Accept: "text/plain" };

export function buildReleasesPath(includeRc: boolean): string {
  const q = includeRc ? "?rc=true" : "";
  return `/sls/releases${q}`;
}

export function buildModuleReleasesPath(
  moduleName: string,
  includeRc: boolean
): string {
  const enc = encodeURIComponent(moduleName);
  const q = includeRc ? "?rc=true" : "";
  return `/sls/releases/${enc}${q}`;
}

export async function fetchText(path: string): Promise<string> {
  const res = await fetch(path, { headers: acceptPlain });
  if (!res.ok) {
    throw new Error(
      res.status === 502
        ? "Upstream service unavailable (check GitHub token and backend logs)."
        : `Request failed (${res.status})`
    );
  }
  return res.text();
}
