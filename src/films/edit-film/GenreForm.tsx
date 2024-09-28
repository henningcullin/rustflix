import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Film } from '@/lib/types';

function GenreForm({ film }: { film: Film | undefined }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Genres</CardTitle>
      </CardHeader>
      <CardContent>
        <div>
          <b>Genre list</b>
        </div>
      </CardContent>
    </Card>
  );
}

export default GenreForm;
