import { z } from 'zod';

export const formSchema = z.object({
  imdb_id: z.string(),
  title: z.string(),
  release_date: z.date(),
  plot: z.string(),
  run_time: z.number(),
  has_color: z.boolean(),
  rating: z.number(),
  has_watched: z.boolean(),
  left_off_point: z.number().optional(),
});

export type FormSchema = z.infer<typeof formSchema>;
