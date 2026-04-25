import { ArrowLeft, ExternalLink, ListFilter, Package, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { fetchModuleReleases, type ModuleReleaseRow } from "@/lib/api";

export function ModulePage() {
  const { name: rawName } = useParams<{ name: string }>();
  const moduleName = rawName ? decodeURIComponent(rawName) : "";

  const [includeRc, setIncludeRc] = useState(false);
  const [includeMilestones, setIncludeMilestones] = useState(false);
  const [rows, setRows] = useState<ModuleReleaseRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    if (!moduleName) return;
    setLoading(true);
    setError(null);
    try {
      setRows(await fetchModuleReleases(moduleName, includeRc, includeMilestones));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load history");
      setRows([]);
    } finally {
      setLoading(false);
    }
  }, [moduleName, includeRc, includeMilestones]);

  useEffect(() => {
    void load();
  }, [load]);

  if (!moduleName) {
    return (
      <p className="text-sm text-muted-foreground">Module not specified.</p>
    );
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
            {moduleName}
          </h1>
          <p className="text-muted-foreground">
            Release history for this module (newest first).
          </p>
        </div>
      </div>

      <Card className="overflow-hidden border shadow-sm">
        <CardHeader className="border-b bg-muted/30">
          <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div className="flex items-center gap-2">
              <Package className="size-5 text-muted-foreground" aria-hidden />
              <div>
                <CardTitle>Versions</CardTitle>
                <CardDescription>
                  Timestamps use the server&apos;s local timezone formatting.
                </CardDescription>
              </div>
            </div>
            <div className="flex items-center gap-2 self-start sm:self-center">
              <Button
                variant="outline"
                size="sm"
                className="gap-2"
                onClick={() => void load()}
                disabled={loading}
                aria-label="Reload module releases"
              >
                <RefreshCw
                  className={`size-4 shrink-0 ${loading ? "animate-spin" : ""}`}
                  aria-hidden
                />
                Reload
              </Button>
              <Popover>
                <PopoverTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    className="gap-2"
                    aria-label="Filter release types"
                  >
                    <ListFilter className="size-4 shrink-0" aria-hidden />
                    Release types
                    {(includeRc || includeMilestones) ? (
                      <span className="rounded-sm bg-muted px-1.5 py-0.5 text-xs font-normal text-muted-foreground tabular-nums">
                        {[includeRc, includeMilestones].filter(Boolean).length}
                      </span>
                    ) : null}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-80" align="end">
                  <PopoverHeader>
                    <PopoverTitle>Release types</PopoverTitle>
                    <PopoverDescription>
                      Production tags are always considered. Turn on options below to
                      include pre-releases in this module&apos;s release history.
                    </PopoverDescription>
                  </PopoverHeader>
                  <div className="flex flex-col gap-3 pt-2">
                    <div className="flex items-start gap-3">
                      <Checkbox
                        id="module-include-rc"
                        checked={includeRc}
                        onCheckedChange={(v) => setIncludeRc(v === true)}
                        className="mt-0.5"
                      />
                      <label
                        htmlFor="module-include-rc"
                        className="cursor-pointer text-sm font-medium leading-snug peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                      >
                        Include release candidates
                      </label>
                    </div>
                    <div className="flex items-start gap-3">
                      <Checkbox
                        id="module-include-milestones"
                        checked={includeMilestones}
                        onCheckedChange={(v) => setIncludeMilestones(v === true)}
                        className="mt-0.5"
                      />
                      <label
                        htmlFor="module-include-milestones"
                        className="cursor-pointer text-sm font-medium leading-snug peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                      >
                        Include milestones
                      </label>
                    </div>
                  </div>
                </PopoverContent>
              </Popover>
            </div>
          </div>
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
                      {r.version}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {r.dateTime}
                    </TableCell>
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
