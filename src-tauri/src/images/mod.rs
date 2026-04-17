use std::path::{Path, PathBuf};

use image::imageops::FilterType;

use crate::error::AppResult;

#[derive(Clone, Copy)]
pub enum CoverSize {
    Small,
    Medium,
    Large,
    Full,
}

impl CoverSize {
    pub fn filename(self) -> &'static str {
        match self {
            Self::Small => "small.jpg",
            Self::Medium => "medium.jpg",
            Self::Large => "large.jpg",
            Self::Full => "full.jpg",
        }
    }

    pub fn max_width(self) -> Option<u32> {
        match self {
            Self::Small => Some(400),
            Self::Medium => Some(800),
            Self::Large => Some(1400),
            Self::Full => None,
        }
    }
}

pub fn film_cover_dir(app_data_dir: &Path, film_id: i64) -> PathBuf {
    app_data_dir.join("covers").join(film_id.to_string())
}

pub async fn download_and_store_cover(
    client: &reqwest::Client,
    url: &str,
    app_data_dir: &Path,
    film_id: i64,
) -> AppResult<String> {
    let bytes = client.get(url).send().await?.error_for_status()?.bytes().await?;

    let dir = film_cover_dir(app_data_dir, film_id);
    let app_data_dir_owned = app_data_dir.to_path_buf();

    tokio::task::spawn_blocking(move || -> AppResult<()> {
        std::fs::create_dir_all(&dir)?;

        let img = image::load_from_memory(&bytes)?;

        for size in [CoverSize::Small, CoverSize::Medium, CoverSize::Large, CoverSize::Full] {
            let out_path = dir.join(size.filename());
            let resized = match size.max_width() {
                Some(w) if img.width() > w => {
                    let ratio = w as f32 / img.width() as f32;
                    let h = (img.height() as f32 * ratio) as u32;
                    img.resize(w, h, FilterType::Lanczos3)
                }
                _ => img.clone(),
            };
            let rgb = resized.to_rgb8();
            let dyn_img = image::DynamicImage::ImageRgb8(rgb);
            let mut out = std::fs::File::create(&out_path)?;
            dyn_img.write_to(&mut out, image::ImageFormat::Jpeg)?;
        }

        let _ = &app_data_dir_owned;
        Ok(())
    })
    .await
    .map_err(|e| crate::error::AppError::Other(format!("image join error: {e}")))??;

    Ok(format!("covers/{film_id}"))
}
