import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Film } from '@/lib/types';

function LanguageForm({ film }: { film: Film | undefined }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>
          <h2 className='text-2xl font-bold mb-4'>Languages</h2>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <b>Language list</b>
      </CardContent>
    </Card>
  );
}

export default LanguageForm;
