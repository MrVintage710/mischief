use bevy::{ecs::system::SystemParam, prelude::*};
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::TerminalMessage;

//==============================================================================================
//        TerminalInputPlugin
//==============================================================================================

#[derive(SystemParam)]
pub struct TerminalInput<'w, 's> {
    terminal_events : MessageReader<'w, 's, TerminalMessage>
}

impl <'w, 's> TerminalInput<'w, 's> {
    pub fn pressed(&mut self, keycode : KeyCode) -> bool {
        for message in self.terminal_events.read() {
            if let Event::Key(key_event) = **message {
                if key_event.code == keycode && key_event.kind == KeyEventKind::Press {
                    return true;
                }
            }
        }
        
        false
    }
    
    pub fn released(&mut self, keycode : KeyCode) -> bool {
        for message in self.terminal_events.read() {
            if let Event::Key(key_event) = **message {
                if key_event.code == keycode && key_event.kind == KeyEventKind::Release {
                    return true;
                }
            }
        }
        
        false
    }
}