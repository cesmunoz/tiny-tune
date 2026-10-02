use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use librespot::{
    core::session::Session,
    playback::{
        audio_backend,
        config::{AudioFormat, PlayerConfig},
        mixer::{self, Mixer, MixerConfig},
        player::{Player, PlayerEventChannel},
    },
};

pub struct AudioRuntime {
    pub player: Arc<Player>,
    pub mixer: Arc<dyn Mixer>,
    pub events: PlayerEventChannel,
}

pub fn initialize(session: Session) -> Result<AudioRuntime> {
    let sink_builder =
        audio_backend::find(None).ok_or_else(|| anyhow!("no supported audio backend found"))?;
    let mixer_builder =
        mixer::find(None).ok_or_else(|| anyhow!("no supported audio mixer found"))?;
    let mixer = mixer_builder(MixerConfig::default())?;
    let player_config = PlayerConfig {
        position_update_interval: Some(Duration::from_secs(1)),
        ..PlayerConfig::default()
    };
    let player = Player::new(player_config, session, mixer.get_soft_volume(), move || {
        sink_builder(None, AudioFormat::default())
    });
    let events = player.get_player_event_channel();

    Ok(AudioRuntime {
        player,
        mixer,
        events,
    })
}
