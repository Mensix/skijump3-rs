use std::fmt;

#[derive(Debug)]
pub enum AssetError {
    /// I/O error reading a file at `path`.
    Io {
        path: String,
        source: std::io::Error,
    },
    /// UTF-8 decode error in `path`.
    Utf8 {
        path: String,
        source: std::str::Utf8Error,
    },
    /// TOML deserialization error in `path`.
    Toml {
        path: String,
        source: toml::de::Error,
    },
    /// Image decode error (from `gfx::png` — no file path context).
    Image(image::ImageError),
    /// Unsupported format version in `path`.
    FormatVersion {
        path: String,
        expected: u32,
        got: u32,
    },
    /// Miscellaneous string error.
    Custom(String),
}

impl AssetError {
    pub fn io(path: impl Into<String>, source: std::io::Error) -> Self {
        AssetError::Io {
            path: path.into(),
            source,
        }
    }

    pub fn utf8(path: impl Into<String>, source: std::str::Utf8Error) -> Self {
        AssetError::Utf8 {
            path: path.into(),
            source,
        }
    }

    pub fn toml(path: impl Into<String>, source: toml::de::Error) -> Self {
        AssetError::Toml {
            path: path.into(),
            source,
        }
    }

    pub fn format_version(path: impl Into<String>, expected: u32, got: u32) -> Self {
        AssetError::FormatVersion {
            path: path.into(),
            expected,
            got,
        }
    }
}

impl Clone for AssetError {
    fn clone(&self) -> Self {
        match self {
            AssetError::Io { path, source } => {
                AssetError::Custom(format!("Failed to read {path}: {source}"))
            }
            AssetError::Utf8 { path, source } => {
                AssetError::Custom(format!("{path} is not valid UTF-8: {source}"))
            }
            AssetError::Toml { path, source } => {
                AssetError::Custom(format!("Failed to parse {path}: {source}"))
            }
            AssetError::Image(e) => AssetError::Custom(format!("Image decode error: {e}")),
            AssetError::FormatVersion {
                path,
                expected,
                got,
            } => AssetError::Custom(format!(
                "Unsupported format version {got} in {path} (expected {expected})"
            )),
            AssetError::Custom(msg) => AssetError::Custom(msg.clone()),
        }
    }
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::Io { path, source } => {
                write!(f, "Failed to read {path}: {source}")
            }
            AssetError::Utf8 { path, source } => {
                write!(f, "{path} is not valid UTF-8: {source}")
            }
            AssetError::Toml { path, source } => {
                write!(f, "Failed to parse {path}: {source}")
            }
            AssetError::Image(e) => write!(f, "Image decode error: {e}"),
            AssetError::FormatVersion {
                path,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Unsupported format version {got} in {path} (expected {expected})"
                )
            }
            AssetError::Custom(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for AssetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AssetError::Io { source, .. } => Some(source),
            AssetError::Utf8 { source, .. } => Some(source),
            AssetError::Toml { source, .. } => Some(source),
            AssetError::Image(e) => Some(e),
            AssetError::FormatVersion { .. } => None,
            AssetError::Custom(_) => None,
        }
    }
}

impl From<image::ImageError> for AssetError {
    fn from(e: image::ImageError) -> Self {
        AssetError::Image(e)
    }
}
