const acceptPlain = { Accept: "text/plain" };
const acceptJson = { Accept: "application/json" };

type VersionJson =
  | { Release: { major: number; minor: number; patch: number } }
  | {
      Candidate: { major: number; minor: number; patch: number; number: number };
    };

function versionToString(v: VersionJson): string {
  if ("Release" in v) {
    const { major, minor, patch } = v.Release;
    return `${major}.${minor}.${patch}`;
  }
  const { major, minor, patch, number } = v.Candidate;
  return `${major}.${minor}.${patch}-RC${number}`;
}

export type ReleaseRow = {
  name: string;
  localizedName: string;
  version: string;
  url: string;
};

export type ModuleReleaseRow = {
  version: string;
  dateTime: string;
  url: string;
};

type ReleaseJson = {
  name: string;
  localized_name: string;
  version: VersionJson;
  url: string;
  date_time: string;
};

type ModuleReleaseJson = {
  version: VersionJson;
  url: string;
  date_time: string;
};

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

export async function fetchJson<T>(path: string): Promise<T> {
  const res = await fetch(path, { headers: acceptJson });
  if (!res.ok) {
    throw new Error(
      res.status === 502
        ? "Upstream service unavailable (check GitHub token and backend logs)."
        : `Request failed (${res.status})`
    );
  }
  return res.json() as Promise<T>;
}

export async function fetchReleases(includeRc: boolean): Promise<ReleaseRow[]> {
  const data = await fetchJson<ReleaseJson[]>(buildReleasesPath(includeRc));
  return data.map((r) => ({
    name: r.name,
    localizedName: r.localized_name,
    version: versionToString(r.version),
    url: r.url,
  }));
}

export async function fetchModuleReleases(
  moduleName: string,
  includeRc: boolean
): Promise<ModuleReleaseRow[]> {
  const data = await fetchJson<ModuleReleaseJson[]>(
    buildModuleReleasesPath(moduleName, includeRc)
  );
  return data.map((r) => ({
    version: versionToString(r.version),
    dateTime: r.date_time,
    url: r.url,
  }));
}
