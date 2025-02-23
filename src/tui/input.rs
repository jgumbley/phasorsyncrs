use crate::event_loop::{EngineMessage, TransportAction};
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

pub fn map_key_event(key: KeyEvent) -> Option<EngineMessage> {
    match key.code {
        KeyCode::Char(' ') => Some(EngineMessage::TransportCommand(TransportAction::Start)),
        KeyCode::Char('s') | KeyCode::Char('S') => {
            Some(EngineMessage::TransportCommand(TransportAction::Stop))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn test_space_maps_to_start() {
        let key_event = KeyEvent::from(KeyCode::Char(' '));
        let result = map_key_event(key_event);
        match result {
            Some(EngineMessage::TransportCommand(TransportAction::Start)) => {}
            _ => panic!("Expected Start command for Space key"),
        }
    }

    #[test]
    fn test_s_maps_to_stop() {
        let key_event = KeyEvent::from(KeyCode::Char('s'));
        let result = map_key_event(key_event);
        match result {
            Some(EngineMessage::TransportCommand(TransportAction::Stop)) => {}
            _ => panic!("Expected Stop command for 's' key"),
        }
    }

    #[test]
    fn test_other_key_returns_none() {
        let key_event = KeyEvent::from(KeyCode::Char('x'));
        assert!(map_key_event(key_event).is_none());
    }
}
