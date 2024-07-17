import { useState } from "react";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export function DirectoryList() {
  const [directories, setDirectories] = useState<string[]>([]);

  return (
    <div>
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
              <TableCell></TableCell>
              <TableCell></TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
