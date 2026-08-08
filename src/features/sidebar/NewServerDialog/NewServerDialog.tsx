import { useState, type ReactElement } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
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
    <Dialog open={open} onOpenChange={onOpenChange}>
      {trigger ? <DialogTrigger render={trigger} /> : null}
      <DialogContent className="w-full sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Add Server</DialogTitle>
          <DialogDescription>
            Create a new Minecraft server from a compose file.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-4 px-8 pb-8">
          <SelectSource mode={mode} setMode={setMode} />
          <CreateServerSourceForm mode={mode} onOpenChange={onOpenChange} />
        </div>
      </DialogContent>
    </Dialog>
  );
}
