use std::{fmt::Display, str::FromStr};

use crate::application::http::{headers::Headers, multipart::MultipartPart};

#[derive(Clone, Eq, PartialEq)]
pub enum MimeType {
    // Image File Types
    Jpeg,
    Gif,
    Png,
    Webp,
    // Audio File Types
    Wave,
    Mp3,
    Ogg,
    Aac,
    Webm,
    Midi,
    // Others go here, add as needed.
    Unknown(String),
}
impl Default for MimeType {
    fn default() -> Self {
        Self::Unknown("application/octet-stream".into())
    }
}
impl Display for MimeType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let data = match self {
            // Image Types
            MimeType::Jpeg => "image/jpeg",
            MimeType::Gif => "image/gif",
            MimeType::Png => "image/png",
            MimeType::Webp => "image/webp",
            // Audio Types
            MimeType::Wave => "audio/wave",
            MimeType::Mp3 => "audio/mpeg",
            MimeType::Ogg => "audio/ogg",
            MimeType::Aac => "audio/aac",
            MimeType::Webm => "audio/webm",
            MimeType::Midi => "audio/midi",
            // Everything Else
            MimeType::Unknown(_) => "application/octect-stream",
        };
        f.write_str(data)
    }
}

// TODO: Merge this MIME type with the logic in the main application logic.
impl MimeType {
    fn is_image(&self) -> bool {
        match self {
            MimeType::Jpeg | MimeType::Gif | MimeType::Png | MimeType::Webp => true,
            _ => false,
        }
    }
    fn is_audio(&self) -> bool {
        match self {
            MimeType::Wave
            | MimeType::Mp3
            | MimeType::Ogg
            | MimeType::Aac
            | MimeType::Webm
            | MimeType::Midi => true,
            _ => false,
        }
    }
}

impl MimeType {
    pub fn try_from_filename(filename: &str) -> Result<Self, crate::Error> {
        let ext = filename
            .split('.')
            .last()
            .ok_or(crate::Error::BadRequest("Invalid filename provided.".into()))?;
        let mime_type = match ext {
            // Image
            "jpg" | "jpeg" => Self::Jpeg,
            "gif" => Self::Gif,
            "png" => Self::Png,
            "webp" => Self::Webp,
            // Audio
            "aac" => Self::Aac,
            "mp3" => Self::Mp3,
            "wav" | "wave" => Self::Wave,
            "weba" => Self::Webm,
            "mid" | "midi" => Self::Midi,
            "ogg" | "oga" | "opus" => Self::Ogg,
            _ => Err(crate::Error::BadRequest("Invalid file format requested".into()))?,
        };
        Ok(mime_type)
    }

    pub fn get_default_extension(&self) -> &str {
        match self {
            Self::Jpeg => ".jpg",
            Self::Gif => ".gif",
            Self::Png => ".png",
            Self::Webp => ".webp",
            // Audio,
            Self::Aac => ".aac",
            Self::Mp3 => ".mp3",
            Self::Wave => ".wav",
            Self::Webm => ".weba",
            Self::Midi => ".mid",
            Self::Ogg => ".ogg",
            Self::Unknown(_) => "",
        }
    }
}
impl FromStr for MimeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // Image
            "image/jpeg" => Ok(Self::Jpeg),
            "image/gif" => Ok(Self::Gif),
            "image/png" => Ok(Self::Png),
            "image/webp" => Ok(Self::Webp),
            // Audio
            "audio/wave" => Ok(Self::Wave),
            "audio/mpeg" => Ok(Self::Mp3),
            "audio/ogg" => Ok(Self::Ogg),
            "audio/aac" => Ok(Self::Aac),
            "audio/webm" => Ok(Self::Webm),
            "audio/midi" => Ok(Self::Midi),
            // Everything Else
            // "application/octect-stream" => Ok(Self::Unknown),
            _ => Ok(Self::Unknown(s.to_lowercase())),
        }
    }
}

#[allow(unused)]
pub trait GetFileDetails {
    fn get_image_mime_type(&self) -> Option<MimeType>;
    fn get_audio_mime_type(&self) -> Option<MimeType>;
    fn get_mime_type(&self) -> Option<MimeType>;
    fn get_content_type(&self) -> Option<&str>;
    fn get_filename(&self) -> Option<String>;
}

impl GetFileDetails for MultipartPart {
    fn get_content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(|s| s.as_str())
    }
    fn get_image_mime_type(&self) -> Option<MimeType> {
        self.headers
            .get("content-type")
            .and_then(|mime| MimeType::from_str(mime).ok())
            .filter(|mime| mime.is_image())
    }
    fn get_audio_mime_type(&self) -> Option<MimeType> {
        self.headers
            .get("content-type")
            .and_then(|mime| MimeType::from_str(mime).ok())
            .filter(|mime| mime.is_audio())
    }
    fn get_mime_type(&self) -> Option<MimeType> {
        self.headers.get("content-type").and_then(|mime| MimeType::from_str(mime).ok())
    }
    fn get_filename(&self) -> Option<String> {
        // TODO: DRY this out
        Headers::parse_params(
            self.headers.get("content-disposition").unwrap().split_once(';').unwrap().1,
        )
        .get("filename")
        .map(|s| s.trim_matches('"').to_owned())
    }
}
