/** CSV rows from GET /sls/releases (text/plain). Format: name, localized_name, version, url */

export type ReleaseRow = {
  name: string;
  localizedName: string;
  version: string;
  url: string;
};

/** GET /sls/releases/{module} — version may include -RCn; date_time can contain commas. */

export type ModuleReleaseRow = {
  version: string;
  dateTime: string;
  url: string;
};

function splitUrlSuffix(line: string): { head: string; url: string } | null {
  let best = line.lastIndexOf(", https://");
  if (best === -1) best = line.lastIndexOf(", http://");
  if (best === -1) return null;
  return {
    head: line.slice(0, best),
    url: line.slice(best + 2),
  };
}

/** Parses one main releases CSV line; tolerates commas inside localized_name. */
export function parseReleaseLine(line: string): ReleaseRow | null {
  const trimmed = line.trim();
  if (!trimmed) return null;

  const su = splitUrlSuffix(trimmed);
  if (!su) return null;

  const { head, url } = su;
  const first = head.indexOf(", ");
  if (first === -1) return null;

  const name = head.slice(0, first);
  const mid = head.slice(first + 2);
  const last = mid.lastIndexOf(", ");
  if (last === -1) return null;

  const localizedName = mid.slice(0, last);
  const version = mid.slice(last + 2);

  return { name, localizedName, version, url };
}

export function parseReleasesCsv(text: string): ReleaseRow[] {
  const rows: ReleaseRow[] = [];
  for (const line of text.split("\n")) {
    const row = parseReleaseLine(line);
    if (row) rows.push(row);
  }
  return rows;
}

/** Parses module history line: version, date_time, url */
export function parseModuleReleaseLine(line: string): ModuleReleaseRow | null {
  const trimmed = line.trim();
  if (!trimmed) return null;

  const su = splitUrlSuffix(trimmed);
  if (!su) return null;

  const first = su.head.indexOf(", ");
  if (first === -1) return null;

  const version = su.head.slice(0, first);
  const dateTime = su.head.slice(first + 2);
  return { version, dateTime, url: su.url };
}

export function parseModuleReleasesCsv(text: string): ModuleReleaseRow[] {
  const rows: ModuleReleaseRow[] = [];
  for (const line of text.split("\n")) {
    const row = parseModuleReleaseLine(line);
    if (row) rows.push(row);
  }
  return rows;
}
