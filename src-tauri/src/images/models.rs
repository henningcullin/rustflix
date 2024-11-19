#[derive(Debug, PartialEq)]
pub enum ImageSize {
    Small,
    Medium,
    Large,
    Full,
}

impl ImageSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageSize::Small => "small",
            ImageSize::Medium => "medium",
            ImageSize::Large => "large",
            ImageSize::Full => "full",
        }
    }
}

impl From<Option<&str>> for ImageSize {
    fn from(size: Option<&str>) -> Self {
        match size {
            Some("small") => ImageSize::Small,
            Some("large") => ImageSize::Large,
            Some("full") => ImageSize::Full,
            _ => ImageSize::Medium, // Default size
        }
    }
}
