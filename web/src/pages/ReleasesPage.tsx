import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ReleaseCommands } from "@/components/sl/commands";
import { VersionLabel } from "@/components/sl/version";
import { fetchReleases, type ReleaseRow } from "@/lib/api";
import { HeaderWithFilter } from "@/components/sl/header";
import { loadReleaseFilter, saveReleaseFilter, type ReleaseFilter } from "@/lib/filter";

export function ReleasesPage() {
  const [releaseFilter, setReleaseFilter] = useState<ReleaseFilter>(() => loadReleaseFilter());
  const [rows, setRows] = useState<ReleaseRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setRows(await fetchReleases(releaseFilter.includeRc, releaseFilter.includeMilestones));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load releases");
      setRows([]);
    } finally {
      setLoading(false);
    }
  }, [releaseFilter.includeRc, releaseFilter.includeMilestones]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    saveReleaseFilter(releaseFilter);
  }, [releaseFilter]);

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-1">
          <h1 className="text-3xl font-semibold tracking-tight text-foreground">SL releases</h1>
          <p className="text-muted-foreground">Latest module versions from GitHub.</p>
        </div>
      </div>

      <Card className="overflow-hidden border shadow-sm">
        <CardHeader className="border-b bg-muted/30">
          <HeaderWithFilter
            title="Modules"
            description="Sorted by module name. Version reflects the newest matching tag per module."
            filter={releaseFilter}
            onFilterChange={(next) => setReleaseFilter(next)}
            loading={loading}
            onReload={() => void load()}
          />
        </CardHeader>
        <CardContent className="p-0">
          {error ? (
            <div className="space-y-3 p-6">
              <p className="text-sm text-destructive">{error}</p>
              <Button variant="outline" size="sm" onClick={() => void load()}>
                Retry
              </Button>
            </div>
          ) : loading ? (
            <div className="space-y-2 p-6">
              {Array.from({ length: 6 }).map((_, i) => (
                <Skeleton key={i} className="h-10 w-full" />
              ))}
            </div>
          ) : rows.length === 0 ? (
            <p className="p-6 text-sm text-muted-foreground">No releases returned.</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow className="hover:bg-transparent">
                  <TableHead className="w-[140px]">Module</TableHead>
                  <TableHead>Name</TableHead>
                  <TableHead className="w-[300px]">Version</TableHead>
                  <TableHead>Published</TableHead>
                  <TableHead className="text-center w-[110px]" />
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map((r) => (
                  <TableRow key={r.name}>
                    <TableCell className="font-medium">
                      <Link
                        to={`/module/${encodeURIComponent(r.name)}`}
                        className="text-primary hover:underline"
                      >
                        {r.name}
                      </Link>
                    </TableCell>
                    <TableCell className="text-muted-foreground">{r.localizedName}</TableCell>
                    <TableCell>
                      <VersionLabel release={r} />
                    </TableCell>
                    <TableCell className="text-muted-foreground">{r.dateTime}</TableCell>
                    <TableCell className="text-center">
                      <ReleaseCommands url={r.url} moduleName={r.name} version={r.version} />
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
