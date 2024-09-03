export interface Film {
    id: number;
    file: string;
    directory: Directory;
    imdb_id?: string;
    title?: string;
    genres: Genre[];
    release_date?: string; // Assuming NaiveDate is a string format, e.g., 'YYYY-MM-DD'
    plot?: string;
    run_time?: number; // seconds
    has_color?: boolean;
    rating?: number;
    languages: Language[];
    keywords: string[];
    directors: Person[];
    stars: Character[];
    has_watched: boolean;
    left_off_point?: number;
    registered: boolean;
}

export interface Directory {
    id: number;
    path: string;
}

export interface Genre {
    id: number;
    name: string;
}

export interface Language {
    id: number;
    name: string;
}

export enum Gender {
    Male = "Male",
    Female = "Female",
}

export enum Country {
    Sweden = "Sweden",
    UnitedKingdom = "United Kingdom",
    Norway = "Norway",
    UnitedStates = "United States",
    Canada = "Canada",
    Mexico = "Mexico",
    Russia = "Russia",
    France = "France",
    Germany = "Germany",
    Spain = "Spain",
    Italy = "Italy",
    Portugal = "Portugal",
}

export interface Person {
    id: number;
    imdb_id?: string;
    name?: string;
    age?: number;
    gender?: Gender;
    birthplace?: Country;
}

export interface Character {
    film_id: number;
    description: string;
    actor: Person;
}
