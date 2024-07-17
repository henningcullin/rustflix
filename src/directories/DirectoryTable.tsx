import { useEffect, useState } from "react";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { invoke } from "@tauri-apps/api/tauri";

interface Directory {
  id: number;
  path: string;
}

export function DirectoryTable() {
  const [directories, setDirectories] = useState<Directory[]>([]);

  async function getDirectories() {
    try {
      const data: Directory[] | null = await invoke("get_all_directories");
      if (data) setDirectories(data);
    } catch (error) {
      console.error(error);
    }
  }

  useEffect(() => {
    getDirectories();
  }, []);

  return (
    <Table>
      <TableCaption>Directories to scan for films</TableCaption>
      <TableHeader>
        <TableRow>
          <TableHead>Path</TableHead>
          <TableHead>Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {directories.map((directory) => (
          <TableRow>
            <TableCell>{directory.id}</TableCell>
            <TableCell>{directory.path}</TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
