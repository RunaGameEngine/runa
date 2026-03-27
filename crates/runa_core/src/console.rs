//! In-game developer console for debugging and commands.
//!
//! The console can be toggled with the backquote (`) key and provides:
//! - Command history with up/down arrow navigation
//! - Basic built-in commands (help, clear, test)
//! - Semi-transparent overlay rendering

use std::collections::VecDeque;

use crate::components::Camera;
use runa_asset::Handle;
use runa_asset::TextureAsset;
use runa_render_api::RenderCommands;
use runa_render_api::RenderQueue;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::keyboard::Key;
use winit::keyboard::KeyCode;

/// Developer console for in-game commands and debugging
pub struct Console {
    /// Message history displayed in the console
    messages: VecDeque<String>,
    /// Maximum number of messages to keep in history
    max_messages: usize,
    /// Whether the console is currently visible
    is_visible: bool,
    /// Current text being typed in the input line
    pub input_buffer: String,
    /// History of previously executed commands
    history: VecDeque<String>,
    /// Current position in command history when navigating
    history_index: Option<usize>,
    /// Font texture for text rendering (reserved for future use)
    #[allow(dead_code)]
    font_texture: Option<Handle<TextureAsset>>,
}

impl Console {
    /// Creates a new console instance
    pub fn new() -> Self {
        let mut console = Self {
            messages: VecDeque::new(),
            max_messages: 50,
            is_visible: false,
            input_buffer: String::new(),
            history: VecDeque::new(),
            history_index: None,
            font_texture: None,
        };

        // Add startup message
        console.add_message("Runa Console initialized. Press ` to open.");

        console
    }

    /// Toggles console visibility
    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;

        // Clear input buffer when opening console
        if self.is_visible {
            self.input_buffer.clear();
            self.history_index = None;
        }
    }

    /// Returns whether the console is currently visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Adds a message to the console output
    pub fn add_message<S: Into<String>>(&mut self, message: S) {
        let msg = message.into();
        self.messages.push_back(msg);

        if self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    /// Renders the console overlay if visible
    pub fn render(&self, queue: &mut RenderQueue, camera: &Camera) {
        if !self.is_visible {
            return;
        }

        // Get screen dimensions
        let screen_width = camera.viewport_size.0 as f32;
        let screen_height = camera.viewport_size.1 as f32;

        // Console layout constants
        let console_padding_x = screen_width * 0.05;
        let console_padding_y = screen_height * 0.05;
        let console_width = screen_width - (console_padding_x * 2.0);
        let console_height = screen_height * 0.5;

        // Position console at top of screen (screen coordinates, Y down)
        let console_left = console_padding_x;
        let console_top = console_padding_y;

        // Draw semi-transparent background
        // Note: DebugRect uses center position
        queue.commands.push(RenderCommands::DebugRect {
            position: glam::Vec3::new(
                console_left + console_width / 2.0,
                console_top + console_height / 2.0,
                0.0,
            ),
            size: glam::Vec2::new(console_width, console_height),
            color: [0.0, 0.0, 0.0, 0.7], // Dark background with 70% opacity
        });

        // Calculate text layout
        let text_padding_x = console_left + 10.0;
        let line_height = 20.0;
        let max_visible_lines = ((console_height - 60.0) / line_height) as usize;

        // Render message history (newest at top, above input)
        let input_line_y = console_top + console_height - 50.0;
        let mut line_y = input_line_y - line_height;
        let mut lines_rendered = 0;

        // Iterate messages in reverse (newest first)
        for message in self.messages.iter().rev() {
            if lines_rendered >= max_visible_lines {
                break;
            }

            queue.commands.push(RenderCommands::Text {
                text: message.clone(),
                position: glam::Vec2::new(text_padding_x, line_y),
                color: [1.0, 1.0, 1.0, 1.0],
                size: 2.5,
            });

            line_y -= line_height;
            lines_rendered += 1;
        }

        // Render input line with cursor
        let cursor_char = if (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            % 1000)
            < 500
        {
            "█"
        } else {
            " "
        };

        queue.commands.push(RenderCommands::Text {
            text: format!("> {}{}", self.input_buffer, cursor_char),
            position: glam::Vec2::new(text_padding_x, input_line_y),
            color: [0.0, 1.0, 0.0, 1.0],
            size: 2.5,
        });
    }

    /// Handles keyboard input for the console.
    /// Call this every frame when the console is active.
    pub fn handle_keyboard(&mut self, event: &KeyEvent, state: ElementState) {
        // Handle backquote to toggle (only on press)
        if state == ElementState::Pressed
            && event.physical_key == winit::keyboard::PhysicalKey::Code(KeyCode::Backquote)
        {
            // Only toggle if not typing a character (to prevent accidental toggles)
            if !event.repeat {
                self.toggle();
                self.add_message("Console opened. Type 'help' for available commands.");
            }
            return;
        }

        // Only process other input when console is visible
        if !self.is_visible {
            return;
        }

        // Only handle key presses, not releases
        if state != ElementState::Pressed {
            return;
        }

        // Handle special keys first
        match event.physical_key {
            winit::keyboard::PhysicalKey::Code(KeyCode::Enter) => {
                // Execute command
                if !self.input_buffer.is_empty() {
                    // Add to history
                    self.history.push_back(self.input_buffer.clone());
                    if self.history.len() > 50 {
                        self.history.pop_front();
                    }
                    self.history_index = None;

                    // Execute and log
                    let command = self.input_buffer.clone();
                    self.add_message(format!("> {}", command));
                    self.execute_command(&command);
                    self.input_buffer.clear();
                }
                return;
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::Backspace) => {
                self.input_buffer.pop();
                return;
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::Escape) => {
                self.is_visible = false;
                self.input_buffer.clear();
                return;
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::ArrowUp) => {
                // Navigate up in history
                if !self.history.is_empty() {
                    let new_index = match self.history_index {
                        Some(i) => i.saturating_sub(1),
                        None => self.history.len() - 1,
                    };
                    self.history_index = Some(new_index);
                    if let Some(cmd) = self.history.get(new_index) {
                        self.input_buffer = cmd.clone();
                    }
                }
                return;
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::ArrowDown) => {
                // Navigate down in history
                if let Some(current_index) = self.history_index {
                    if current_index < self.history.len() - 1 {
                        let new_index = current_index + 1;
                        self.history_index = Some(new_index);
                        if let Some(cmd) = self.history.get(new_index) {
                            self.input_buffer = cmd.clone();
                        }
                    } else {
                        // Clear if at end of history
                        self.history_index = None;
                        self.input_buffer.clear();
                    }
                }
                return;
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::Tab) => {
                // Simple autocomplete - just add spaces for indentation
                self.input_buffer.push_str("    ");
                return;
            }
            _ => {}
        }

        // Handle character input (for typing text) - only printable characters
        if let Key::Character(c) = &event.logical_key {
            eprintln!("DEBUG: Key::Character = {:?}", c);
            for ch in c.chars() {
                eprintln!("DEBUG: char = {:?} (code={})", ch, ch as u32);
                // Only add printable ASCII characters
                if ch >= ' ' && ch <= '~' {
                    self.input_buffer.push(ch);
                    eprintln!("DEBUG: added char, buffer = {:?}", self.input_buffer);
                }
            }
        }
    }

    /// Executes a console command
    fn execute_command(&mut self, command: &str) {
        let trimmed = command.trim();

        // Ignore empty commands
        if trimmed.is_empty() {
            return;
        }

        // Parse command and arguments
        let mut parts = trimmed.split_whitespace();
        let cmd = parts.next().unwrap_or("").to_lowercase();

        match cmd.as_str() {
            "help" => {
                self.add_message("Available commands:");
                self.add_message("  help     - Show this help");
                self.add_message("  clear    - Clear console output");
                self.add_message("  test     - Test command");
                self.add_message("  fps      - Toggle FPS display");
                self.add_message("  quit     - Close the console");
            }
            "clear" => {
                self.messages.clear();
                self.add_message("Console cleared.");
            }
            "test" => {
                self.add_message("Test command executed successfully!");
            }
            "fps" => {
                self.add_message("FPS command - implement in your app");
            }
            "quit" | "exit" => {
                self.is_visible = false;
                self.add_message("Console closed.");
            }
            _ => {
                self.add_message(format!(
                    "Unknown command: '{}'. Type 'help' for commands.",
                    trimmed
                ));
            }
        }
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}
