import { Film } from '@/lib/types';
import { ColumnDef } from '@tanstack/react-table';

type DataTable = {
  columns: ColumnDef<Film>;
};

export default function DataTable({ columns }: DataTable) {}
