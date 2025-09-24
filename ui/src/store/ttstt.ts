// TTSTT Store

import { create } from 'zustand';
import * as api from '../utils/api';
import { Provider, ProviderConfig, ApiKey, AudioTextPair } from '../types/ttstt';

// Audio Recorder class for STT
class AudioRecorder {
  private mediaRecorder: MediaRecorder | null = null;
  private chunks: Blob[] = [];
  
  async startRecording(): Promise<void> {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    this.mediaRecorder = new MediaRecorder(stream);
    this.chunks = [];
    
    this.mediaRecorder.ondataavailable = (e) => {
      this.chunks.push(e.data);
    };
    
    this.mediaRecorder.start();
  }
  
  async stopRecording(): Promise<Blob> {
    return new Promise((resolve) => {
      if (!this.mediaRecorder) {
        resolve(new Blob());
        return;
      }
      
      this.mediaRecorder.onstop = () => {
        const blob = new Blob(this.chunks, { type: 'audio/webm' });
        resolve(blob);
        
        // Stop all tracks to release microphone
        this.mediaRecorder?.stream.getTracks().forEach(track => track.stop());
      };
      
      this.mediaRecorder.stop();
    });
  }
}

interface TtsttStore {
  // State
  providers: ProviderConfig[];
  apiKeys: ApiKey[];
  history: AudioTextPair[];
  isRecording: boolean;
  audioUrl: string | null;
  transcribedText: string | null;
  adminKey: string | null;
  isLoading: boolean;
  error: string | null;
  recorder: AudioRecorder | null;
  
  // Actions - Setup
  getAdminKey: () => Promise<void>;
  setAdminKey: (key: string) => void;
  
  // Actions - Providers
  loadProviders: () => Promise<void>;
  addProvider: (config: ProviderConfig) => Promise<void>;
  removeProvider: (provider: Provider) => Promise<void>;
  setDefaultProvider: (provider: Provider, type: 'tts' | 'stt') => Promise<void>;
  
  // Actions - API Keys
  loadApiKeys: () => Promise<void>;
  generateApiKey: (name: string, role: 'Admin' | 'Requestor') => Promise<string>;
  revokeApiKey: (key: string) => Promise<void>;
  
  // Actions - TTS/STT
  testTts: (text: string) => Promise<void>;
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<void>;
  
  // Actions - History
  loadHistory: (limit?: number, offset?: number) => Promise<void>;
  
  // Utility actions
  clearError: () => void;
  setError: (error: string) => void;
}

const useTtsttStore = create<TtsttStore>((set, get) => ({
  // Initial state
  providers: [],
  apiKeys: [],
  history: [],
  isRecording: false,
  audioUrl: null,
  transcribedText: null,
  adminKey: null,
  isLoading: false,
  error: null,
  recorder: null,
  
  // Get initial admin key
  getAdminKey: async () => {
    try {
      set({ isLoading: true, error: null });
      const response = await api.getAdminKey();
      const data = JSON.parse(response);
      set({ adminKey: data.adminKey });
    } catch (error) {
      // Admin key already retrieved or error
      console.log('Admin key not available');
    } finally {
      set({ isLoading: false });
    }
  },
  
  setAdminKey: (key: string) => {
    set({ adminKey: key });
  },
  
  // Provider management
  loadProviders: async () => {
    try {
      set({ isLoading: true, error: null });
      const response = await api.getProviders();
      const providers = JSON.parse(response);
      set({ providers });
    } catch (error: any) {
      const errorMessage = error?.details || error?.message || error;
      set({ error: `Failed to load providers: ${errorMessage}` });
    } finally {
      set({ isLoading: false });
    }
  },
  
  addProvider: async (config: ProviderConfig) => {
    try {
      set({ isLoading: true, error: null });
      const adminKey = get().adminKey;
      if (!adminKey) throw new Error('Admin key required');
      
      await api.addProvider(JSON.stringify({
        apiKey: adminKey,
        config: {
          provider: config.provider,
          api_key: config.apiKey,
          is_default_tts: config.isDefaultTts,
          is_default_stt: config.isDefaultStt,
        },
      }));
      
      await get().loadProviders();
    } catch (error: any) {
      const errorMessage = error?.details || error?.message || error;
      set({ error: `Failed to add provider: ${errorMessage}` });
    } finally {
      set({ isLoading: false });
    }
  },
  
  removeProvider: async (provider: Provider) => {
    try {
      set({ isLoading: true, error: null });
      const adminKey = get().adminKey;
      if (!adminKey) throw new Error('Admin key required');
      
      await api.removeProvider(JSON.stringify({
        apiKey: adminKey,
        provider,
      }));
      
      await get().loadProviders();
    } catch (error) {
      set({ error: `Failed to remove provider: ${error}` });
    } finally {
      set({ isLoading: false });
    }
  },
  
  setDefaultProvider: async (provider: Provider, type: 'tts' | 'stt') => {
    try {
      set({ isLoading: true, error: null });
      const adminKey = get().adminKey;
      if (!adminKey) throw new Error('Admin key required');
      
      await api.setDefaultProvider(JSON.stringify({
        apiKey: adminKey,
        provider,
        type,
      }));
      
      await get().loadProviders();
    } catch (error) {
      set({ error: `Failed to set default provider: ${error}` });
    } finally {
      set({ isLoading: false });
    }
  },
  
  // API Key management
  loadApiKeys: async () => {
    try {
      set({ isLoading: true, error: null });
      const adminKey = get().adminKey;
      if (!adminKey) throw new Error('Admin key required');
      
      const response = await api.listApiKeys(JSON.stringify({ apiKey: adminKey }));
      const apiKeys = JSON.parse(response);
      set({ apiKeys });
    } catch (error) {
      set({ error: `Failed to load API keys: ${error}` });
    } finally {
      set({ isLoading: false });
    }
  },
  
  generateApiKey: async (name: string, role: 'Admin' | 'Requestor') => {
    try {
      set({ isLoading: true, error: null });
      const adminKey = get().adminKey;
      if (!adminKey) throw new Error('Admin key required');
      
      const response = await api.generateApiKey(JSON.stringify({
        apiKey: adminKey,
        name,
        role,
      }));
      
      const data = JSON.parse(response);
      await get().loadApiKeys();
      return data.key;
    } catch (error) {
      set({ error: `Failed to generate API key: ${error}` });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },
  
  revokeApiKey: async (keyToRevoke: string) => {
    try {
      set({ isLoading: true, error: null });
      const adminKey = get().adminKey;
      if (!adminKey) throw new Error('Admin key required');
      
      await api.revokeApiKey(JSON.stringify({
        apiKey: adminKey,
        keyToRevoke,
      }));
      
      await get().loadApiKeys();
    } catch (error) {
      set({ error: `Failed to revoke API key: ${error}` });
    } finally {
      set({ isLoading: false });
    }
  },
  
  // TTS testing
  testTts: async (text: string) => {
    try {
      set({ isLoading: true, error: null, audioUrl: null });
      
      const response = await api.testTts(JSON.stringify({ text }));
      const data = JSON.parse(response);
      
      // Convert base64 to blob URL
      const audioData = atob(data.audio_data);
      const audioArray = new Uint8Array(audioData.length);
      for (let i = 0; i < audioData.length; i++) {
        audioArray[i] = audioData.charCodeAt(i);
      }
      
      const blob = new Blob([audioArray], { type: `audio/${data.format}` });
      const url = URL.createObjectURL(blob);
      
      set({ audioUrl: url });
    } catch (error: any) {
      // Extract the actual error message from the API response
      const errorMessage = error?.details || error?.message || error;
      set({ error: `TTS failed: ${errorMessage}` });
    } finally {
      set({ isLoading: false });
    }
  },
  
  // STT testing
  startRecording: async () => {
    try {
      const recorder = new AudioRecorder();
      await recorder.startRecording();
      set({ isRecording: true, recorder, transcribedText: null, error: null });
    } catch (error) {
      set({ error: `Failed to start recording: ${error}` });
    }
  },
  
  stopRecording: async () => {
    try {
      set({ isLoading: true });
      const { recorder } = get();
      
      if (!recorder) {
        throw new Error('No recorder available');
      }
      
      const audioBlob = await recorder.stopRecording();
      
      // Convert blob to base64
      const reader = new FileReader();
      reader.readAsDataURL(audioBlob);
      
      reader.onloadend = async () => {
        const base64 = reader.result as string;
        const base64Data = base64.split(',')[1];
        
        try {
          const response = await api.testStt(JSON.stringify({
            audioData: base64Data,
          }));
          
          const data = JSON.parse(response);
          set({ transcribedText: data.text, isRecording: false });
        } catch (error: any) {
          // Extract the actual error message from the API response
          const errorMessage = error?.details || error?.message || error;
          set({ error: `STT failed: ${errorMessage}` });
        } finally {
          set({ isLoading: false });
        }
      };
      
      set({ recorder: null });
    } catch (error) {
      set({ error: `Failed to stop recording: ${error}`, isRecording: false });
      set({ isLoading: false });
    }
  },
  
  // History
  loadHistory: async (limit: number = 50, offset: number = 0) => {
    try {
      set({ isLoading: true, error: null });
      
      const response = await api.getHistory(JSON.stringify({ limit, offset }));
      const history = JSON.parse(response);
      set({ history });
    } catch (error) {
      set({ error: `Failed to load history: ${error}` });
    } finally {
      set({ isLoading: false });
    }
  },
  
  // Utility
  clearError: () => {
    set({ error: null });
  },
  
  setError: (error: string) => {
    set({ error });
  },
}));

export default useTtsttStore;