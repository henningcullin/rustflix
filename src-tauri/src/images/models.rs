#[derive(Debug)]
pub enum ImageSize {
    Small,
    Medium,
    Large,
    Full,
}

impl ImageSize {
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        match self {
            ImageSize::Small => Some((400, 400)),
            ImageSize::Medium => Some((800, 800)),
            ImageSize::Large => Some((1400, 1400)),
            ImageSize::Full => None, // Full size does not require resizing
        }
    }

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
