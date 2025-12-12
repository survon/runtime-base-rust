use super::state::{JukeboxState, JukeboxIntent, JukeboxEvent, JukeboxStateMachine};
use crate::util::io::bus::{BusMessage, MessageBus};
use crate::util::audio::SurvonAudioPlayer;
use tokio::sync::mpsc;
use std::time::Duration;

pub struct JukeboxActor {
    state: JukeboxState,
    message_bus: MessageBus,
    intent_rx: mpsc::UnboundedReceiver<JukeboxIntent>,
    intent_tx: mpsc::UnboundedSender<JukeboxIntent>, // For self-sent intents

    // Side-effect handler (isolated!)
    audio_player: Option<SurvonAudioPlayer>,
}

impl JukeboxActor {
    pub fn new(message_bus: MessageBus) -> (Self, mpsc::UnboundedSender<JukeboxIntent>) {
        let (intent_tx, intent_rx) = mpsc::unbounded_channel();

        let actor = Self {
            state: JukeboxState::default(),
            message_bus,
            intent_rx,
            intent_tx: intent_tx.clone(),
            audio_player: None,
        };

        (actor, intent_tx)
    }

    /// Run the actor event loop
    pub async fn run(mut self) {
        while let Some(intent) = self.intent_rx.recv().await {
            self.process_intent(intent).await;
        }
    }

    async fn process_intent(&mut self, intent: JukeboxIntent) {
        // Pure state transition
        let (new_state, events) = JukeboxStateMachine::transition(
            self.state.clone(),
            intent,
        );

        // Update internal state
        self.state = new_state;

        // Handle side effects (audio playback)
        self.handle_side_effects(&events).await;

        // Publish events to message bus
        for event in events {
            self.publish_event(event).await;
        }
    }

    async fn handle_side_effects(&mut self, events: &[JukeboxEvent]) {
        for event in events {
            match event {
                JukeboxEvent::TrackStarted { track, .. } => {
                    // Stop current player
                    if let Some(player) = self.audio_player.as_mut() {
                        let _ = player.stop();
                    }

                    // Start new player
                    let mut player = SurvonAudioPlayer::new_with_audio_jack(
                        &track.file_path,
                        self.state.volume,
                    );

                    if let Ok(_) = player.play() {
                        self.audio_player = Some(player);

                        // Spawn task to detect track end
                        self.spawn_playback_monitor();
                    }
                }

                JukeboxEvent::VolumeChanged(volume) => {
                    if let Some(player) = self.audio_player.as_mut() {
                        player.set_volume(*volume);
                    }
                }

                JukeboxEvent::StateChanged(state) if !state.is_playing => {
                    if let Some(player) = self.audio_player.as_mut() {
                        let _ = player.stop();
                    }
                }

                _ => {}
            }
        }
    }

    fn spawn_playback_monitor(&self) {
        let intent_tx = self.intent_tx.clone();

        if let Some(player) = &self.audio_player {
            // Clone the player's finished check capability
            // (You'll need to adjust based on your SurvonAudioPlayer API)
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // Check if playback finished
                    // (Pseudo-code - adjust to your player API)
                    // if player.is_finished() {
                    //     let _ = intent_tx.send(JukeboxIntent::TrackEnded);
                    //     break;
                    // }
                }
            });
        }
    }

    async fn publish_event(&self, event: JukeboxEvent) {
        let topic = match &event {
            JukeboxEvent::StateChanged(_) => "jukebox.state",
            JukeboxEvent::TrackStarted { .. } => "jukebox.track.started",
            JukeboxEvent::TrackEnded { .. } => "jukebox.track.ended",
            JukeboxEvent::PlaybackError { .. } => "jukebox.error",
            JukeboxEvent::VolumeChanged(_) => "jukebox.volume",
            JukeboxEvent::PlaylistLoaded { .. } => "jukebox.playlist.loaded",
        };

        let payload = serde_json::to_string(&event).unwrap();

        let _ = self.message_bus.publish(BusMessage::new(
            topic.to_string(),
            payload,
            "jukebox".to_string(),
        )).await;
    }
}
