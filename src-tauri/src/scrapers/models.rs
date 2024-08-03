#[derive(Debug)]
pub struct ScrapedFilm {
    pub title: Option<String>,
    pub genres: Vec<String>,
    pub release_date: Option<String>,
    pub plot: Option<String>,
    pub run_time: Option<String>,
    pub color: Option<String>,
    pub directors: Vec<ScrapedDirector>,
    pub stars: Vec<ScrapedStar>,
    pub cover: Option<String>,
    pub rating: Option<f64>,
    pub languages: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug)]
pub struct ScrapedDirector {
    pub imdb_id: String,
    pub real_name: String,
}

#[derive(Debug)]
pub struct ScrapedStar {
    pub imdb_id: String,
    pub real_name: String,
    pub character: String,
    pub avatar: Option<String>,
}
