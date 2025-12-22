// src/widgets/jukebox/widget.rs
use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, ListState, Paragraph, Widget},
    layout::{Constraint, Direction, Layout, Rect},
};
use tokio::sync::mpsc;
use color_eyre::Result;

use super::database::{Album, JukeboxDatabase, Track};
use super::state::{JukeboxState, JukeboxIntent, JukeboxEvent};
use crate::util::{
    database::Database,
    io::bus::{MessageBus,BusMessage}
};
use crate::ui::style::dim_unless_focused;

#[derive(Debug, PartialEq, Clone)]
pub enum JukeboxMode {
    Playing,
    AlbumList,
    TrackList,
}

#[derive(Debug)]
pub struct JukeboxWidget {
    // State (read-only copy)
    current_state: JukeboxState,

    // Intent sender (write-only)
    intent_tx: mpsc::UnboundedSender<JukeboxIntent>,

    // Event receiver (for state updates)
    event_rx: mpsc::UnboundedReceiver<BusMessage>,

    // UI state (not jukebox state)
    mode: JukeboxMode,
    albums: Vec<Album>,
    selected_album_index: usize,
    album_list_state: ListState,
    tracks: Vec<Track>,
    selected_track_index: usize,
    track_list_state: ListState,
    eq_bars: [usize; 16],
    eq_frame: usize,

    // Database for loading albums/tracks
    database: Database,
}

impl JukeboxWidget {
    pub async fn new(
        database: Database,
        message_bus: &MessageBus,
        intent_tx: mpsc::UnboundedSender<JukeboxIntent>,
    ) -> Result<Self> {
        // Subscribe to state changes
        let event_rx = message_bus.subscribe("jukebox.state".to_string()).await;

        let mut widget = Self {
            current_state: JukeboxState::default(),
            intent_tx,
            event_rx,
            mode: JukeboxMode::Playing,
            albums: Vec::new(),
            selected_album_index: 0,
            album_list_state: ListState::default(),
            tracks: Vec::new(),
            selected_track_index: 0,
            track_list_state: ListState::default(),
            eq_bars: [0; 16],
            eq_frame: 0,
            database,
        };

        widget.refresh_albums();

        Ok(widget)
    }

    fn refresh_albums(&mut self) {
        if let Ok(albums) = self.database.get_all_albums() {
            self.albums = albums;
        }
    }

    pub fn poll_state(&mut self) {
        while let Ok(msg) = self.event_rx.try_recv() {
            if let Ok(event) = serde_json::from_str::<JukeboxEvent>(&msg.payload) {
                if let JukeboxEvent::StateChanged(state) = event {
                    self.current_state = state;
                }
            }
        }
    }

    fn load_tracks_for_selected_album(&mut self) {
        if let Some(album) = self.albums.get(self.selected_album_index) {
            if let Ok(tracks) = self.database.get_tracks_for_album(album.id) {
                self.tracks = tracks;
                self.selected_track_index = 0;
            }
        }
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            JukeboxMode::Playing => JukeboxMode::AlbumList,
            JukeboxMode::AlbumList => JukeboxMode::Playing,
            JukeboxMode::TrackList => JukeboxMode::AlbumList,
        };
    }

    pub fn handle_enter(&mut self) {
        match self.mode {
            JukeboxMode::AlbumList => {
                self.load_tracks_for_selected_album();
                self.mode = JukeboxMode::TrackList;
            }
            JukeboxMode::TrackList => {
                if let Some(album) = self.albums.get(self.selected_album_index) {
                    // Send intent to load album and start playing
                    let _ = self.intent_tx.send(JukeboxIntent::LoadAlbum {
                        album: album.clone(),
                        tracks: self.tracks.clone(),
                    });
                    let _ = self.intent_tx.send(JukeboxIntent::PlayTrack {
                        index: self.selected_track_index,
                    });
                    self.mode = JukeboxMode::Playing;
                }
            }
            _ => {}
        }
    }

    pub fn next_item(&mut self) {
        match self.mode {
            JukeboxMode::AlbumList => {
                if !self.albums.is_empty() {
                    self.selected_album_index = (self.selected_album_index + 1) % self.albums.len();
                }
            }
            JukeboxMode::TrackList => {
                if !self.tracks.is_empty() {
                    self.selected_track_index = (self.selected_track_index + 1) % self.tracks.len();
                }
            }
            _ => {}
        }
    }

    pub fn prev_item(&mut self) {
        match self.mode {
            JukeboxMode::AlbumList => {
                if !self.albums.is_empty() {
                    self.selected_album_index = if self.selected_album_index == 0 {
                        self.albums.len() - 1
                    } else {
                        self.selected_album_index - 1
                    };
                }
            }
            JukeboxMode::TrackList => {
                if !self.tracks.is_empty() {
                    self.selected_track_index = if self.selected_track_index == 0 {
                        self.tracks.len() - 1
                    } else {
                        self.selected_track_index - 1
                    };
                }
            }
            _ => {}
        }
    }

    pub fn is_playing(&self) -> bool {
        self.current_state.is_playing
    }

    pub fn play_pause(&self) {
        if self.current_state.is_playing {
            let _ = self.intent_tx.send(JukeboxIntent::Pause);
        } else {
            let _ = self.intent_tx.send(JukeboxIntent::Play);
        }
    }

    pub fn next_track(&self) {
        let _ = self.intent_tx.send(JukeboxIntent::NextTrack);
    }

    pub fn previous_track(&self) {
        let _ = self.intent_tx.send(JukeboxIntent::PreviousTrack);
    }

    pub fn volume_up(&self) {
        let _ = self.intent_tx.send(JukeboxIntent::VolumeUp);
    }

    pub fn volume_down(&self) {
        let _ = self.intent_tx.send(JukeboxIntent::VolumeDown);
    }

    fn update_eq_animation(&mut self) {
        use rand::Rng;

        self.eq_frame = (self.eq_frame + 1) % 60;

        if self.current_state.is_playing {
            let mut rng = rand::thread_rng();
            for bar in &mut self.eq_bars {
                let change: i32 = rng.gen_range(-1..=1);
                let new_value = (*bar as i32) + change;
                *bar = new_value.clamp(0, 7) as usize;
            }
        } else {
            for bar in &mut self.eq_bars {
                if *bar > 0 {
                    *bar -= 1;
                }
            }
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, is_focused: Option<bool>) {
        // Poll for state updates
        self.poll_state();

        // Update animations
        self.update_eq_animation();

        // Render based on mode
        match self.mode {
            JukeboxMode::Playing => self.render_now_playing(area, buf, is_focused),
            JukeboxMode::AlbumList => self.render_album_list(area, buf, is_focused),
            JukeboxMode::TrackList => self.render_track_list(area, buf, is_focused),
        }
    }

    fn render_now_playing(&self, area: Rect, buf: &mut Buffer, is_focused: Option<bool>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Track info
                Constraint::Length(3),  // Controls
            ])
            .split(area);

        // Use current_state instead of player
        let track = &self.current_state.current_track;
        let album = &self.current_state.current_album;

        let track_text = if let Some(track) = track {
            // Check if the file actually exists
            if !std::path::Path::new(&track.file_path).exists() {
                vec![
                    Line::from("♪ Audio Missing").fg(Color::Red),
                    Line::from(format!("  Track: {}", track.title)).fg(Color::Gray),
                    Line::from("  Download audio files to play").fg(Color::Yellow),
                ]
            } else {
                // Build lines conditionally based on what we have
                let mut lines = vec![
                    Line::from(vec![
                        Span::raw("♪ ").fg(Color::Cyan),
                        Span::raw(track.title.clone()).fg(Color::White).bold(),
                    ]),
                ];

                // Add album info if available
                if let Some(album) = album {
                    lines.push(Line::from(vec![
                        Span::raw("  Artist: ").fg(Color::Gray),
                        Span::raw(album.artist.clone()).fg(Color::Yellow),
                    ]));
                    lines.push(Line::from(vec![
                        Span::raw("  Album: ").fg(Color::Gray),
                        Span::raw(album.title.clone()).fg(Color::Green),
                    ]));
                }

                lines.push(self.get_eq_visualizer());
                lines
            }
        } else {
            vec![
                Line::from("♪ No track playing").fg(Color::Gray),
                Line::from("  Press 'm' to browse library").fg(Color::Gray),
            ]
        };

        let border_style = dim_unless_focused(is_focused, Style::default().fg(Color::Cyan));

        Paragraph::new(track_text)
            .block(
                Block::bordered()
                    .title(" 🎵 Jukebox ")
                    .border_type(BorderType::Rounded)
                    .style(border_style)
            )
            .render(chunks[0], buf);

        // Controls - use current_state.volume
        let controls_text = if self.is_playing() {
            "⏸ Space: Pause | ⏭ →: Next | ⏮ ←: Prev | 🔊 +/-: Volume | 'm': Library"
        } else {
            "▶ Space: Play | ⏭ →: Next | ⏮ ←: Prev | 🔊 +/-: Volume | 'm': Library"
        };

        let volume = self.current_state.volume;
        let volume_display = format!("Vol: {}%", (volume * 100.0) as i32);

        Paragraph::new(vec![
            Line::from(controls_text).fg(Color::Gray),
            Line::from(volume_display).fg(Color::Yellow),
        ])
            .block(Block::bordered().border_type(BorderType::Rounded))
            .render(chunks[1], buf);
    }

    fn get_eq_visualizer(&self) -> Line {
        const BLOCKS: [&str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

        // Build EQ display as a single line of block characters
        let eq_display: String = self.eq_bars
            .iter()
            .map(|&level| BLOCKS[level])
            .collect();

        // Determine color based on average level
        let avg_level = self.eq_bars.iter().sum::<usize>() as f32 / self.eq_bars.len() as f32;
        let color = if avg_level > 5.0 {
            Color::Red
        } else if avg_level > 3.0 {
            Color::Yellow
        } else if avg_level > 0.0 {
            Color::Green
        } else {
            Color::DarkGray
        };

        let icon = if self.is_playing() { "🔊" } else { "🔇" };

        Line::from(vec![
            Span::raw(icon).fg(color),
            Span::raw(eq_display).fg(color).bold(),
        ])
    }

    fn render_album_list(&mut self, area: Rect, buf: &mut Buffer, is_focused: Option<bool>) {
        let border_style = dim_unless_focused(is_focused, Style::default().fg(Color::Cyan));

        let items: Vec<ListItem> = self.albums
            .iter()
            .enumerate()
            .map(|(i, album)| {
                let is_selected = i == self.selected_album_index;
                let style = if is_selected {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let content = format!(
                    "{} {} - {} ({})",
                    if is_selected { "▶" } else { " " },
                    album.artist,
                    album.title,
                    album.year.map(|y| y.to_string()).unwrap_or_else(|| "Unknown".to_string())
                );

                ListItem::new(content).style(style)
            })
            .collect();

        self.album_list_state.select(Some(self.selected_album_index));

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(" 🎵 Album Library (Enter: View Tracks | Esc: Back) ")
                    .border_type(BorderType::Rounded)
                    .style(border_style)
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        ratatui::widgets::StatefulWidget::render(list, area, buf, &mut self.album_list_state);
    }

    fn render_track_list(&mut self, area: Rect, buf: &mut Buffer, is_focused: Option<bool>) {
        let border_style = dim_unless_focused(is_focused, Style::default().fg(Color::Cyan));

        let album_title = self.albums
            .get(self.selected_album_index)
            .map(|a| format!("{} - {}", a.artist, a.title))
            .unwrap_or_else(|| "Unknown Album".to_string());

        let items: Vec<ListItem> = self.tracks
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let is_selected = i == self.selected_track_index;
                let file_exists = std::path::Path::new(&track.file_path).exists();

                let style = if !file_exists {
                    Style::default().fg(Color::Red)
                } else if is_selected {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let duration = track.duration_seconds
                    .map(|s| format!("{}:{:02}", s / 60, s % 60))
                    .unwrap_or_else(|| "?:??".to_string());

                let status_icon = if !file_exists {
                    "❌"
                } else if is_selected {
                    "▶"
                } else {
                    " "
                };

                let content = format!(
                    "{} {}. {} [{}]",
                    status_icon,
                    track.track_number,
                    track.title,
                    duration
                );

                ListItem::new(content).style(style)
            })
            .collect();

        self.track_list_state.select(Some(self.selected_track_index));

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(format!("🎵 {} (Enter: Play | Esc: Back)", album_title))
                    .border_type(BorderType::Rounded)
                    .style(border_style)
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        ratatui::widgets::StatefulWidget::render(list, area, buf, &mut self.track_list_state);
    }
}
