import { ListFilter, Package, RefreshCw } from "lucide-react";
import { CardDescription, CardTitle } from "./card";
import { Button } from "./button";
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from "./popover";
import { Checkbox } from "./checkbox";
import type { ReleaseFilter } from "@/lib/filter";

type HeaderWithFilterProps = {
  title: string;
  description: string;
  filter: ReleaseFilter;
  loading: boolean;
  onReload: () => void;
  onFilterChange: (next: ReleaseFilter) => void;
};

export function HeaderWithFilter({
  title,
  description,
  filter,
  loading,
  onReload,
  onFilterChange,
}: HeaderWithFilterProps) {
  return (
    <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
      <div className="flex items-center gap-2">
        <Package className="size-5 text-muted-foreground" aria-hidden />
        <div>
          <CardTitle>{title}</CardTitle>
          <CardDescription>{description}</CardDescription>
        </div>
      </div>
      <div className="flex items-center gap-2 self-start sm:self-center">
        <Button
          variant="outline"
          size="sm"
          className="gap-2"
          onClick={() => onReload()}
          disabled={loading}
          aria-label="Reload module releases"
        >
          <RefreshCw className={`size-4 shrink-0 ${loading ? "animate-spin" : ""}`} aria-hidden />
          Reload
        </Button>
        <Popover>
          <PopoverTrigger asChild>
            <Button variant="outline" size="sm" className="gap-2" aria-label="Filter release types">
              <ListFilter className="size-4 shrink-0" aria-hidden />
              Release types
              {filter.includeRc || filter.includeMilestones ? (
                <span className="rounded-sm bg-muted px-1.5 py-0.5 text-xs font-normal text-muted-foreground tabular-nums">
                  {[filter.includeRc, filter.includeMilestones].filter(Boolean).length}
                </span>
              ) : null}
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-80" align="end">
            <PopoverHeader>
              <PopoverTitle>Release types</PopoverTitle>
              <PopoverDescription>
                Production tags are always considered. Turn on options below to include pre-releases
                or milestones in this module&apos;s release history.
              </PopoverDescription>
            </PopoverHeader>
            <div className="flex flex-col gap-3 pt-2">
              <div className="flex items-start gap-3">
                <Checkbox
                  id="module-include-rc"
                  checked={filter.includeRc}
                  onCheckedChange={(v) => onFilterChange({ ...filter, includeRc: v === true })}
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
                  checked={filter.includeMilestones}
                  onCheckedChange={(v) =>
                    onFilterChange({ ...filter, includeMilestones: v === true })
                  }
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
  );
}
