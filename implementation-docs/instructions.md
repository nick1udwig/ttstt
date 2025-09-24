# 250923

## ttstt

ttstt is a wrapper around tts & stt providers.

### Prompt

ttstt receives requests to do tts on text or stt on speech audio.
It serves those requests by sending them on to providers.
To start with, OpenAI will be the provider of both tts and stt, but the application MUST be designed so other providers (elevenlabs, play.ai, groq, ...) can be added later.

The UI has pages to:
1. Test tts and stt.
   - tts: a text input box, a "Send" button, and a place where the completed audio transcript is playable once it is received (or an error message is displayed if request fails).
   - stt: a "Hold to Record" button. When held and then released, the audio is sent to the STT provider and the text is displayed (or an error).
2. Input provider API key and select default provider and default parameters for requests
3. Generate API keys for interacting with ttstt itself

The backend:
1. Manages sending tts & stt requests, handling responses
2. Manages provider API keys
3. Manages API keys for interacting with ttstt itself (`admin` which can do anything including altering app settings; `requestor` which can send tts or stt requests)
4. Stores audio:text pairs that result from requests, as well as metadata about the request

Read the crates in ~/git/hyperware-openai-ttstt to handle the communication with OpenAI for TTS and STT
