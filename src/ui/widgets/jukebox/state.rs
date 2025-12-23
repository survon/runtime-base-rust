use serde::{Deserialize, Serialize};
use super::database::{Album, Track};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JukeboxState {
    pub current_album: Option<Album>,
    pub current_track: Option<Track>,
    pub playlist: Vec<Track>,
    pub current_index: usize,
    pub volume: f32,
    pub is_playing: bool,
    pub shuffle: bool,
    pub repeat: bool,
    pub repeat_track: bool,
    pub playback_position_ms: u64,
}

impl Default for JukeboxState {
    fn default() -> Self {
        Self {
            current_album: None,
            current_track: None,
            playlist: Vec::new(),
            current_index: 0,
            volume: 1.0,
            is_playing: false,
            shuffle: false,
            repeat: false,
            repeat_track: false,
            playback_position_ms: 0,
        }
    }
}

// ----------------------------------------------------------------------------
// INTENT - Messages that express what user wants to do
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum JukeboxIntent {
    // Playback control
    Play,
    Pause,
    Stop,
    NextTrack,
    PreviousTrack,

    // Volume
    SetVolume(f32),
    VolumeUp,
    VolumeDown,

    // Playlist management
    LoadAlbum { album: Album, tracks: Vec<Track> },
    PlayTrack { index: usize },

    // Modes
    ToggleShuffle,
    ToggleRepeat,
    ToggleRepeatTrack,

    // Internal events (sent by actor itself)
    TrackEnded,
}

// ----------------------------------------------------------------------------
// EVENTS - Things that happened (past tense)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum JukeboxEvent {
    StateChanged(JukeboxState),
    TrackStarted { track: Track, album: Option<Album> },
    TrackEnded { track: Track },
    PlaybackError { track: Track, error: String },
    VolumeChanged(f32),
    PlaylistLoaded { album: Album, track_count: usize },
}

// ----------------------------------------------------------------------------
// STATE MACHINE - Pure function: (State, Intent) -> (State, Vec<Event>)
// ----------------------------------------------------------------------------

pub struct JukeboxStateMachine;

impl JukeboxStateMachine {
    /// Pure state transition - takes state + intent, returns new state + events
    pub fn transition(
        state: JukeboxState,
        intent: JukeboxIntent,
    ) -> (JukeboxState, Vec<JukeboxEvent>) {
        use JukeboxIntent::*;

        match intent {
            Play => Self::handle_play(state),
            Pause => Self::handle_pause(state),
            Stop => Self::handle_stop(state),
            NextTrack => Self::handle_next(state),
            PreviousTrack => Self::handle_previous(state),
            SetVolume(v) => Self::handle_set_volume(state, v),
            VolumeUp => Self::handle_volume_up(state),
            VolumeDown => Self::handle_volume_down(state),
            LoadAlbum { album, tracks } => Self::handle_load_album(state, album, tracks),
            PlayTrack { index } => Self::handle_play_track(state, index),
            ToggleShuffle => Self::handle_toggle_shuffle(state),
            ToggleRepeat => Self::handle_toggle_repeat(state),
            ToggleRepeatTrack => Self::handle_toggle_repeat_track(state),
            TrackEnded => Self::handle_track_ended(state),
        }
    }

    fn handle_play(mut state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        if state.current_track.is_none() && !state.playlist.is_empty() {
            // Start from beginning if nothing playing
            return Self::handle_play_track(state, 0);
        }

        state.is_playing = true;

        let mut events = vec![
            JukeboxEvent::StateChanged(state.clone()),
        ];

        if let Some(track) = &state.current_track {
            events.push(JukeboxEvent::TrackStarted {
                track: track.clone(),
                album: state.current_album.clone(),
            });
        }

        (state, events)
    }

    fn handle_pause(mut state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        state.is_playing = false;
        (state.clone(), vec![JukeboxEvent::StateChanged(state)])
    }

    fn handle_stop(mut state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        let old_track = state.current_track.clone();

        state.is_playing = false;
        state.current_track = None;
        state.playback_position_ms = 0;

        let mut events = vec![JukeboxEvent::StateChanged(state.clone())];

        if let Some(track) = old_track {
            events.push(JukeboxEvent::TrackEnded { track });
        }

        (state, events)
    }

    fn handle_next(state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        if state.playlist.is_empty() {
            return (state, vec![]);
        }

        let new_index = if state.current_index + 1 >= state.playlist.len() {
            if state.repeat {
                0
            } else {
                return Self::handle_stop(state);
            }
        } else {
            state.current_index + 1
        };

        Self::handle_play_track(state, new_index)
    }

    fn handle_previous(state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        if state.playlist.is_empty() {
            return (state, vec![]);
        }

        let new_index = if state.current_index == 0 {
            state.playlist.len() - 1
        } else {
            state.current_index - 1
        };

        Self::handle_play_track(state, new_index)
    }

    fn handle_set_volume(mut state: JukeboxState, volume: f32) -> (JukeboxState, Vec<JukeboxEvent>) {
        state.volume = volume.clamp(0.0, 1.0);

        (state.clone(), vec![
            JukeboxEvent::VolumeChanged(state.volume),
            JukeboxEvent::StateChanged(state),
        ])
    }

    fn handle_volume_up(state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        let volume = state.volume + 0.1;
        Self::handle_set_volume(state, volume)
    }

    fn handle_volume_down(state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        let volume = state.volume - 0.1;
        Self::handle_set_volume(state, volume)
    }

    fn handle_load_album(
        mut state: JukeboxState,
        album: Album,
        tracks: Vec<Track>,
    ) -> (JukeboxState, Vec<JukeboxEvent>) {
        let track_count = tracks.len();

        state.current_album = Some(album.clone());
        state.playlist = tracks;
        state.current_index = 0;
        state.is_playing = false;

        (state.clone(), vec![
            JukeboxEvent::PlaylistLoaded { album, track_count },
            JukeboxEvent::StateChanged(state),
        ])
    }

    fn handle_play_track(mut state: JukeboxState, index: usize) -> (JukeboxState, Vec<JukeboxEvent>) {
        if index >= state.playlist.len() {
            return (state, vec![]);
        }

        let track = state.playlist[index].clone();

        // Check if file exists
        if !std::path::Path::new(&track.file_path).exists() {
            return (state.clone(), vec![
                JukeboxEvent::PlaybackError {
                    track,
                    error: "Audio file not found".to_string(),
                },
                JukeboxEvent::StateChanged(state),
            ]);
        }

        state.current_index = index;
        state.current_track = Some(track.clone());
        state.is_playing = true;
        state.playback_position_ms = 0;

        (state.clone(), vec![
            JukeboxEvent::TrackStarted {
                track,
                album: state.current_album.clone(),
            },
            JukeboxEvent::StateChanged(state),
        ])
    }

    fn handle_track_ended(state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        if state.repeat_track {
            // Replay same track
            let index = state.current_index;
            return Self::handle_play_track(state, index);
        }

        // Auto-advance to next track
        Self::handle_next(state)
    }

    fn handle_toggle_shuffle(mut state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        state.shuffle = !state.shuffle;
        // TODO: Actually shuffle playlist
        (state.clone(), vec![JukeboxEvent::StateChanged(state)])
    }

    fn handle_toggle_repeat(mut state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        state.repeat = !state.repeat;
        (state.clone(), vec![JukeboxEvent::StateChanged(state)])
    }

    fn handle_toggle_repeat_track(mut state: JukeboxState) -> (JukeboxState, Vec<JukeboxEvent>) {
        state.repeat_track = !state.repeat_track;
        (state.clone(), vec![JukeboxEvent::StateChanged(state)])
    }
}
