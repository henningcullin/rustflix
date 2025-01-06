export type Film = {
  id: number;
  file?: string;
  imdb_id?: string;
  title?: string;
  genres: Genre[];
  release_date?: string; // Assuming NaiveDate is a string format, e.g., 'YYYY-MM-DD'
  plot?: string;
  run_time?: number; // seconds
  has_color?: boolean;
  rating?: number;
  languages: Language[];
  keywords: Keyword[];
  directors: Person[];
  stars: Character[];
  has_watched: boolean;
  left_off_point?: number;
  registered: boolean;
};

export type Directory = {
  id: number;
  path: string;
};

export type Genre = {
  id: number;
  name: string;
};

export type Language = {
  id: number;
  name: string;
};

export type Keyword = {
  id: number;
  name: string;
};

export enum Gender {
  Male = 'Male',
  Female = 'Female',
}

export enum Country {
  Sweden = 'Sweden',
  UnitedKingdom = 'United Kingdom',
  Norway = 'Norway',
  UnitedStates = 'United States',
  Canada = 'Canada',
  Mexico = 'Mexico',
  Russia = 'Russia',
  France = 'France',
  Germany = 'Germany',
  Spain = 'Spain',
  Italy = 'Italy',
  Portugal = 'Portugal',
}

export type Person = {
  id: number;
  imdb_id?: string;
  name?: string;
  age?: number;
  gender?: Gender;
  birthplace?: Country;
};

export type Character = {
  film_id: number;
  description: string;
  actor: Person;
};
