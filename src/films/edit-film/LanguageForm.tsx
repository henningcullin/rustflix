import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Film } from '@/lib/types';

function LanguageForm({ film }: { film: Film | undefined }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Languages</CardTitle>
      </CardHeader>
      <CardContent>
        <b>Language list</b>
      </CardContent>
    </Card>
  );
}

export default LanguageForm;
