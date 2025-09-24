# TTSTT Implementation Plan

## Overview
TTSTT (Text-to-Speech & Speech-to-Text) is a Hyperware application that acts as a wrapper/proxy for various TTS and STT providers. It provides a unified interface for handling speech synthesis and transcription requests while managing provider API keys and storing the resulting audio-text pairs.

## Architecture Overview

### Core Components

1. **Provider System**: Abstracted interface for TTS/STT providers
   - Initially supports OpenAI
   - Designed for easy extension with ElevenLabs, Play.AI, Groq, etc.
   
2. **API Key Management**: Two-tier system
   - Provider API keys (for external services like OpenAI)
   - TTSTT API keys (for client authentication - admin and requestor roles)
   
3. **Request Processing**: Handles incoming TTS/STT requests and routes to providers
   
4. **Storage System**: Persists audio-text pairs and request metadata
   
5. **Web UI**: Three main pages for testing, configuration, and key management

## Detailed Implementation Steps

### Phase 1: Backend Core Structure

#### 1.1 Update Project Structure
- Rename `skeleton-app` to `ttstt` throughout the project
- Update `Cargo.toml`, `metadata.json`, and folder names accordingly
- Update the app name in UI components

#### 1.2 Define Core Types and State

Create the following type hierarchy in `ttstt/src/lib.rs`:

```rust
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
}

// TTS Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TtsRequest {
    text: String,
    provider: Option<Provider>,
    voice: Option<String>,
    model: Option<String>,
    format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TtsResponse {
    audio_data: Vec<u8>,
    format: String,
    provider: Provider,
}

// STT Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SttRequest {
    audio_data: Vec<u8>,
    provider: Option<Provider>,
    model: Option<String>,
    language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SttResponse {
    text: String,
    provider: Provider,
}

// Storage Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioTextPair {
    id: String,
    text: String,
    audio_data: Vec<u8>,
    audio_format: String,
    provider: Provider,
    timestamp: String,
    request_type: RequestType,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestType {
    TTS,
    STT,
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
}
```

#### 1.3 Add Dependencies

Add to `ttstt/Cargo.toml`:
```toml
[dependencies]
# Existing dependencies...
hyperware-openai-tts = { path = "../../hyperware-openai-ttstt/hyperware-openai-tts" }
hyperware-openai-stt = { path = "../../hyperware-openai-ttstt/hyperware-openai-stt" }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = "0.4"
base64 = "0.21"
```

#### 1.4 Implement Provider Trait System

Create a trait for provider abstraction:

```rust
#[async_trait::async_trait]
trait TtsProvider {
    async fn synthesize(&self, request: TtsRequest) -> Result<TtsResponse, String>;
}

#[async_trait::async_trait]
trait SttProvider {
    async fn transcribe(&self, request: SttRequest) -> Result<SttResponse, String>;
}
```

#### 1.5 Implement OpenAI Provider

Create OpenAI provider implementation using the provided crates:

```rust
struct OpenAIProvider {
    api_key: String,
}

impl OpenAIProvider {
    async fn handle_tts(&self, request: TtsRequest) -> Result<TtsResponse, String> {
        // Use hyperware-openai-tts crate
        // Convert TtsRequest to OpenAI format
        // Call SpeechClient
        // Return TtsResponse
    }
    
    async fn handle_stt(&self, request: SttRequest) -> Result<SttResponse, String> {
        // Use hyperware-openai-stt crate
        // Convert SttRequest to OpenAI format
        // Call TranscriptionClient
        // Return SttResponse
    }
}
```

### Phase 2: HTTP Endpoints

Implement the following endpoints in `ttstt/src/lib.rs`:

#### 2.1 TTS/STT Request Endpoints
```rust
#[http]
async fn tts(&mut self, request_body: String) -> Result<String, String>
// Handles TTS requests with optional API key authentication

#[http]
async fn stt(&mut self, request_body: String) -> Result<String, String>
// Handles STT requests with optional API key authentication
```

#### 2.2 Configuration Endpoints
```rust
#[http]
async fn add_provider(&mut self, request_body: String) -> Result<String, String>
// Admin only: Add/update provider configuration

#[http]
async fn remove_provider(&mut self, request_body: String) -> Result<String, String>
// Admin only: Remove provider configuration

#[http]
async fn get_providers(&self) -> Result<String, String>
// Get list of configured providers (without API keys)

#[http]
async fn set_default_provider(&mut self, request_body: String) -> Result<String, String>
// Admin only: Set default TTS/STT provider
```

#### 2.3 API Key Management Endpoints
```rust
#[http]
async fn generate_api_key(&mut self, request_body: String) -> Result<String, String>
// Admin only: Generate new TTSTT API key

#[http]
async fn revoke_api_key(&mut self, request_body: String) -> Result<String, String>
// Admin only: Revoke TTSTT API key

#[http]
async fn list_api_keys(&self) -> Result<String, String>
// Admin only: List all API keys (without actual key values)
```

#### 2.4 Storage/History Endpoints
```rust
#[http]
async fn get_history(&self, request_body: String) -> Result<String, String>
// Get audio-text pair history with pagination

#[http]
async fn get_audio_text_pair(&self, request_body: String) -> Result<String, String>
// Get specific audio-text pair by ID
```

#### 2.5 Testing Endpoints
```rust
#[http]
async fn test_tts(&mut self, request_body: String) -> Result<String, String>
// Direct TTS test endpoint for UI

#[http]
async fn test_stt(&mut self, request_body: String) -> Result<String, String>
// Direct STT test endpoint for UI
```

### Phase 3: Authentication & Authorization

Implement authentication middleware:

1. Check for API key in request headers or body
2. Validate API key against stored keys
3. Check role permissions for admin-only endpoints
4. Allow unauthenticated access to test endpoints (configurable)

### Phase 4: Frontend Implementation

#### 4.1 Update UI Structure

Rename and restructure the UI to have three main pages:

1. **Test Page** (`/test`)
   - TTS testing section with text input and audio playback
   - STT testing section with hold-to-record functionality
   
2. **Settings Page** (`/settings`)
   - Provider configuration (add/remove/edit)
   - Default provider selection
   - API key input fields
   
3. **API Keys Page** (`/keys`)
   - Generate new TTSTT API keys
   - View existing keys
   - Revoke keys

#### 4.2 Create Type Definitions

In `ui/src/types/ttstt.ts`:
```typescript
export interface Provider {
  provider: 'OpenAI' // | 'ElevenLabs' | 'PlayAI' | 'Groq'
  apiKey?: string
  isDefaultTts: boolean
  isDefaultStt: boolean
}

export interface TtsRequest {
  text: string
  provider?: string
  voice?: string
  model?: string
  format?: string
}

export interface SttRequest {
  audioData: string // base64
  provider?: string
  model?: string
  language?: string
}

export interface ApiKey {
  key?: string
  role: 'Admin' | 'Requestor'
  createdAt: string
  name: string
}
```

#### 4.3 Implement Store

Create Zustand store in `ui/src/store/ttstt.ts`:
```typescript
interface TtsttStore {
  // State
  providers: Provider[]
  apiKeys: ApiKey[]
  isRecording: boolean
  audioUrl: string | null
  transcribedText: string | null
  
  // Actions
  loadProviders: () => Promise<void>
  addProvider: (provider: Provider) => Promise<void>
  removeProvider: (provider: string) => Promise<void>
  setDefaultProvider: (provider: string, type: 'tts' | 'stt') => Promise<void>
  
  generateApiKey: (name: string, role: string) => Promise<void>
  revokeApiKey: (key: string) => Promise<void>
  
  testTts: (text: string) => Promise<void>
  startRecording: () => void
  stopRecording: () => Promise<void>
}
```

#### 4.4 Implement Components

1. **TestPage Component**
   - TTS test section with textarea and submit button
   - Audio player for TTS results
   - STT test section with hold-to-record button
   - Display area for transcribed text

2. **SettingsPage Component**
   - Form for adding new providers
   - List of existing providers with edit/delete
   - Default provider selection dropdowns

3. **ApiKeysPage Component**
   - Form for generating new keys (name, role)
   - List of existing keys with revoke buttons
   - Copy-to-clipboard functionality for new keys

#### 4.5 Audio Recording Implementation

Use Web Audio API for recording:
```typescript
class AudioRecorder {
  private mediaRecorder: MediaRecorder | null = null
  private chunks: Blob[] = []
  
  async startRecording() {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
    this.mediaRecorder = new MediaRecorder(stream)
    this.chunks = []
    
    this.mediaRecorder.ondataavailable = (e) => {
      this.chunks.push(e.data)
    }
    
    this.mediaRecorder.start()
  }
  
  async stopRecording(): Promise<Blob> {
    return new Promise((resolve) => {
      this.mediaRecorder.onstop = () => {
        const blob = new Blob(this.chunks, { type: 'audio/webm' })
        resolve(blob)
      }
      this.mediaRecorder.stop()
    })
  }
}
```

### Phase 5: Integration & Testing

1. **Build Process**
   - Run `kit build --hyperapp` to generate WIT bindings
   - This creates the API interface in `ui/target/caller-utils.ts`
   
2. **API Integration**
   - Use generated `caller-utils.ts` for all backend calls
   - Handle base64 encoding/decoding for audio data
   
3. **Error Handling**
   - Implement proper error boundaries in React
   - Display user-friendly error messages
   - Log errors for debugging

4. **Testing Checklist**
   - Test TTS with various text inputs
   - Test STT with microphone recording
   - Test provider configuration changes
   - Test API key generation and authentication
   - Test switching between providers
   - Test audio playback functionality

## Security Considerations

1. **API Key Storage**
   - Never expose provider API keys in responses
   - Hash TTSTT API keys before storage
   - Use secure random generation for API keys

2. **Authentication**
   - Implement rate limiting
   - Validate all inputs
   - Check permissions for admin endpoints

3. **Data Privacy**
   - Option to disable storage of audio-text pairs
   - Implement data retention policies
   - Secure handling of audio data

## Future Extensions

The architecture is designed to easily support:

1. **Additional Providers**
   - ElevenLabs (premium voices)
   - Play.AI (interactive voices)
   - Groq (fast inference)
   - Local models (Whisper, Coqui)

2. **Advanced Features**
   - Voice cloning
   - Real-time streaming
   - Language detection
   - Speaker diarization
   - Batch processing

3. **Monitoring & Analytics**
   - Usage statistics
   - Cost tracking per provider
   - Performance metrics
   - Error rates

## Implementation Order

1. **Backend Core** (Phase 1)
   - Set up types and state
   - Implement OpenAI provider
   - Basic request handling

2. **HTTP Endpoints** (Phase 2)
   - Implement all endpoints
   - Test with curl/Postman

3. **Frontend Basic** (Phase 4.1-4.3)
   - Create pages and routing
   - Implement store
   - Basic UI components

4. **Integration** (Phase 5)
   - Connect frontend to backend
   - Test end-to-end flows

5. **Authentication** (Phase 3)
   - Add API key validation
   - Implement role-based access

6. **Polish & Testing**
   - Error handling
   - UI improvements
   - Comprehensive testing

## Notes for Implementor

- Start by examining the example apps in `resources/example-apps/` for patterns
- Use `resources/guides/` for specific implementation details
- The OpenAI TTS/STT crates are in `/home/nick/git/hyperware-openai-ttstt/`
- Remember to build with `kit build --hyperapp` after backend changes
- The generated API will be in `ui/target/caller-utils.ts`
- Use the generated caller-utils functions for all backend communication
- Don't manually create fetch requests to the backend