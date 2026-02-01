use std::collections::VecDeque;

use crate::components::camera2d::Camera2D;
use runa_asset::handle::Handle;
use runa_asset::texture::TextureAsset;
use runa_render_api::queue::RenderQueue;

pub struct Console {
    messages: VecDeque<String>,
    max_messages: usize,
    is_visible: bool,
    pub input_buffer: String,
    history: VecDeque<String>,
    history_index: Option<usize>,
    pub(crate) font_texture: Option<Handle<TextureAsset>>,
}

impl Console {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 50,
            is_visible: false,
            input_buffer: String::new(),
            history: VecDeque::new(),
            history_index: None,
            font_texture: None,
        }
    }

    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn add_message<S: Into<String>>(&mut self, message: S) {
        let msg = message.into();
        self.messages.push_back(msg);

        if self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    pub fn render(&self, queue: &mut RenderQueue, camera: &Camera2D) {
        if !self.is_visible {
            return;
        }

        // Calculate screen dimensions for console positioning
        let screen_width = camera.viewport_size.0 as f32;
        let screen_height = camera.viewport_size.1 as f32;

        // Calculate world coordinates for screen corners
        let top_left = camera.screen_to_world((0.0, 0.0));
        let bottom_right = camera.screen_to_world((screen_width, screen_height));

        // Calculate console dimensions (top half of screen)
        let console_height = (bottom_right.y - top_left.y) * 0.6; // 60% of screen height
        let console_width = bottom_right.x - top_left.x; // Full width

        // Position console at the top
        let console_position = glam::Vec2::new(
            top_left.x + console_width / 2.0,
            top_left.y + console_height / 2.0,
        );

        // Draw console background (semi-transparent overlay)
        queue
            .commands
            .push(runa_render_api::command::RenderCommands::DebugRect {
                position: console_position,
                size: glam::Vec2::new(console_width * 0.95, console_height * 0.95), // Slightly smaller for padding
                color: [0.0, 0.0, 0.0, 0.7], // Black with 70% opacity
            });

        // Render console text - for now using placeholder text rendering
        // In a real implementation, we'd render actual text using a font system
        let line_height = console_height * 0.05; // Approximate line height
        let start_y = console_position.y - console_height * 0.45; // Starting Y position (near top of console)

        // Render recent messages (most recent at the top)
        for (i, message) in self.messages.iter().rev().take(10).enumerate() {
            let text_y = start_y - (i as f32 * line_height * 1.2);
            queue
                .commands
                .push(runa_render_api::command::RenderCommands::Text {
                    text: message.clone(),
                    position: glam::Vec2::new(console_position.x - console_width * 0.45, text_y),
                    color: [1.0, 1.0, 1.0, 1.0], // White text
                    size: 0.5,                   // Text size
                });
        }

        // Render input buffer with prompt
        let input_y = start_y - (10.0 * line_height * 1.2); // Below the messages
        queue
            .commands
            .push(runa_render_api::command::RenderCommands::Text {
                text: format!("> {}", self.input_buffer),
                position: glam::Vec2::new(console_position.x - console_width * 0.45, input_y),
                color: [0.0, 1.0, 0.0, 1.0], // Green text for input
                size: 0.5,
            });
    }

    pub fn handle_input(&mut self, input: &crate::input::InputState) {
        // Handle toggling with the backtick key
        if input.is_key_just_pressed(winit::keyboard::KeyCode::Backquote) {
            self.toggle();
        }

        // Only process other input if console is visible
        if !self.is_visible {
            return;
        }

        // Handle character input for the command line
        // This is a simplified approach - in a real implementation, we'd need to handle
        // text input events from the windowing system
        use winit::keyboard::{Key, NamedKey};

        // For now, we'll simulate character input by checking specific keys
        // In a real implementation, we'd hook into the window event system directly

        // Handle Enter key to execute command
        if input.is_key_just_pressed(winit::keyboard::KeyCode::Enter) {
            if !self.input_buffer.is_empty() {
                // Add the command to history
                self.history.push_back(self.input_buffer.clone());
                if self.history.len() > 50 {
                    self.history.pop_front();
                }

                // Process the command
                self.execute_command();

                // Clear the input buffer
                self.input_buffer.clear();
            }
        }

        // Handle Backspace
        if input.is_key_just_pressed(winit::keyboard::KeyCode::Backspace) {
            self.input_buffer.pop();
        }

        // Handle Escape to close console
        if input.is_key_just_pressed(winit::keyboard::KeyCode::Escape) {
            self.is_visible = false;
        }

        // Handle Up/Down arrows for command history (simplified)
        if input.is_key_just_pressed(winit::keyboard::KeyCode::ArrowUp) {
            if let Some(prev_cmd) = self
                .history
                .iter()
                .rev()
                .nth(self.history.len().saturating_sub(1))
            {
                self.input_buffer = prev_cmd.clone();
            }
        }

        // Add some simulated character input (for demo purposes)
        // In a real implementation, we'd need to handle actual text input
        if input.is_key_just_pressed(winit::keyboard::KeyCode::KeyA) && !self.is_visible {
            self.add_message("Simulated input: A key pressed");
        }
    }

    fn execute_command(&mut self) {
        // Log the command execution
        self.add_message(format!("> {}", self.input_buffer));

        // Basic command processing
        match self.input_buffer.trim().to_lowercase().as_str() {
            "help" => {
                self.add_message("Available commands: help, clear, test, quit");
            }
            "clear" => {
                self.messages.clear();
            }
            "test" => {
                self.add_message("Test command executed!");
            }
            "quit" | "exit" => {
                self.add_message("Use ESC to close the console");
            }
            "" => {} // Ignore empty commands
            _ => {
                self.add_message(format!("Unknown command: {}", self.input_buffer));
            }
        }
    }
}
