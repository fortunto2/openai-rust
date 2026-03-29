// Image types — re-exported from openai-types.

pub use openai_types::image::*;

/// Extension trait for saving base64-encoded images to disk.
#[cfg(feature = "images")]
pub trait ImageSaveExt {
    /// Decode the base64 image data and save it to a file.
    fn save(&self, path: &std::path::Path) -> Result<(), crate::error::OpenAIError>;
}

#[cfg(feature = "images")]
impl ImageSaveExt for Image {
    fn save(&self, path: &std::path::Path) -> Result<(), crate::error::OpenAIError> {
        use std::io::Write;

        let b64 = self.b64_json.as_ref().ok_or_else(|| {
            crate::error::OpenAIError::InvalidArgument("No base64 data available".into())
        })?;
        let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
            .map_err(|e| {
                crate::error::OpenAIError::InvalidArgument(format!(
                    "Failed to decode base64: {}",
                    e
                ))
            })?;
        let mut file = std::fs::File::create(path).map_err(|e| {
            crate::error::OpenAIError::InvalidArgument(format!("Failed to create file: {}", e))
        })?;
        file.write_all(&bytes).map_err(|e| {
            crate::error::OpenAIError::InvalidArgument(format!("Failed to write file: {}", e))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_image_generate() {
        let req = ImageGenerateRequest::new("A cute cat")
            .model("dall-e-3")
            .n(1);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["prompt"], "A cute cat");
        assert_eq!(json["model"], "dall-e-3");
        assert_eq!(json["n"], 1);
    }

    #[test]
    fn test_serialize_image_generate_gpt_image_fields() {
        let req = ImageGenerateRequest::new("A dog")
            .model("gpt-image-1")
            .quality(ImageQuality::High)
            .output_format(ImageOutputFormat::Webp)
            .background(ImageBackground::Transparent);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-image-1");
        assert_eq!(json["quality"], "high");
        assert_eq!(json["output_format"], "webp");
        assert_eq!(json["background"], "transparent");
    }

    #[test]
    fn test_deserialize_images_response() {
        let json = r#"{
            "created": 1699012949,
            "data": [
                {"url": "https://example.com/image1.png"},
                {"url": "https://example.com/image2.png"}
            ]
        }"#;
        let resp: ImagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 2);
        assert!(resp.data[0].url.is_some());
    }

    #[test]
    fn test_deserialize_images_response_with_b64() {
        let json = r#"{
            "created": 1699012949,
            "data": [{"b64_json": "iVBORw0KGgo="}]
        }"#;
        let resp: ImagesResponse = serde_json::from_str(json).unwrap();
        assert!(resp.data[0].b64_json.is_some());
    }
}
