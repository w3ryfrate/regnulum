use std::collections::HashSet;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct InputState {
    current_keys: HashSet<KeyCode>,
    previous_keys: HashSet<KeyCode>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            current_keys: HashSet::new(),
            previous_keys: HashSet::new(),
        }
    }

    pub fn on_key_event(&mut self, key_event: KeyEvent) {
        if let (ElementState::Pressed, PhysicalKey::Code(keycode)) =
            (key_event.state, key_event.physical_key)
        {
            self.current_keys.insert(keycode);
        }
    }

    pub fn is_key_pressed(&self, keycode: KeyCode) -> bool {
        self.current_keys.contains(&keycode) && !self.previous_keys.contains(&keycode)
    }

    pub fn is_key_down(&self, keycode: KeyCode) -> bool {
        self.current_keys.contains(&keycode)
    }

    pub fn refresh(&mut self) {
        self.previous_keys = self.current_keys.clone();
        self.current_keys.clear();
    }
}
