import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "react";

export function DirectoryForm() {
  const [directory, setDirectory] = useState<string>("");

  async function selectDirectory() {
    try {
      const dir: string | null = await invoke("select_directory");
      if (typeof dir === "string") setDirectory(dir);
    } catch (error) {
      console.error("Failed to select directory", error);
    }
  }

  return (
    <>
      <Button onClick={selectDirectory}>Select Directory</Button>
      <p>{directory}</p>
    </>
  );
}

async function addDirectory(path: string) {
  try {
    await invoke("add_directory", { path });
  } catch (error) {
    console.error("Failed to add directory:", error);
  }
}
