import { useState, type ReactElement } from "react";
import { Plus } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import CreateServerSourceForm from "./components/CreateServerSourceForm";
import SelectSource from "./components/SelectSource";
import { CreateServerSource } from "@/store/serversStore";

export function NewServerDialog({
  open,
  onOpenChange,
  trigger,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  trigger?: ReactElement;
}) {
  const [mode, setMode] = useState<CreateServerSource>("template");

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetTrigger
        render={
          trigger ?? (
            <Button variant="ghost" size="icon-sm" className="h-7 w-7">
              <Plus />
            </Button>
          )
        }
      />
      <SheetContent side="right" className="w-full sm:max-w-md">
        <SheetHeader>
          <SheetTitle>Add Server</SheetTitle>
          <SheetDescription>
            Create a new Minecraft server from a compose file.
          </SheetDescription>
        </SheetHeader>
        <div className="flex flex-col gap-4 px-8 pb-8">
          <SelectSource mode={mode} setMode={setMode} />
          <CreateServerSourceForm mode={mode} onOpenChange={onOpenChange} />
        </div>
      </SheetContent>
    </Sheet>
  );
}
