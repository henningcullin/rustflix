use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ScrapedFilm {
    pub id: u32,
    pub title: Option<String>,
    pub genres: Vec<String>,
    pub imdb_id: String,
    pub release_date: Option<String>,
    pub plot: Option<String>,
    pub run_time: Option<String>,
    pub color: Option<String>,
    pub directors: Vec<ScrapedDirector>,
    pub stars: Vec<ScrapedStar>,
    pub cover_image: Option<String>,
    pub rating: Option<f64>,
    pub languages: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScrapedDirector {
    pub imdb_id: String,
    pub real_name: String,
}

#[derive(Debug, Serialize)]
pub struct ScrapedStar {
    pub imdb_id: String,
    pub real_name: String,
    pub character: String,
    pub avatar: Option<String>,
}
