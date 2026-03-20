// Images resource — client.images().generate() / edit() / create_variation()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::image::{
    ImageEditParams, ImageGenerateRequest, ImageVariationParams, ImagesResponse,
};

/// Access image endpoints.
pub struct Images<'a> {
    client: &'a OpenAI,
}

impl<'a> Images<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Generate images from a text prompt.
    ///
    /// `POST /images/generations`
    pub async fn generate(
        &self,
        request: ImageGenerateRequest,
    ) -> Result<ImagesResponse, OpenAIError> {
        self.client.post("/images/generations", &request).await
    }

    /// Create an edited image from a source image and prompt (multipart upload).
    ///
    /// `POST /images/edits`
    pub async fn edit(&self, params: ImageEditParams) -> Result<ImagesResponse, OpenAIError> {
        let mut form = reqwest::multipart::Form::new()
            .part(
                "image",
                reqwest::multipart::Part::bytes(params.image).file_name(params.image_filename),
            )
            .text("prompt", params.prompt);

        if let Some(m) = params.model {
            form = form.text("model", m);
        }
        if let Some((mask_bytes, mask_name)) = params.mask {
            form = form.part(
                "mask",
                reqwest::multipart::Part::bytes(mask_bytes).file_name(mask_name),
            );
        }
        if let Some(n) = params.n {
            form = form.text("n", n.to_string());
        }
        if let Some(s) = params.size {
            form = form.text("size", s);
        }

        self.client.post_multipart("/images/edits", form).await
    }

    /// Create a variation of a given image (multipart upload).
    ///
    /// `POST /images/variations`
    pub async fn create_variation(
        &self,
        params: ImageVariationParams,
    ) -> Result<ImagesResponse, OpenAIError> {
        let mut form = reqwest::multipart::Form::new().part(
            "image",
            reqwest::multipart::Part::bytes(params.image).file_name(params.image_filename),
        );

        if let Some(m) = params.model {
            form = form.text("model", m);
        }
        if let Some(n) = params.n {
            form = form.text("n", n.to_string());
        }
        if let Some(s) = params.size {
            form = form.text("size", s);
        }

        self.client.post_multipart("/images/variations", form).await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::image::{ImageGenerateRequest, ImageVariationParams};

    #[tokio::test]
    async fn test_images_generate() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/images/generations")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "created": 1589478378,
                    "data": [
                        {"url": "https://example.com/image.png", "revised_prompt": "A cute otter"}
                    ]
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let mut request = ImageGenerateRequest::new("A cute baby sea otter");
        request.model = Some("dall-e-3".into());

        let response = client.images().generate(request).await.unwrap();
        assert_eq!(response.created, 1589478378);
        let data = response.data.unwrap();
        assert_eq!(data.len(), 1);
        assert!(data[0].url.is_some());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_images_create_variation() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/images/variations")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "created": 1589478378,
                    "data": [{"url": "https://example.com/variation.png"}]
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let params = ImageVariationParams::new(vec![0x89, 0x50, 0x4E, 0x47], "image.png");

        let response = client.images().create_variation(params).await.unwrap();
        assert_eq!(response.data.unwrap().len(), 1);
        mock.assert_async().await;
    }
}
