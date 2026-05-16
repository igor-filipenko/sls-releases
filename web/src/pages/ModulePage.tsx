import { ArrowLeft, ExternalLink } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";

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
import { fetchModule, fetchModuleReleases, type Module, type ReleaseRow } from "@/lib/api";
import { VersionLabel } from "@/components/sl/version";
import { HeaderWithFilter } from "@/components/sl/header";
import { loadReleaseFilter, saveReleaseFilter, type ReleaseFilter } from "@/lib/filter";

export function ModulePage() {
  const { name: rawName } = useParams<{ name: string }>();
  const moduleName = rawName ? decodeURIComponent(rawName) : "";

  const [releaseFilter, setReleaseFilter] = useState<ReleaseFilter>(() => loadReleaseFilter());
  const [module, setModule] = useState<Module>({ name: "", localizedName: "" });
  const [rows, setRows] = useState<ReleaseRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    if (!moduleName) return;
    setLoading(true);
    setError(null);
    try {
      setModule(await fetchModule(moduleName));
      setRows(
        await fetchModuleReleases(
          moduleName,
          releaseFilter.includeRc,
          releaseFilter.includeMilestones,
        ),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load history");
      setRows([]);
    } finally {
      setLoading(false);
    }
  }, [moduleName, releaseFilter.includeRc, releaseFilter.includeMilestones]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    saveReleaseFilter(releaseFilter);
  }, [releaseFilter]);

  if (!moduleName) {
    return <p className="text-sm text-muted-foreground">Module not specified.</p>;
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="space-y-2">
          <Button variant="ghost" size="sm" className="-ml-2 w-fit" asChild>
            <Link to="/" className="gap-2">
              <ArrowLeft className="size-4" />
              All modules
            </Link>
          </Button>
          <h1 className="text-3xl font-semibold tracking-tight font-mono">
            {module.localizedName}
          </h1>
          <p className="text-muted-foreground">
            Release history for the module {module.name} (newest first).
          </p>
        </div>
      </div>

      <Card className="overflow-hidden border shadow-sm">
        <CardHeader className="border-b bg-muted/30">
          <HeaderWithFilter
            title="Versions"
            description="Timestamps use the server's local timezone formatting."
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
              {Array.from({ length: 5 }).map((_, i) => (
                <Skeleton key={i} className="h-10 w-full" />
              ))}
            </div>
          ) : rows.length === 0 ? (
            <p className="p-6 text-sm text-muted-foreground">
              No releases for this module with the current filter.
            </p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow className="hover:bg-transparent">
                  <TableHead className="w-[180px]">Version</TableHead>
                  <TableHead>Published</TableHead>
                  <TableHead className="text-center w-[110px]">GitHub</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map((r) => (
                  <TableRow key={`${r.version}-${r.url}`}>
                    <TableCell className="font-mono text-sm tabular-nums">
                      <VersionLabel release={r} />
                    </TableCell>
                    <TableCell className="text-muted-foreground">{r.dateTime}</TableCell>
                    <TableCell className="text-right">
                      <Button variant="ghost" size="sm" asChild>
                        <a
                          href={r.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="inline-flex items-center gap-1"
                        >
                          Open
                          <ExternalLink className="size-3.5" />
                        </a>
                      </Button>
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
