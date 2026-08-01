import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import ConsoleTab from "./components/ConsoleTab";

export default function ServerPanel() {
  return (
    <Tabs defaultValue={"console"}>
      <TabsList variant={"line"}>
        <TabsTrigger value={"console"}>Console</TabsTrigger>
      </TabsList>
      <TabsContent value={"console"}>
        <ConsoleTab></ConsoleTab>
      </TabsContent>
    </Tabs>
  );
}
