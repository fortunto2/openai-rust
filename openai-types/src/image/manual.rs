// Manual: hand-crafted image types (request/response builders, enums, save helper).

use serde::{Deserialize, Serialize};

/// Image quality level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum ImageQuality {
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "hd")]
    Hd,
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "auto")]
    Auto,
}

/// Image dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum ImageSize {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "1024x1024")]
    S1024x1024,
    #[serde(rename = "1536x1024")]
    S1536x1024,
    #[serde(rename = "1024x1536")]
    S1024x1536,
    #[serde(rename = "256x256")]
    S256x256,
    #[serde(rename = "512x512")]
    S512x512,
    #[serde(rename = "1792x1024")]
    S1792x1024,
    #[serde(rename = "1024x1792")]
    S1024x1792,
}

/// Image style (dall-e-3 only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum ImageStyle {
    #[serde(rename = "vivid")]
    Vivid,
    #[serde(rename = "natural")]
    Natural,
}

/// Output format for generated images (GPT image models).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum ImageOutputFormat {
    #[serde(rename = "png")]
    Png,
    #[serde(rename = "jpeg")]
    Jpeg,
    #[serde(rename = "webp")]
    Webp,
}

/// Response format for images.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum ImageResponseFormat {
    #[serde(rename = "url")]
    Url,
    #[serde(rename = "b64_json")]
    B64Json,
}

/// Background transparency for generated images (GPT image models).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum ImageBackground {
    #[serde(rename = "transparent")]
    Transparent,
    #[serde(rename = "opaque")]
    Opaque,
    #[serde(rename = "auto")]
    Auto,
}

/// Content moderation level for image generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum ImageModeration {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "auto")]
    Auto,
}

// -- Request types --

/// Request body for `POST /images/generations`.
#[derive(Debug, Clone, Serialize)]
pub struct ImageGenerateRequest {
    /// Text description of desired image(s).
    pub prompt: String,

    /// Model to use (e.g. "dall-e-3", "gpt-image-1").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Number of images to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,

    /// Quality level (standard or hd for dall-e-3; low/medium/high/auto for gpt-image-1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ImageQuality>,

    /// Image dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,

    /// Response format (url or b64_json).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ImageResponseFormat>,

    /// Style (vivid or natural) -- dall-e-3 only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ImageStyle>,

    /// A unique identifier representing your end-user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    // GPT-image-1 specific fields
    /// Output format (png, jpeg, webp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<ImageOutputFormat>,

    /// Output compression quality (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<i64>,

    /// Background style (transparent, opaque, auto).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<ImageBackground>,

    /// Moderation level (low, auto).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation: Option<ImageModeration>,
}

impl ImageGenerateRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            n: None,
            quality: None,
            size: None,
            response_format: None,
            style: None,
            user: None,
            output_format: None,
            output_compression: None,
            background: None,
            moderation: None,
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn n(mut self, n: i64) -> Self {
        self.n = Some(n);
        self
    }

    pub fn quality(mut self, quality: ImageQuality) -> Self {
        self.quality = Some(quality);
        self
    }

    pub fn size(mut self, size: ImageSize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn response_format(mut self, response_format: ImageResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    pub fn style(mut self, style: ImageStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn output_format(mut self, output_format: ImageOutputFormat) -> Self {
        self.output_format = Some(output_format);
        self
    }

    pub fn output_compression(mut self, output_compression: i64) -> Self {
        self.output_compression = Some(output_compression);
        self
    }

    pub fn background(mut self, background: ImageBackground) -> Self {
        self.background = Some(background);
        self
    }

    pub fn moderation(mut self, moderation: ImageModeration) -> Self {
        self.moderation = Some(moderation);
        self
    }
}

/// Request body for `POST /images/edits`.
#[derive(Debug, Clone, Serialize)]
pub struct ImageEditRequest {
    /// Image to edit (multipart file).
    #[serde(skip_serializing)]
    pub image: Vec<u8>,
    #[serde(skip_serializing)]
    pub image_filename: String,

    /// Prompt describing the edit.
    pub prompt: String,

    /// Additional image for reference (optional).
    #[serde(skip_serializing)]
    pub mask: Option<Vec<u8>>,
    #[serde(skip_serializing)]
    pub mask_filename: Option<String>,

    /// Model to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Number of images to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,

    /// Image dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,

    /// Response format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ImageResponseFormat>,

    /// A unique identifier representing your end-user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl ImageEditRequest {
    pub fn new(
        image: Vec<u8>,
        image_filename: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            image,
            image_filename: image_filename.into(),
            prompt: prompt.into(),
            mask: None,
            mask_filename: None,
            model: None,
            n: None,
            size: None,
            response_format: None,
            user: None,
        }
    }
}

/// Backward compatibility alias.
pub type ImageEditParams = ImageEditRequest;

/// Request body for `POST /images/variations`.
#[derive(Debug, Clone, Serialize)]
pub struct ImageVariationRequest {
    /// Image to vary (multipart file).
    #[serde(skip_serializing)]
    pub image: Vec<u8>,
    #[serde(skip_serializing)]
    pub image_filename: String,

    /// Number of images to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,

    /// Image dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,

    /// Response format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ImageResponseFormat>,

    /// A unique identifier representing your end-user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl ImageVariationRequest {
    pub fn new(image: Vec<u8>, image_filename: impl Into<String>) -> Self {
        Self {
            image,
            image_filename: image_filename.into(),
            n: None,
            size: None,
            response_format: None,
            user: None,
        }
    }
}

/// Backward compatibility alias.
pub type ImageVariationParams = ImageVariationRequest;

// -- Response types --

/// A single generated image.
#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    /// The URL of the generated image (if response_format is url).
    #[serde(default)]
    pub url: Option<String>,
    /// The base64-encoded JSON of the generated image (if response_format is b64_json).
    #[serde(default)]
    pub b64_json: Option<String>,
    /// The prompt that was used to generate the image (dall-e-3 only).
    #[serde(default)]
    pub revised_prompt: Option<String>,
}

/// Response from image generation endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct ImagesResponse {
    pub created: i64,
    pub data: Vec<Image>,
}
