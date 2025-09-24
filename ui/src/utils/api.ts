// API utilities for TTSTT

import * as CallerUtils from '../../../target/ui/caller-utils';

// Re-export all the generated API functions with snake_case names
export const addProvider = CallerUtils.add_provider;
export const removeProvider = CallerUtils.remove_provider;
export const getProviders = CallerUtils.get_providers;
export const setDefaultProvider = CallerUtils.set_default_provider;
export const generateApiKey = CallerUtils.generate_api_key;
export const revokeApiKey = CallerUtils.revoke_api_key;
export const listApiKeys = CallerUtils.list_api_keys;
export const testTts = CallerUtils.test_tts;
export const testStt = CallerUtils.test_stt;
export const getHistory = CallerUtils.get_history;
export const getAudioTextPair = CallerUtils.get_audio_text_pair;
export const getAdminKey = CallerUtils.get_admin_key;

// Re-export the error class for convenience
export { ApiError } from '../../../target/ui/caller-utils';