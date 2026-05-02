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

export type Module = {
  name: string;
  localizedName: string;
}

export type ReleaseRow = {
  name: string;
  localizedName: string;
  version: string;
  dateTime: string;
  kind: string;
  url: string;
};

type ModuleJson = {
  name: string;
  localized_name: string;
};

type ReleaseJson = {
  name: string;
  localized_name: string;
  kind: string;
  version: VersionJson;
  url: string;
  date_time: string;
};

export function buildModulesPath(
  name?: string
): string {
  return name ? `/sls/modules?name=${name}` : "/sls/modules";
}

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

export async function fetchModule(name: string): Promise<Module> {
  const data = await fetchJson<ModuleJson[]>(buildModulesPath(name));
  const m = data[0];
  if (!m) throw new Error("Module not found");
  return {
    name: m.name,
    localizedName: m.localized_name,
  };
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
    dateTime: r.date_time,
    kind: r.kind,
    url: r.url,
  }));
}

export async function fetchModuleReleases(
  moduleName: string,
  includeRc: boolean,
  includeMilestones: boolean
): Promise<ReleaseRow[]> {
  const data = await fetchJson<ReleaseJson[]>(
    buildModuleReleasesPath(moduleName, includeRc, includeMilestones)
  );
  return data.map((r) => ({
    name: r.name,
    localizedName: r.localized_name,
    version: versionToString(r.version),
    dateTime: r.date_time,
    kind: r.kind,
    url: r.url,
  }));
}
