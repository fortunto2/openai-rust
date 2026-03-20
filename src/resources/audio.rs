// Audio resource — client.audio().transcriptions() / translations() / speech()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::audio::{
    SpeechRequest, Transcription, TranscriptionParams, Translation, TranslationParams,
};

/// Access audio endpoints.
pub struct Audio<'a> {
    client: &'a OpenAI,
}

impl<'a> Audio<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Access transcription endpoints.
    pub fn transcriptions(&self) -> Transcriptions<'_> {
        Transcriptions {
            client: self.client,
        }
    }

    /// Access translation endpoints.
    pub fn translations(&self) -> Translations<'_> {
        Translations {
            client: self.client,
        }
    }

    /// Access speech endpoints.
    pub fn speech(&self) -> Speech<'_> {
        Speech {
            client: self.client,
        }
    }
}

/// Audio transcription endpoint.
pub struct Transcriptions<'a> {
    client: &'a OpenAI,
}

impl<'a> Transcriptions<'a> {
    /// Transcribe audio to text.
    ///
    /// `POST /audio/transcriptions`
    pub async fn create(&self, params: TranscriptionParams) -> Result<Transcription, OpenAIError> {
        let mut form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(params.file).file_name(params.filename),
            )
            .text("model", params.model);

        if let Some(lang) = params.language {
            form = form.text("language", lang);
        }
        if let Some(prompt) = params.prompt {
            form = form.text("prompt", prompt);
        }
        if let Some(fmt) = params.response_format {
            form = form.text("response_format", fmt);
        }
        if let Some(temp) = params.temperature {
            form = form.text("temperature", temp.to_string());
        }

        self.client
            .post_multipart("/audio/transcriptions", form)
            .await
    }
}

/// Audio translation endpoint.
pub struct Translations<'a> {
    client: &'a OpenAI,
}

impl<'a> Translations<'a> {
    /// Translate audio to English text.
    ///
    /// `POST /audio/translations`
    pub async fn create(&self, params: TranslationParams) -> Result<Translation, OpenAIError> {
        let mut form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(params.file).file_name(params.filename),
            )
            .text("model", params.model);

        if let Some(prompt) = params.prompt {
            form = form.text("prompt", prompt);
        }
        if let Some(fmt) = params.response_format {
            form = form.text("response_format", fmt);
        }
        if let Some(temp) = params.temperature {
            form = form.text("temperature", temp.to_string());
        }

        self.client
            .post_multipart("/audio/translations", form)
            .await
    }
}

/// Audio speech endpoint.
pub struct Speech<'a> {
    client: &'a OpenAI,
}

impl<'a> Speech<'a> {
    /// Generate audio from text.
    ///
    /// `POST /audio/speech`
    ///
    /// Returns raw audio bytes.
    pub async fn create(&self, request: SpeechRequest) -> Result<bytes::Bytes, OpenAIError> {
        self.client.post_raw("/audio/speech", &request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::audio::{SpeechRequest, TranscriptionParams, TranslationParams};

    #[tokio::test]
    async fn test_audio_transcription_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/audio/transcriptions")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"text": "Hello world from audio"}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let params = TranscriptionParams::new(
            vec![0xFF, 0xFB, 0x90], // fake MP3 header
            "audio.mp3",
            "whisper-1",
        );

        let response = client
            .audio()
            .transcriptions()
            .create(params)
            .await
            .unwrap();
        assert_eq!(response.text, "Hello world from audio");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_audio_translation_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/audio/translations")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"text": "Hello world in English"}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let params = TranslationParams::new(vec![0xFF, 0xFB, 0x90], "audio.mp3", "whisper-1");

        let response = client.audio().translations().create(params).await.unwrap();
        assert_eq!(response.text, "Hello world in English");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_audio_speech_create() {
        let mut server = mockito::Server::new_async().await;
        let audio_bytes = vec![0xFF, 0xFB, 0x90, 0x00]; // fake audio
        let mock = server
            .mock("POST", "/audio/speech")
            .match_header("authorization", "Bearer sk-test")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "audio/mpeg")
            .with_body(audio_bytes.clone())
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let request = SpeechRequest::new("Hello world", "tts-1", "alloy");

        let response = client.audio().speech().create(request).await.unwrap();
        assert_eq!(response.as_ref(), audio_bytes.as_slice());
        mock.assert_async().await;
    }
}
