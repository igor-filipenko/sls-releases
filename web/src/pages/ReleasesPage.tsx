import { ExternalLink, Package } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { fetchReleases, type ReleaseRow } from "@/lib/api";

function isRcVersion(version: string): boolean {
  return version.includes("Candidate");
}

function isMilestoneVersion(version: string): boolean {
  return version.includes("Milestone");
}

export function ReleasesPage() {
  const [includeRc, setIncludeRc] = useState(false);
  const [includeMilestones, setIncludeMilestones] = useState(false);
  const [rows, setRows] = useState<ReleaseRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setRows(await fetchReleases(includeRc, includeMilestones));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load releases");
      setRows([]);
    } finally {
      setLoading(false);
    }
  }, [includeRc, includeMilestones]);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-1">
          <h1 className="text-3xl font-semibold tracking-tight text-foreground">
            SLS releases
          </h1>
          <p className="text-muted-foreground">
            Latest module versions from GitHub.
          </p>
        </div>
        <div className="flex flex-col gap-2 rounded-lg border bg-card px-4 py-2 shadow-sm">
          <div className="flex items-center justify-between gap-3">
            <label
              htmlFor="rc-toggle"
              className="text-sm font-medium leading-none"
            >
              Include release candidates
            </label>
            <Switch
              id="rc-toggle"
              checked={includeRc}
              onCheckedChange={setIncludeRc}
            />
          </div>
          <div className="flex items-center justify-between gap-3">
            <label
              htmlFor="ms-toggle"
              className="text-sm font-medium leading-none"
            >
              Include milestones
            </label>
            <Switch
              id="ms-toggle"
              checked={includeMilestones}
              onCheckedChange={setIncludeMilestones}
            />
          </div>
        </div>
      </div>

      <Card className="overflow-hidden border shadow-sm">
        <CardHeader className="border-b bg-muted/30">
          <div className="flex items-center gap-2">
            <Package className="size-5 text-muted-foreground" aria-hidden />
            <div>
              <CardTitle>Modules</CardTitle>
              <CardDescription>
                Sorted by module name. Version reflects the newest matching tag
                per module.
              </CardDescription>
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
              {Array.from({ length: 6 }).map((_, i) => (
                <Skeleton key={i} className="h-10 w-full" />
              ))}
            </div>
          ) : rows.length === 0 ? (
            <p className="p-6 text-sm text-muted-foreground">
              No releases returned.
            </p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow className="hover:bg-transparent">
                  <TableHead className="w-[140px]">Module</TableHead>
                  <TableHead>Name</TableHead>
                  <TableHead className="w-[450px]">Version</TableHead>
                  <TableHead className="w-[10px]">GitHub</TableHead>
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
                    <TableCell className="text-muted-foreground">
                      {r.localizedName}
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-sm tabular-nums">
                          {r.version}
                        </span>
                        {isRcVersion(r.kind) ? (
                          <Badge className="bg-yellow-400 text-black">RC</Badge>
                        ) : isMilestoneVersion(r.kind) ? (
                          <Badge className="bg-red-500 text-white">
                            Milestone
                          </Badge>
                        ) : (
                          <Badge className="bg-green-500 text-white">Production</Badge>
                        )}
                      </div>
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
