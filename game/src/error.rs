use std::fmt;

#[derive(Debug)]
pub(crate) enum AssetError {
    Io(std::io::Error),
    Image(image::ImageError),
    Custom(String),
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::Io(e) => write!(f, "I/O error: {e}"),
            AssetError::Image(e) => write!(f, "Image decode error: {e}"),
            AssetError::Custom(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for AssetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AssetError::Io(e) => Some(e),
            AssetError::Image(e) => Some(e),
            AssetError::Custom(_) => None,
        }
    }
}

impl From<std::io::Error> for AssetError {
    fn from(e: std::io::Error) -> Self {
        AssetError::Io(e)
    }
}

impl From<image::ImageError> for AssetError {
    fn from(e: image::ImageError) -> Self {
        AssetError::Image(e)
    }
}
