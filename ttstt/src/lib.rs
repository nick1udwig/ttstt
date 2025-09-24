// TTSTT - Text-to-Speech & Speech-to-Text Wrapper
// Provides a unified interface for multiple TTS/STT providers

use hyperprocess_macro::*;
use hyperware_process_lib::{
    our,
    homepage::add_to_homepage,
    hyperapp::SaveOptions,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

// Import OpenAI clients
use hyperware_openai_tts::{client::SpeechClient, types::{AudioFormat as OpenAIAudioFormat, TtsModel as OpenAITtsModel, Voice as OpenAIVoice}};
use hyperware_openai_stt::{client::TranscriptionClient, types::{Model as OpenAISttModel}};

// Provider abstraction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Provider {
    OpenAI,
    // Future: ElevenLabs, PlayAI, Groq
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    provider: Provider,
    api_key: String,
    is_default_tts: bool,
    is_default_stt: bool,
    default_voice: Option<String>,
    default_speed: Option<f32>,
}

// TTS Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TtsRequest {
    text: String,
    provider: Option<Provider>,
    voice: Option<String>,
    model: Option<String>,
    format: Option<String>,
    speed: Option<f32>,
    api_key: Option<String>, // For request authentication
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TtsResponse {
    audio_data: String, // Base64 encoded
    format: String,
    provider: Provider,
}

// STT Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SttRequest {
    audio_data: String, // Base64 encoded
    provider: Option<Provider>,
    model: Option<String>,
    language: Option<String>,
    api_key: Option<String>, // For request authentication
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SttResponse {
    text: String,
    provider: Provider,
}

// Storage Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestType {
    TTS,
    STT,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioTextPair {
    id: String,
    text: String,
    audio_data: String, // Base64 encoded
    audio_format: String,
    provider: Provider,
    timestamp: String,
    request_type: RequestType,
    metadata: Vec<(String, String)>, // Using Vec instead of HashMap for WIT compatibility
}

// API Key Management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiKeyRole {
    Admin,
    Requestor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiKey {
    key: String,
    role: ApiKeyRole,
    created_at: String,
    name: String,
}

// App State
#[derive(Default, Serialize, Deserialize)]
pub struct TtsttState {
    // Provider configurations
    providers: Vec<ProviderConfig>,
    
    // TTSTT API keys
    api_keys: Vec<ApiKey>,
    
    // Stored audio-text pairs
    audio_text_pairs: Vec<AudioTextPair>,
    
    // Settings
    default_tts_provider: Option<Provider>,
    default_stt_provider: Option<Provider>,
    
    // Admin key (generated on first init)
    admin_key: String,
}

// Helper methods (outside of hyperprocess impl block)
impl TtsttState {
    // Helper: Validate API key and check permissions
    fn validate_api_key(&self, api_key: Option<String>, require_admin: bool) -> Result<(), String> {
        let key = api_key.ok_or("API key required")?;
        
        let api_key_entry = self.api_keys.iter()
            .find(|k| k.key == key)
            .ok_or("Invalid API key")?;
        
        if require_admin && !matches!(api_key_entry.role, ApiKeyRole::Admin) {
            return Err("Admin permission required".to_string());
        }
        
        Ok(())
    }
    
    // Helper: Get provider config
    fn get_provider_config(&self, provider: &Provider) -> Result<&ProviderConfig, String> {
        self.providers.iter()
            .find(|p| p.provider == *provider)
            .ok_or_else(|| format!("Provider {:?} not configured", provider))
    }
    
    // OpenAI TTS implementation
    async fn handle_openai_tts(&self, request: TtsRequest) -> Result<TtsResponse, String> {
        let config = self.get_provider_config(&Provider::OpenAI)?;
        
        // Create OpenAI TTS client
        let client = SpeechClient::new(&config.api_key);
        
        // Map voice string to OpenAI voice enum, use provider default if not specified
        let voice_str = request.voice.as_deref()
            .or(config.default_voice.as_deref())
            .unwrap_or("nova");
        
        let voice = match voice_str {
            "alloy" => OpenAIVoice::Alloy,
            "ash" => OpenAIVoice::Ash,
            "ballad" => OpenAIVoice::Ballad,
            "coral" => OpenAIVoice::Coral,
            "echo" => OpenAIVoice::Echo,
            "fable" => OpenAIVoice::Fable,
            "onyx" => OpenAIVoice::Onyx,
            "nova" => OpenAIVoice::Nova,
            "sage" => OpenAIVoice::Sage,
            "shimmer" => OpenAIVoice::Shimmer,
            "verse" => OpenAIVoice::Verse,
            _ => OpenAIVoice::Nova, // Default to Nova
        };
        
        // Map model string to OpenAI model enum
        let model = match request.model.as_deref() {
            Some("tts-1") => OpenAITtsModel::Tts1,
            Some("tts-1-hd") => OpenAITtsModel::Tts1Hd,
            Some("gpt-4o-mini-tts") => OpenAITtsModel::Gpt4oMiniTts,
            _ => OpenAITtsModel::Gpt4oMiniTts, // Default to gpt-4o-mini-tts
        };
        
        // Map format string to OpenAI format enum
        let format = match request.format.as_deref() {
            Some("mp3") => OpenAIAudioFormat::Mp3,
            Some("opus") => OpenAIAudioFormat::Opus,
            Some("aac") => OpenAIAudioFormat::Aac,
            Some("flac") => OpenAIAudioFormat::Flac,
            Some("wav") => OpenAIAudioFormat::Wav,
            Some("pcm") => OpenAIAudioFormat::Pcm,
            _ => OpenAIAudioFormat::Mp3, // Default
        };
        
        // Build and execute request
        let mut builder = client.synthesize()
            .model(model)
            .voice(voice)
            .input(request.text.clone())
            .response_format(format);
        
        // Set speed: use request speed, then provider default, then 1.5
        let speed = request.speed
            .or(config.default_speed)
            .unwrap_or(1.5);
        builder = builder.speed(speed);
        
        let response = builder.execute()
            .await
            .map_err(|e| format!("OpenAI TTS error: {:?}", e))?;
        
        Ok(TtsResponse {
            audio_data: BASE64.encode(&response.audio_data),
            format: request.format.unwrap_or("mp3".to_string()),
            provider: Provider::OpenAI,
        })
    }
    
    // OpenAI STT implementation  
    async fn handle_openai_stt(&self, request: SttRequest) -> Result<SttResponse, String> {
        let config = self.get_provider_config(&Provider::OpenAI)?;
        
        // Create OpenAI STT client
        let client = TranscriptionClient::new(&config.api_key);
        
        // Decode base64 audio data
        let audio_data = BASE64.decode(&request.audio_data)
            .map_err(|e| format!("Failed to decode audio data: {}", e))?;
        
        // Map model string to OpenAI model enum
        let model = match request.model.as_deref() {
            Some("whisper-1") => OpenAISttModel::Whisper1,
            Some("gpt-4o-transcribe") => OpenAISttModel::Gpt4oTranscribe,
            Some("gpt-4o-mini-transcribe") => OpenAISttModel::Gpt4oMiniTranscribe,
            _ => OpenAISttModel::Whisper1, // Default
        };
        
        // Build and execute request
        let mut builder = client.transcribe()
            .file(audio_data, "audio.webm")
            .model(model);
        
        if let Some(lang) = request.language.clone() {
            builder = builder.language(lang);
        }
        
        let response = builder.execute().await
            .map_err(|e| format!("OpenAI STT error: {:?}", e))?;
        
        Ok(SttResponse {
            text: response.text,
            provider: Provider::OpenAI,
        })
    }
}

#[hyperprocess(
    name = "TTSTT",
    ui = Some(HttpBindingConfig::default()),
    endpoints = vec![
        Binding::Http { 
            path: "/api", 
            config: HttpBindingConfig::new(false, false, false, None) 
        }
    ],
    save_config = SaveOptions::EveryMessage,
    wit_world = "ttstt-dot-os-v0"
)]
impl TtsttState {
    #[init]
    async fn initialize(&mut self) {
        add_to_homepage("TTSTT", Some("🎙️"), Some("/"), None);
        
        // Generate initial admin key if not exists
        if self.admin_key.is_empty() {
            self.admin_key = format!("ttstt-admin-{}", Uuid::new_v4());
            println!("Generated admin API key: {}", self.admin_key);
            
            // Add to API keys list
            self.api_keys.push(ApiKey {
                key: self.admin_key.clone(),
                role: ApiKeyRole::Admin,
                created_at: Utc::now().to_rfc3339(),
                name: "Initial Admin Key".to_string(),
            });
        }
        
        let our_node = our().node.clone();
        println!("TTSTT initialized on node: {}", our_node);
    }
    
    // TTS endpoint
    #[http]
    async fn tts(&mut self, request_body: String) -> Result<String, String> {
        let request: TtsRequest = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        // Validate API key if provided
        if request.api_key.is_some() {
            self.validate_api_key(request.api_key.clone(), false)?;
        }
        
        // Determine provider
        let provider = request.provider.clone()
            .or(self.default_tts_provider.clone())
            .ok_or("No provider specified and no default configured")?;
        
        // Handle request based on provider
        let response = match provider {
            Provider::OpenAI => self.handle_openai_tts(request.clone()).await?,
        };
        
        // Store audio-text pair
        let pair = AudioTextPair {
            id: Uuid::new_v4().to_string(),
            text: request.text.clone(),
            audio_data: response.audio_data.clone(),
            audio_format: response.format.clone(),
            provider: response.provider.clone(),
            timestamp: Utc::now().to_rfc3339(),
            request_type: RequestType::TTS,
            metadata: vec![],
        };
        self.audio_text_pairs.push(pair);
        
        serde_json::to_string(&response)
            .map_err(|e| format!("Failed to serialize response: {}", e))
    }
    
    // STT endpoint
    #[http]
    async fn stt(&mut self, request_body: String) -> Result<String, String> {
        let request: SttRequest = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        // Validate API key if provided
        if request.api_key.is_some() {
            self.validate_api_key(request.api_key.clone(), false)?;
        }
        
        // Determine provider
        let provider = request.provider.clone()
            .or(self.default_stt_provider.clone())
            .ok_or("No provider specified and no default configured")?;
        
        // Handle request based on provider
        let response = match provider {
            Provider::OpenAI => self.handle_openai_stt(request.clone()).await?,
        };
        
        // Store audio-text pair
        let pair = AudioTextPair {
            id: Uuid::new_v4().to_string(),
            text: response.text.clone(),
            audio_data: request.audio_data.clone(),
            audio_format: "webm".to_string(), // Default for recorded audio
            provider: response.provider.clone(),
            timestamp: Utc::now().to_rfc3339(),
            request_type: RequestType::STT,
            metadata: vec![],
        };
        self.audio_text_pairs.push(pair);
        
        serde_json::to_string(&response)
            .map_err(|e| format!("Failed to serialize response: {}", e))
    }
    
    // Test TTS endpoint (for UI)
    #[http]
    async fn test_tts(&mut self, request_body: String) -> Result<String, String> {
        // Parse simple test request
        let test_request: serde_json::Value = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let text = test_request.get("text")
            .and_then(|v| v.as_str())
            .ok_or("Text field required")?;
        
        // Create TTS request with defaults
        let request = TtsRequest {
            text: text.to_string(),
            provider: self.default_tts_provider.clone(),
            voice: Some("nova".to_string()), // Default to nova
            model: Some("gpt-4o-mini-tts".to_string()), // Default to gpt-4o-mini-tts
            format: Some("mp3".to_string()),
            speed: Some(1.5), // Default to 1.5x speed
            api_key: None,
        };
        
        // Process request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        
        self.tts(request_json).await
    }
    
    // Test STT endpoint (for UI)
    #[http]
    async fn test_stt(&mut self, request_body: String) -> Result<String, String> {
        // Parse simple test request
        let test_request: serde_json::Value = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let audio_data = test_request.get("audioData")
            .and_then(|v| v.as_str())
            .ok_or("audioData field required")?;
        
        // Create STT request with defaults
        let request = SttRequest {
            audio_data: audio_data.to_string(),
            provider: self.default_stt_provider.clone(),
            model: None,
            language: None,
            api_key: None,
        };
        
        // Process request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        
        self.stt(request_json).await
    }
    
    // Provider management
    #[http]
    async fn add_provider(&mut self, request_body: String) -> Result<String, String> {
        // Parse request with API key
        let request: serde_json::Value = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let api_key = request.get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        self.validate_api_key(api_key, true)?;
        
        let config: ProviderConfig = serde_json::from_value(request.get("config")
            .ok_or("config field required")?
            .clone())
            .map_err(|e| format!("Invalid config: {}", e))?;
        
        // Remove existing config for this provider
        self.providers.retain(|p| p.provider != config.provider);
        
        // Update default providers if needed
        if config.is_default_tts {
            self.default_tts_provider = Some(config.provider.clone());
            // Clear other default TTS
            for provider in &mut self.providers {
                provider.is_default_tts = false;
            }
        }
        
        if config.is_default_stt {
            self.default_stt_provider = Some(config.provider.clone());
            // Clear other default STT
            for provider in &mut self.providers {
                provider.is_default_stt = false;
            }
        }
        
        self.providers.push(config);
        
        Ok("Provider added successfully".to_string())
    }
    
    #[http]
    async fn remove_provider(&mut self, request_body: String) -> Result<String, String> {
        let request: serde_json::Value = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let api_key = request.get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        self.validate_api_key(api_key, true)?;
        
        let provider_str = request.get("provider")
            .and_then(|v| v.as_str())
            .ok_or("provider field required")?;
        
        let provider: Provider = serde_json::from_value(serde_json::json!(provider_str))
            .map_err(|e| format!("Invalid provider: {}", e))?;
        
        self.providers.retain(|p| p.provider != provider);
        
        // Clear defaults if removed
        if self.default_tts_provider == Some(provider.clone()) {
            self.default_tts_provider = None;
        }
        if self.default_stt_provider == Some(provider) {
            self.default_stt_provider = None;
        }
        
        Ok("Provider removed successfully".to_string())
    }
    
    #[http]
    async fn get_providers(&self) -> Result<String, String> {
        // Return providers without API keys
        let safe_providers: Vec<_> = self.providers.iter()
            .map(|p| serde_json::json!({
                "provider": p.provider,
                "isDefaultTts": p.is_default_tts,
                "isDefaultStt": p.is_default_stt,
                "defaultVoice": p.default_voice,
                "defaultSpeed": p.default_speed,
            }))
            .collect();
        
        serde_json::to_string(&safe_providers)
            .map_err(|e| format!("Failed to serialize providers: {}", e))
    }
    
    #[http]
    async fn set_default_provider(&mut self, request_body: String) -> Result<String, String> {
        let request: serde_json::Value = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let api_key = request.get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        self.validate_api_key(api_key, true)?;
        
        let provider_str = request.get("provider")
            .and_then(|v| v.as_str())
            .ok_or("provider field required")?;
        
        let provider: Provider = serde_json::from_value(serde_json::json!(provider_str))
            .map_err(|e| format!("Invalid provider: {}", e))?;
        
        let provider_type = request.get("type")
            .and_then(|v| v.as_str())
            .ok_or("type field required (tts or stt)")?;
        
        // Ensure provider exists
        let exists = self.providers.iter().any(|p| p.provider == provider);
        if !exists {
            return Err("Provider not configured".to_string());
        }
        
        match provider_type {
            "tts" => {
                // Clear other defaults and set new one
                for p in &mut self.providers {
                    p.is_default_tts = p.provider == provider;
                }
                self.default_tts_provider = Some(provider);
            },
            "stt" => {
                // Clear other defaults and set new one
                for p in &mut self.providers {
                    p.is_default_stt = p.provider == provider;
                }
                self.default_stt_provider = Some(provider);
            },
            _ => return Err("Invalid type: must be 'tts' or 'stt'".to_string()),
        }
        
        Ok("Default provider set successfully".to_string())
    }
    
    // API Key management
    #[http]
    async fn generate_api_key(&mut self, request_body: String) -> Result<String, String> {
        let request: serde_json::Value = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let api_key = request.get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        self.validate_api_key(api_key, true)?;
        
        let name = request.get("name")
            .and_then(|v| v.as_str())
            .ok_or("name field required")?;
        
        let role_str = request.get("role")
            .and_then(|v| v.as_str())
            .ok_or("role field required")?;
        
        let role = match role_str {
            "Admin" => ApiKeyRole::Admin,
            "Requestor" => ApiKeyRole::Requestor,
            _ => return Err("Invalid role: must be 'Admin' or 'Requestor'".to_string()),
        };
        
        let new_key = ApiKey {
            key: format!("ttstt-{}-{}", 
                if matches!(role, ApiKeyRole::Admin) { "admin" } else { "req" },
                Uuid::new_v4()),
            role,
            created_at: Utc::now().to_rfc3339(),
            name: name.to_string(),
        };
        
        let key_value = new_key.key.clone();
        self.api_keys.push(new_key);
        
        // Return the newly generated key
        serde_json::to_string(&serde_json::json!({
            "key": key_value,
            "name": name,
            "role": role_str,
        }))
        .map_err(|e| format!("Failed to serialize response: {}", e))
    }
    
    #[http]
    async fn revoke_api_key(&mut self, request_body: String) -> Result<String, String> {
        let request: serde_json::Value = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let api_key = request.get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        self.validate_api_key(api_key, true)?;
        
        let key_to_revoke = request.get("keyToRevoke")
            .and_then(|v| v.as_str())
            .ok_or("keyToRevoke field required")?;
        
        // Don't allow revoking the initial admin key
        if key_to_revoke == self.admin_key {
            return Err("Cannot revoke initial admin key".to_string());
        }
        
        self.api_keys.retain(|k| k.key != key_to_revoke);
        
        Ok("API key revoked successfully".to_string())
    }
    
    #[http]
    async fn list_api_keys(&self, request_body: String) -> Result<String, String> {
        let request: serde_json::Value = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let api_key = request.get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        self.validate_api_key(api_key, true)?;
        
        // Return keys without actual key values
        let safe_keys: Vec<_> = self.api_keys.iter()
            .map(|k| serde_json::json!({
                "name": k.name,
                "role": k.role,
                "createdAt": k.created_at,
                "keyPreview": format!("{}...", &k.key[..20.min(k.key.len())]),
            }))
            .collect();
        
        serde_json::to_string(&safe_keys)
            .map_err(|e| format!("Failed to serialize keys: {}", e))
    }
    
    // Storage/History
    #[http]
    async fn get_history(&self, request_body: String) -> Result<String, String> {
        let request: serde_json::Value = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let limit = request.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;
        
        let offset = request.get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        
        let pairs: Vec<_> = self.audio_text_pairs.iter()
            .rev() // Most recent first
            .skip(offset)
            .take(limit)
            .collect();
        
        serde_json::to_string(&pairs)
            .map_err(|e| format!("Failed to serialize history: {}", e))
    }
    
    #[http]
    async fn get_audio_text_pair(&self, request_body: String) -> Result<String, String> {
        let request: serde_json::Value = serde_json::from_str(&request_body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let id = request.get("id")
            .and_then(|v| v.as_str())
            .ok_or("id field required")?;
        
        let pair = self.audio_text_pairs.iter()
            .find(|p| p.id == id)
            .ok_or("Audio-text pair not found")?;
        
        serde_json::to_string(pair)
            .map_err(|e| format!("Failed to serialize pair: {}", e))
    }
    
    // Get initial admin key (for first-time setup)
    #[http]
    async fn get_admin_key(&self) -> Result<String, String> {
        // Only return if no other admin keys exist
        let admin_count = self.api_keys.iter()
            .filter(|k| matches!(k.role, ApiKeyRole::Admin))
            .count();
        
        if admin_count == 1 {
            Ok(serde_json::json!({
                "adminKey": self.admin_key,
                "message": "Save this key! It will not be shown again."
            }).to_string())
        } else {
            Err("Admin key already retrieved".to_string())
        }
    }
}