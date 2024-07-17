import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "react";

export function DirectoryForm() {
  const [path, setPath] = useState<string>("");

  async function addDirectory(path: string) {
    try {
      await invoke("add_directory", { path });
    } catch (error) {
      console.error("Failed to add directory:", error);
    }
  }

  return (
    <form
      onSubmit={async (event) => {
        event.preventDefault();
        (event.target as HTMLFormElement).reset();
        await addDirectory(path);
      }}
    >
      <label>Path</label>
      <input
        type="file"
        name="path"
        placeholder="Directory"
        value={path}
        onChange={(event) => {
          console.log(event.target.value);
        }}
      />
      <button>Add Directory</button>
    </form>
  );
}
