export type Directory = {
  id: number;
  path: string;
  recursive: number;
  created_at: string;
};

export type MatchedFile = {
  film_id: number;
  file_path: string;
  title: string;
};

export type UnmatchedFile = {
  file_path: string;
  display_name: string;
};

export type ScanResult = {
  matched: MatchedFile[];
  unmatched: UnmatchedFile[];
};

export type Film = {
  id: number;
  file_path: string;
  tmdb_id: number | null;
  imdb_id: string | null;
  title: string;
  original_title: string | null;
  overview: string | null;
  release_date: string | null;
  runtime: number | null;
  rating: number | null;
  poster_path: string | null;
  backdrop_path: string | null;
  left_off_point: number;
  watched: number;
  created_at: string;
  updated_at: string;
};

export type FilmListItem = {
  id: number;
  title: string;
  release_date: string | null;
  runtime: number | null;
  rating: number | null;
  poster_path: string | null;
  watched: number;
  left_off_point: number;
};

export type Genre = { id: number; name: string };

export type CastMember = {
  person_id: number;
  name: string;
  profile_path: string | null;
  character: string | null;
  role: string;
  sort_order: number;
};

export type FilmDetail = Film & {
  genres: Genre[];
  cast: CastMember[];
};

export type TmdbSearchResult = {
  id: number;
  title: string;
  original_title: string | null;
  overview: string | null;
  release_date: string | null;
  poster_path: string | null;
  backdrop_path: string | null;
  vote_average: number | null;
};

export type Settings = {
  tmdb_api_key: string | null;
};
