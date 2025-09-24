import React, { useState, useEffect } from 'react';
import useTtsttStore from '../store/ttstt';
import { Provider, ProviderConfig } from '../types/ttstt';

// Available voices for OpenAI
const OPENAI_VOICES = [
  'alloy', 'ash', 'ballad', 'coral', 'echo', 
  'fable', 'nova', 'onyx', 'sage', 'shimmer', 'verse'
];

function SettingsPage() {
  const {
    providers,
    loadProviders,
    addProvider,
    removeProvider,
    setDefaultProvider,
    isLoading,
  } = useTtsttStore();

  // Load providers on mount
  useEffect(() => {
    loadProviders();
  }, []);

  // Form state
  const [showAddForm, setShowAddForm] = useState(false);
  const [newProvider, setNewProvider] = useState<Partial<ProviderConfig>>({
    provider: 'OpenAI',
    apiKey: '',
    isDefaultTts: false,
    isDefaultStt: false,
    defaultVoice: 'nova',
    defaultSpeed: 1.5,
  });

  const handleAddProvider = async (e: React.FormEvent) => {
    e.preventDefault();
    if (newProvider.apiKey && newProvider.provider) {
      await addProvider(newProvider as ProviderConfig);
      setNewProvider({
        provider: 'OpenAI',
        apiKey: '',
        isDefaultTts: false,
        isDefaultStt: false,
        defaultVoice: 'nova',
        defaultSpeed: 1.5,
      });
      setShowAddForm(false);
    }
  };

  const handleRemoveProvider = async (provider: Provider) => {
    if (window.confirm(`Remove ${provider} provider?`)) {
      await removeProvider(provider);
    }
  };

  const handleSetDefault = async (provider: Provider, type: 'tts' | 'stt') => {
    await setDefaultProvider(provider, type);
  };

  const handleUpdateProviderSettings = async (provider: Provider, voice: string, speed: number) => {
    // Find the existing provider config
    const existingProvider = providers.find(p => p.provider === (provider === 'OpenAI' ? 'OpenAi' : provider));
    if (existingProvider) {
      // Update the provider with new settings
      await removeProvider(provider);
      await addProvider({
        provider: provider,
        apiKey: '',  // We don't have the original API key here
        isDefaultTts: existingProvider.is_default_tts,
        isDefaultStt: existingProvider.is_default_stt,
        defaultVoice: voice,
        defaultSpeed: speed,
      });
    }
  };

  return (
    <div className="settings-page">
      <h2>Provider Settings</h2>

      <div className="card">
        <h3>Configured Providers</h3>
        {providers.length === 0 ? (
          <p className="text-muted">No providers configured. Click "Add Provider" below to get started.</p>
        ) : (
          <div className="flex flex-col gap-4">
            {providers.map((provider) => (
              <div key={provider.provider} className="card" style={{ background: 'var(--surface)' }}>
                <h4 style={{ marginBottom: '1rem' }}>{provider.provider}</h4>
                
                {/* Voice and Speed Settings */}
                <div className="form-row" style={{ marginBottom: '1rem' }}>
                  <div className="form-group" style={{ marginBottom: 0 }}>
                    <label>Default Voice</label>
                    <select
                      value={provider.default_voice || 'nova'}
                      onChange={(e) => handleUpdateProviderSettings(
                        provider.provider === 'OpenAi' ? 'OpenAI' as Provider : provider.provider as Provider, 
                        e.target.value, 
                        provider.default_speed || 1.5
                      )}
                      disabled={isLoading}
                    >
                      {OPENAI_VOICES.map(voice => (
                        <option key={voice} value={voice}>
                          {voice.charAt(0).toUpperCase() + voice.slice(1)}
                        </option>
                      ))}
                    </select>
                  </div>
                  
                  <div className="form-group" style={{ marginBottom: 0 }}>
                    <label>Default Speed</label>
                    <div className="flex items-center gap-2">
                      <input
                        type="range"
                        min="0.25"
                        max="4.0"
                        step="0.25"
                        value={provider.default_speed || 1.5}
                        onChange={(e) => handleUpdateProviderSettings(
                          provider.provider === 'OpenAi' ? 'OpenAI' as Provider : provider.provider as Provider,
                          provider.default_voice || 'nova',
                          parseFloat(e.target.value)
                        )}
                        disabled={isLoading}
                        style={{ flex: 1 }}
                      />
                      <span style={{ minWidth: '3rem', textAlign: 'right' }}>
                        {(provider.default_speed || 1.5).toFixed(2)}x
                      </span>
                    </div>
                  </div>
                </div>

                {/* Default Settings */}
                <div className="flex gap-4" style={{ marginBottom: '1rem' }}>
                  <label className="flex items-center gap-2" style={{ marginBottom: 0 }}>
                    <input
                      type="radio"
                      name="default-tts"
                      checked={provider.is_default_tts}
                      onChange={() => handleSetDefault(provider.provider === 'OpenAi' ? 'OpenAI' as Provider : provider.provider as Provider, 'tts')}
                      disabled={isLoading}
                      style={{ width: 'auto' }}
                    />
                    <span>Default for TTS</span>
                  </label>
                  
                  <label className="flex items-center gap-2" style={{ marginBottom: 0 }}>
                    <input
                      type="radio"
                      name="default-stt"
                      checked={provider.is_default_stt}
                      onChange={() => handleSetDefault(provider.provider === 'OpenAi' ? 'OpenAI' as Provider : provider.provider as Provider, 'stt')}
                      disabled={isLoading}
                      style={{ width: 'auto' }}
                    />
                    <span>Default for STT</span>
                  </label>
                </div>

                {/* Remove Button */}
                <button
                  onClick={() => handleRemoveProvider(provider.provider === 'OpenAi' ? 'OpenAI' as Provider : provider.provider as Provider)}
                  disabled={isLoading}
                  className="danger"
                  style={{ fontSize: '0.875rem' }}
                >
                  Remove Provider
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="mt-4">
        <button
          onClick={() => setShowAddForm(!showAddForm)}
          disabled={isLoading}
          className="primary"
        >
          {showAddForm ? 'Cancel' : '+ Add Provider'}
        </button>

        {showAddForm && (
          <form onSubmit={handleAddProvider} className="card mt-3">
            <h3>Add New Provider</h3>
            <div className="form-group">
              <label>Provider</label>
              <select
                value={newProvider.provider}
                onChange={(e) => setNewProvider({ ...newProvider, provider: e.target.value as Provider })}
                disabled={isLoading}
              >
                <option value="OpenAI">OpenAI</option>
              </select>
            </div>

            <div className="form-group">
              <label>API Key</label>
              <input
                type="password"
                value={newProvider.apiKey}
                onChange={(e) => setNewProvider({ ...newProvider, apiKey: e.target.value })}
                placeholder="Enter your provider API key"
                required
                disabled={isLoading}
              />
            </div>

            <div className="form-row">
              <div className="form-group">
                <label>Default Voice</label>
                <select
                  value={newProvider.defaultVoice}
                  onChange={(e) => setNewProvider({ ...newProvider, defaultVoice: e.target.value })}
                  disabled={isLoading}
                >
                  {OPENAI_VOICES.map(voice => (
                    <option key={voice} value={voice}>
                      {voice.charAt(0).toUpperCase() + voice.slice(1)}
                    </option>
                  ))}
                </select>
              </div>

              <div className="form-group">
                <label>Default Speed</label>
                <div className="flex items-center gap-2">
                  <input
                    type="range"
                    min="0.25"
                    max="4.0"
                    step="0.25"
                    value={newProvider.defaultSpeed}
                    onChange={(e) => setNewProvider({ ...newProvider, defaultSpeed: parseFloat(e.target.value) })}
                    disabled={isLoading}
                    style={{ flex: 1 }}
                  />
                  <span style={{ minWidth: '3rem', textAlign: 'right' }}>
                    {(newProvider.defaultSpeed || 1.5).toFixed(2)}x
                  </span>
                </div>
              </div>
            </div>

            <div className="flex gap-4 mb-4">
              <label className="flex items-center gap-2" style={{ marginBottom: 0, cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={newProvider.isDefaultTts}
                  onChange={(e) => setNewProvider({ ...newProvider, isDefaultTts: e.target.checked })}
                  disabled={isLoading}
                  style={{ width: 'auto' }}
                />
                <span>Set as default TTS provider</span>
              </label>
              
              <label className="flex items-center gap-2" style={{ marginBottom: 0, cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={newProvider.isDefaultStt}
                  onChange={(e) => setNewProvider({ ...newProvider, isDefaultStt: e.target.checked })}
                  disabled={isLoading}
                  style={{ width: 'auto' }}
                />
                <span>Set as default STT provider</span>
              </label>
            </div>

            <button type="submit" disabled={isLoading || !newProvider.apiKey} className="primary">
              Add Provider
            </button>
          </form>
        )}
      </div>
    </div>
  );
}

export default SettingsPage;