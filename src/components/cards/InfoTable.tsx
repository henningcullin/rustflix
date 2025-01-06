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
  accessorKey: string;
  cell?: ({ item }: { item: T }) => ReactNode;
};

export type InfoTableConfig<T> = {
  item: T;
  infoRows: InfoRow<T>[];
};

function InfoTableCell<T>({ item, infoRow }: { item: T; infoRow: InfoRow<T> }) {
  return (
    <TableRow key={infoRow.accessorKey}>
      <TableHead>{infoRow.caption}</TableHead>
      <TableCell>
        {typeof infoRow.cell === 'function' ? (
          infoRow.cell({ item })
        ) : (
          <p>Blank</p>
        )}
      </TableCell>
    </TableRow>
  );
}

export default function InfoTable<T>({
  config,
}: {
  config: InfoTableConfig<T>;
}) {
  return (
    <Table>
      <TableBody>
        {config.infoRows.map((infoRow) =>
          InfoTableCell({ item: config.item, infoRow })
        )}
      </TableBody>
    </Table>
  );
}
