import { Copy, ExternalLink, Link2 } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";

type ReleaseCommandsProps = {
  url: string;
  moduleName: string;
  version: string;
};

export function ReleaseCommands({ url, moduleName, version }: ReleaseCommandsProps) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label={`GitHub actions for ${moduleName} ${version}`}
        onClick={() => setOpen(true)}
      >
        <Link2 className="size-4" />
      </Button>
      <CommandDialog
        open={open}
        onOpenChange={setOpen}
        title="GitHub"
        description={`Actions for ${moduleName} ${version}`}
      >
        <CommandInput placeholder="Search actions..." />
        <CommandList>
          <CommandEmpty>No actions found.</CommandEmpty>
          <CommandGroup heading="GitHub">
            <CommandItem
              onSelect={() => {
                window.open(url, "_blank", "noopener,noreferrer");
                setOpen(false);
              }}
            >
              <ExternalLink />
              <span>Open on GitHub</span>
            </CommandItem>
            <CommandItem
              onSelect={() => {
                void navigator.clipboard.writeText(url);
                setOpen(false);
              }}
            >
              <Copy />
              <span>Copy link</span>
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </CommandDialog>
    </>
  );
}
