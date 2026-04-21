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
  kind: string;
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
  kind: string;
  version: VersionJson;
  url: string;
  date_time: string;
};

type ModuleReleaseJson = {
  version: VersionJson;
  url: string;
  date_time: string;
};

export function buildReleasesPath(
  includeRc: boolean,
  includeMilestones: boolean
): string {
  const params = new URLSearchParams();
  if (includeRc) params.set("rc", "true");
  if (includeMilestones) params.set("ms", "true");
  const q = params.toString();
  return q ? `/sls/releases?${q}` : "/sls/releases";
}

export function buildModuleReleasesPath(
  moduleName: string,
  includeRc: boolean,
  includeMilestones: boolean
): string {
  const enc = encodeURIComponent(moduleName);
  const params = new URLSearchParams();
  if (includeRc) params.set("rc", "true");
  if (includeMilestones) params.set("ms", "true");
  const q = params.toString();
  return q ? `/sls/releases/${enc}?${q}` : `/sls/releases/${enc}`;
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

export async function fetchReleases(
  includeRc: boolean,
  includeMilestones: boolean
): Promise<ReleaseRow[]> {
  const data = await fetchJson<ReleaseJson[]>(
    buildReleasesPath(includeRc, includeMilestones)
  );
  return data.map((r) => ({
    name: r.name,
    localizedName: r.localized_name,
    version: versionToString(r.version),
    kind: r.kind,
    url: r.url,
  }));
}

export async function fetchModuleReleases(
  moduleName: string,
  includeRc: boolean,
  includeMilestones: boolean
): Promise<ModuleReleaseRow[]> {
  const data = await fetchJson<ModuleReleaseJson[]>(
    buildModuleReleasesPath(moduleName, includeRc, includeMilestones)
  );
  return data.map((r) => ({
    version: versionToString(r.version),
    dateTime: r.date_time,
    url: r.url,
  }));
}
