import { Badge } from '@/components/ui/badge';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Film } from '@/lib/types';
import { TrashIcon } from '@radix-ui/react-icons';

function KeywordForm({ film }: { film: Film | undefined }) {
  return (
    <div className='mt-8 grid place-items-center'>
      <Card className='w-[400px]'>
        <CardHeader>
          <CardTitle>Title</CardTitle>
          <CardDescription>Description</CardDescription>
        </CardHeader>
        <CardContent>Content</CardContent>
      </Card>
      <div className='flex gap-5 mt-12'>
        {film?.keywords?.map((keyword) => (
          <Badge>
            <span className='text-sm mr-2'>{keyword}</span>
            <TrashIcon className='w-5 h-5' />
          </Badge>
        ))}
      </div>
    </div>
  );
}

export default KeywordForm;
