import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "react";

import { Label } from "@/components/ui/label";

export function DirectoryForm() {
  const [directory, setDirectory] = useState<string>("");

  async function addDirectory(path: string) {
    try {
      await invoke("add_directory", { path });
      setDirectory("");
    } catch (error) {}
  }

  async function selectDirectory() {
    try {
      const dir: string | null = await invoke("select_directory");
      if (typeof dir === "string" && dir) setDirectory(dir);
    } catch (error) {
      console.error("Failed to select directory", error);
    }
  }

  return (
    <form style={{ width: "450px" }}>
      <Label>Directory</Label>
      <div style={{ display: "flex", gap: "5px" }}>
        <Input value={directory} readOnly disabled />
        <Button onClick={selectDirectory}>Select Directory</Button>
      </div>
      <Button onClick={() => addDirectory(directory)}>Add Directory</Button>
    </form>
  );
}
