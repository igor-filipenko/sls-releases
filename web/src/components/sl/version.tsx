import { isMilestoneVersion, isRcVersion } from "@/lib/utils";
import type { ReleaseRow } from "@/lib/api";
import { Badge } from "@/components/ui/badge";

export function VersionLabel({ release }: { release: ReleaseRow }) {
  return (
    <div className="flex items-center gap-2">
      <span className="font-mono text-sm tabular-nums">{release.version}</span>
      {isRcVersion(release.kind) ? (
        <Badge className="bg-yellow-700 text-black">RC</Badge>
      ) : isMilestoneVersion(release.kind) ? (
        <Badge className="bg-blue-700 text-white">Milestone</Badge>
      ) : (
        <Badge className="bg-green-700 text-white">Production</Badge>
      )}
    </div>
  );
}
