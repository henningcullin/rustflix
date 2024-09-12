import { z } from 'zod';

export const formSchema = z.object({
    id: z.string(),
    file: z.string().optional(),
    directory: z.string().optional(),
    imdb_id: z.string(),
    title: z.string(),
    release_date: z.date(),
    plot: z.string(),
    run_time: z.number(),
    has_color: z.boolean(),
    rating: z.number(),
    has_watched: z.boolean(),
    left_off_point: z.number().optional(),
    registered: z.boolean().optional(),
    genres: z.array(z.number()),
    directors: z.array(z.number()),
    stars: z.array(z.number()),
    languages: z.array(z.number()),
    keywords: z.array(z.string()),
  });
  
  export type FormSchema = z.infer<typeof formSchema>;