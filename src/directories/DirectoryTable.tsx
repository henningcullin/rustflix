import { useEffect, useState } from "react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { invoke } from "@tauri-apps/api/tauri";
import { Directory } from "./Directories";
import { useAtom } from "jotai";
import { directoryAtom } from "@/lib/atoms";

export function DirectoryTable() {
  const [directories, setDirectories] = useAtom(directoryAtom);

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
      <TableHeader>
        <TableRow>
          <TableHead className="text-center">Path</TableHead>
          <TableHead className="text-center w-6">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {directories.map((directory) => (
          <TableRow>
            <TableCell>{directory.path}</TableCell>
            <TableCell></TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
