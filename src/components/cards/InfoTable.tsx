import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
} from '@/components/ui/table';
import { ReactNode } from 'react';

export type InfoRow<T> = {
  caption: string;
  accessorKey: keyof T;
  cell: ({ item }: { item: T }) => ReactNode;
};

export type InfoTableConfig<T> = {
  infoRows: InfoRow<T>[];
};

type InfoRowProps<T> = {
  item: T;
  infoRow: InfoRow<T>;
  index: number;
};

function InfoTableRow<T>({ item, infoRow, index }: InfoRowProps<T>) {
  return (
    <TableRow key={index} className='w-full'>
      <TableHead>{infoRow.caption}</TableHead>
      <TableCell>{infoRow.cell({ item })}</TableCell>
    </TableRow>
  );
}

type InfoTableProps<T> = {
  item: T;
  config: InfoTableConfig<T>;
};

export function InfoTable<T>({ item, config }: InfoTableProps<T>) {
  return (
    <Table className='text-base'>
      <TableBody>
        {config.infoRows.map((infoRow, index) => (
          <InfoTableRow item={item} infoRow={infoRow} index={index} />
        ))}
      </TableBody>
    </Table>
  );
}
