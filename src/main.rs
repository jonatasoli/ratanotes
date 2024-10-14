use crossterm::event::{Event, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use std::{fs, io, path::PathBuf};
use tui_textarea::TextArea;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug, Default)]
enum InputMode {
    #[default]
    Editing,  // Normal mode for editing text
    FileName, // Mode to input the file name
    FilePath,
}

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    textarea: TextArea<'static>,
    folder_path: PathBuf,
    file_name_input: TextArea<'static>,
    input_mode: InputMode,
}

impl App {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Ratanotes: press ESC to exit"),
        );

        let mut file_name_input = TextArea::default();
        file_name_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter file name"),
        );

        // Set the initial folder to the current directory
        let folder_path = std::env::current_dir().unwrap();
        Self {
            exit: false,
            textarea,
            folder_path,
            file_name_input,
            input_mode: InputMode::Editing,
        }
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        match self.input_mode {
            InputMode::Editing => {
                // Define the layout with two sections: left and right, using `area`
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(70), // 70% of the width for the left section
                            Constraint::Percentage(30), // 30% of the width for the right section
                        ]
                        .as_ref(),
                    )
                    .split(area);

                // Render the TextArea (editable)
                self.textarea.render(chunks[0], frame.buffer_mut());

                // Render the folder navigation block on the right
                let folder_nav = Paragraph::new(format!(
                    "Folder Navigation:\n- {}",
                    self.folder_path.display()
                ))
                .style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Black)
                        .add_modifier(Modifier::ITALIC),
                )
                .block(Block::default().title("Folders").borders(Borders::ALL));

                folder_nav.render(chunks[1], frame.buffer_mut()); // Render the folder navigation
            }
            InputMode::FileName => {
                // Draw the file name input popup in the center of the screen
                let popup_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(40),
                            Constraint::Percentage(20), // Middle section for file name input
                            Constraint::Percentage(40),
                        ]
                        .as_ref(),
                    )
                    .split(area);

                let centered_chunk = popup_layout[1]; // Use the middle area for the popup

                self.file_name_input.render(centered_chunk, frame.buffer_mut());
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }

            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.input_mode {
            InputMode::Editing => match key_event.code {
                KeyCode::Esc => self.exit = true,
                KeyCode::Char('s') if key_event.modifiers == KeyModifiers::CONTROL => {
                    // Switch to FileName input mode when Ctrl+S is pressed
                    self.input_mode = InputMode::FileName;
                }
                _ => {
                    self.textarea.input(key_event);
                }
            },
            InputMode::FileName => match key_event.code {
                KeyCode::Enter => {
                    // Save the file when Enter is pressed in FileName input mode
                    let file_name = self.file_name_input.lines().join("");
                    self.save_file(file_name);
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Esc => {
                    // Cancel file name input and return to Editing mode
                    self.input_mode = InputMode::Editing;
                }
                _ => {
                    self.file_name_input.input(key_event);
                }
            },
        }
    }

    fn save_file(&self, file_name: String) {
        if file_name.is_empty() {
            eprintln!("File name cannot be empty!");
            return;
        }
        // Combine folder path and file name
        let file_path = self.folder_path.join(file_name.clone());

        // Save the content of the textarea to the file
        let content = self.textarea.lines().join("\n");

        match fs::write(&file_path, content) {
            Ok(_) => println!("File saved to {:?}", file_name),
            Err(e) => eprintln!("Failed to save file: {}", e),
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Define the layout with two sections: left and right, using `area`
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(70), // 70% of the width for the left section
                    Constraint::Percentage(30), // 30% of the width for the right section
                ]
                .as_ref(),
            )
            .split(area);

        self.textarea.render(chunks[0], buf); // Use `render` to draw to the buffer

        // Render the folder navigation block on the right
        let folder_nav = Paragraph::new(format!(
            "Folder Navigation:\n- {}",
            self.folder_path.display()
        ))
        .style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        )
        .block(Block::default().title("Folders").borders(Borders::ALL));

        folder_nav.render(chunks[1], buf); // Render the folder navigation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);

        Ok(())
    }
}
