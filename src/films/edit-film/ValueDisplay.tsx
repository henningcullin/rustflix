import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useToast } from '@/hooks/use-toast';
import { useCallback } from 'react';

interface ValueDisplayProps {
  label: string;
  value: string | undefined;
  icon: React.ReactNode; // The icon component to display
}

function ValueDisplay({ label, value, icon }: ValueDisplayProps) {
  const { toast } = useToast();

  const handleCopy = useCallback(() => {
    if (value) {
      navigator.clipboard
        .writeText(value)
        .then(() => {
          console.log('before toast');
          toast({
            title: 'Copied to clipboard',
            description: `The ${label?.toLowerCase()} was copied to your clipboard`,
          });
        })
        .catch((err) => {
          console.error('Failed to copy: ', err);
          toast({
            title: 'Failed to copy to clipboard',
            description: `An error occured while sending your ${label?.toLowerCase()} to the clipboard`,
          });
        });
    }
  }, [label, value]);

  if (!value) return null;

  return (
    <div className='py-[0.6em] shadow-sm items-center'>
      <Label className='flex items-center gap-2 pb-2 text-base font-medium flex-grow'>
        {icon}
        {label}
      </Label>
      <div className='flex'>
        <Input readOnly value={value} />
        <Button variant='outline' onClick={handleCopy} className='ml-2'>
          Copy
        </Button>
      </div>
    </div>
  );
}

export default ValueDisplay;
