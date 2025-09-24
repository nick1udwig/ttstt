// TTSTT Type Definitions

export type Provider = 'OpenAI'; // Future: | 'ElevenLabs' | 'PlayAI' | 'Groq'

export interface ProviderConfig {
  provider: Provider;
  apiKey?: string;
  isDefaultTts: boolean;
  isDefaultStt: boolean;
  defaultVoice?: string;
  defaultSpeed?: number;
}

export interface TtsRequest {
  text: string;
  provider?: Provider;
  voice?: string;
  model?: string;
  format?: string;
}

export interface TtsResponse {
  audio_data: string; // Base64 encoded
  format: string;
  provider: Provider;
}

export interface SttRequest {
  audioData: string; // Base64 encoded
  provider?: Provider;
  model?: string;
  language?: string;
}

export interface SttResponse {
  text: string;
  provider: Provider;
}

export interface ApiKey {
  key?: string;
  role: 'Admin' | 'Requestor';
  createdAt: string;
  name: string;
  keyPreview?: string;
}

export interface AudioTextPair {
  id: string;
  text: string;
  audio_data: string; // Base64 encoded
  audio_format: string;
  provider: Provider;
  timestamp: string;
  request_type: 'TTS' | 'STT';
}

export interface AdminKeyResponse {
  adminKey: string;
  message: string;
}