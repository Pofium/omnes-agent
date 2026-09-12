//! Voice duplex event dispatch for WebSocket sessions.
#![cfg(feature = "gateway-voice-duplex")]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VoiceEvent {
    /// Client signals that speech has started.
    #[serde(rename = "speech_start")]
    SpeechStart,

    /// Client signals that speech has ended, with optional transcript.
    #[serde(rename = "speech_end")]
    SpeechEnd {
        #[serde(default)]
        transcript: Option<String>,
    },

    /// Client requests cancellation of in-progress TTS.
    #[serde(rename = "barge_in")]
    BargeIn,

    /// Server cancels in-progress TTS.
    #[serde(rename = "tts_cancel")]
    TtsCancel,

    /// Server sends a chunk of base64-encoded audio.
    #[serde(rename = "tts_chunk")]
    TtsChunk {
        audio_b64: String,
        #[serde(default)]
        format: Option<String>,
    },
}

/// Attempt to parse a text frame as a voice event.
/// Returns `Some(VoiceEvent)` if the JSON parses as a known voice event type,
/// or `None` if it's not a voice event (let it fall through to normal handling).
pub fn try_parse_voice_event(text: &str) -> Option<VoiceEvent> {
    serde_json::from_str::<VoiceEvent>(text).ok()
}

pub fn handle_voice_event(event: VoiceEvent) -> Option<serde_json::Value> {
    match event {
        VoiceEvent::SpeechStart => {
            ::omnesagent_log::record!(
                DEBUG,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note),
                "voice duplex: speech_start received"
            );
            None
        }
        VoiceEvent::SpeechEnd { transcript } => {
            ::omnesagent_log::record!(
                DEBUG,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                    .with_attrs(::serde_json::json!({"transcript": transcript})),
                "voice duplex: speech_end received"
            );
            None
        }
        VoiceEvent::BargeIn => {
            ::omnesagent_log::record!(
                DEBUG,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note),
                "voice duplex: barge_in received, emitting tts_cancel"
            );
            Some(serde_json::json!({
                "type": "tts_cancel",
                "reason": "barge_in"
            }))
        }
        VoiceEvent::TtsCancel | VoiceEvent::TtsChunk { .. } => {
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                    .with_outcome(::omnesagent_log::EventOutcome::Unknown),
                "voice duplex: received server-side event from client"
            );
            Some(serde_json::json!({
                "type": "error",
                "code": "invalid_event_direction",
                "message": "this event type is server-to-client only"
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Roundtrip serialization tests (moved from omnesagent-api) ──

    #[test]
    fn voice_event_speech_start_roundtrip() {
        let event = VoiceEvent::SpeechStart;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "{\"type\":\"speech_start\"}");
    }

    #[test]
    fn voice_event_speech_end_roundtrip() {
        let json = r#"{"type":"speech_end","transcript":"hello"}"#;
        let event: VoiceEvent = serde_json::from_str(json).unwrap();
        match event {
            VoiceEvent::SpeechEnd { transcript } => {
                assert_eq!(transcript.as_deref(), Some("hello"));
            }
            _ => panic!("expected SpeechEnd"),
        }
    }

    #[test]
    fn voice_event_barge_in_roundtrip() {
        let event = VoiceEvent::BargeIn;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "{\"type\":\"barge_in\"}");
    }

    #[test]
    fn voice_event_tts_chunk_roundtrip() {
        let event = VoiceEvent::TtsChunk {
            audio_b64: "AAAA".to_string(),
            format: Some("mp3".to_string()),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VoiceEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, VoiceEvent::TtsChunk { .. }));
    }

    // ── Parse tests ──

    #[test]
    fn parse_speech_start() {
        let event = try_parse_voice_event(r#"{"type":"speech_start"}"#);
        assert!(event.is_some());
    }

    #[test]
    fn parse_speech_end() {
        let event = try_parse_voice_event(r#"{"type":"speech_end","transcript":"hello"}"#);
        assert!(event.is_some());
    }

    #[test]
    fn parse_barge_in() {
        let event = try_parse_voice_event(r#"{"type":"barge_in"}"#);
        assert!(event.is_some());
    }

    #[test]
    fn non_voice_event_returns_none() {
        let event = try_parse_voice_event(r#"{"type":"message","content":"hello"}"#);
        assert!(event.is_none());
    }

    #[test]
    fn invalid_json_returns_none() {
        let event = try_parse_voice_event("not json");
        assert!(event.is_none());
    }

    #[test]
    fn tts_chunk_parse() {
        let event =
            try_parse_voice_event(r#"{"type":"tts_chunk","audio_b64":"AAAA","format":"mp3"}"#);
        assert!(event.is_some());
    }

    // ── Error frame tests ──

    #[test]
    fn server_events_return_error_frame() {
        let cancel_result = handle_voice_event(VoiceEvent::TtsCancel);
        assert!(cancel_result.is_some());
        let err = cancel_result.unwrap();
        assert_eq!(err["type"], "error");
        assert_eq!(err["code"], "invalid_event_direction");

        let chunk_result = handle_voice_event(VoiceEvent::TtsChunk {
            audio_b64: "AAAA".into(),
            format: None,
        });
        assert!(chunk_result.is_some());
        assert_eq!(chunk_result.unwrap()["code"], "invalid_event_direction");
    }

    #[test]
    fn client_events_speech_start_and_end_return_none() {
        assert!(handle_voice_event(VoiceEvent::SpeechStart).is_none());
        assert!(handle_voice_event(VoiceEvent::SpeechEnd { transcript: None }).is_none());
    }

    #[test]
    fn barge_in_returns_tts_cancel() {
        let res = handle_voice_event(VoiceEvent::BargeIn);
        assert!(res.is_some());
        let payload = res.unwrap();
        assert_eq!(payload["type"], "tts_cancel");
        assert_eq!(payload["reason"], "barge_in");
    }
}
