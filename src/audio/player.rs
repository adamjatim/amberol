// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

use async_channel::{Receiver, Sender};
use glib::clone;
use gtk::glib;
use log::{debug, error};

use crate::{
    application::ApplicationAction,
    audio::{
        Controller, CoverCache, GstBackend, InhibitController, MprisController, PlayerState, Queue,
        Song, WaveformGenerator,
    },
};

#[derive(Clone, Debug)]
pub enum PlaybackAction {
    Play,
    Pause,
    Stop,
    SkipPrevious,
    SkipNext,

    UpdatePosition(u64, bool),
    VolumeChanged(f64),
    Repeat(RepeatMode),
    Seek(i64),
    PlayNext,

    Raise,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone, Copy, Debug, glib::Enum, PartialEq, Default)]
#[enum_type(name = "AmberolRepeatMode")]
pub enum RepeatMode {
    #[default]
    Consecutive,
    RepeatAll,
    RepeatOne,
}

impl Display for RepeatMode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            RepeatMode::Consecutive => write!(f, "consecutive"),
            RepeatMode::RepeatAll => write!(f, "repeat-all"),
            RepeatMode::RepeatOne => write!(f, "repeat-one"),
        }
    }
}

#[derive(Clone, Copy, Debug, glib::Enum, PartialEq)]
#[enum_type(name = "AmberolReplayGainMode")]
pub enum ReplayGainMode {
    #[enum_value(name = "album")]
    Album,
    #[enum_value(name = "track")]
    Track,
    #[enum_value(name = "off")]
    Off,
}

impl Default for ReplayGainMode {
    fn default() -> Self {
        Self::Off
    }
}

impl From<i32> for ReplayGainMode {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Album,
            1 => Self::Track,
            2 => Self::Off,
            _ => panic!("invalid ReplayGainMode enum key"),
        }
    }
}

impl From<ReplayGainMode> for i32 {
    fn from(value: ReplayGainMode) -> Self {
        match value {
            ReplayGainMode::Album => 0,
            ReplayGainMode::Track => 1,
            ReplayGainMode::Off => 2,
        }
    }
}

#[derive(Debug)]
pub enum SeekDirection {
    Forward,
    Backwards,
}

pub struct AudioPlayer {
    app_sender: Sender<ApplicationAction>,
    receiver: RefCell<Option<Receiver<PlaybackAction>>>,
    backend: GstBackend,
    controllers: Vec<Box<dyn Controller>>,
    queue: Queue,
    state: PlayerState,
    waveform_generator: WaveformGenerator,
}

impl fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioPlayer").finish()
    }
}

impl AudioPlayer {
    pub fn new(app_sender: Sender<ApplicationAction>) -> Rc<Self> {
        let (sender, r) = async_channel::unbounded();
        let receiver = RefCell::new(Some(r));

        let mut controllers: Vec<Box<dyn Controller>> = Vec::new();

        let mpris_controller = MprisController::new(sender.clone());
        controllers.push(Box::new(mpris_controller));

        let inhibit_controller = InhibitController::new();
        controllers.push(Box::new(inhibit_controller));

        let waveform_generator = WaveformGenerator::new();
        controllers.push(Box::new(waveform_generator.clone()));

        let backend = GstBackend::new(sender);

        let queue = Queue::default();
        let state = PlayerState::default();

        let res = Rc::new(Self {
            app_sender,
            receiver,
            backend,
            controllers,
            queue,
            state,
            waveform_generator,
        });

        res.clone().setup_channel();

        res
    }

    fn setup_channel(self: Rc<Self>) {
        let receiver = self.receiver.borrow_mut().take().unwrap();

        glib::MainContext::default().spawn_local(clone!(
            #[strong(rename_to = this)]
            self,
            async move {
                use futures::prelude::*;

                let mut receiver = std::pin::pin!(receiver);
                while let Some(action) = receiver.next().await {
                    this.process_action(action);
                }
            }
        ));
    }

    fn process_action(&self, action: PlaybackAction) -> glib::ControlFlow {
        match action {
            PlaybackAction::Play => self.set_playback_state(PlaybackState::Playing),
            PlaybackAction::Pause => self.set_playback_state(PlaybackState::Paused),
            PlaybackAction::Stop => self.set_playback_state(PlaybackState::Stopped),
            PlaybackAction::SkipPrevious => self.skip_previous(),
            PlaybackAction::SkipNext => self.skip_next(),
            PlaybackAction::UpdatePosition(pos, notify) => self.update_position(pos, notify),
            PlaybackAction::VolumeChanged(vol) => self.update_volume(vol),
            PlaybackAction::PlayNext => self.play_next(),
            PlaybackAction::Raise => self.present(),
            PlaybackAction::Repeat(mode) => self.update_repeat_mode(mode),
            PlaybackAction::Seek(offset) => self.seek_offset(offset),
            // _ => debug!("Received action {:?}", action),
        }

        glib::ControlFlow::Continue
    }

    fn set_playback_state(&self, state: PlaybackState) {
        if let Some(current_song) = self.state.current_song() {
            debug!("Current song: {}", current_song.uri());

            self.state.set_playback_state(&state);

            for c in &self.controllers {
                c.set_playback_state(&state);
            }

            match state {
                PlaybackState::Playing => self.backend.play(),
                PlaybackState::Paused => self.backend.pause(),
                PlaybackState::Stopped => self.backend.stop(),
            }
        } else {
            debug!("Getting the next song");
            if let Some(next_song) = self.queue.next_song() {
                debug!("Next song: {}", next_song.uri());

                for c in &self.controllers {
                    c.set_song(&next_song);
                }

                next_song.set_playing(true);

                self.backend.set_song_uri(Some(&next_song.uri()));
                self.state.set_current_song(Some(next_song));
                self.state.set_playback_state(&state);

                for c in &self.controllers {
                    c.set_playback_state(&state);
                }

                match state {
                    PlaybackState::Playing => self.backend.play(),
                    PlaybackState::Paused => self.backend.pause(),
                    PlaybackState::Stopped => self.backend.stop(),
                }
            } else {
                debug!("No songs left");
                self.backend.set_song_uri(None);
                self.state.set_current_song(None);
                self.state.set_playback_state(&PlaybackState::Stopped);

                for c in &self.controllers {
                    c.set_playback_state(&PlaybackState::Stopped);
                }
            }
        }
    }

    fn play_next(&self) {
        self.skip_next();
    }

    pub fn toggle_play(&self) {
        if self.queue.is_empty() {
            return;
        }

        if self.state.playing() {
            self.set_playback_state(PlaybackState::Paused);
        } else {
            self.set_playback_state(PlaybackState::Playing);
        }
    }

    pub fn play(&self) {
        if !self.state.playing() {
            self.set_playback_state(PlaybackState::Playing);
        }
    }

    pub fn pause(&self) {
        if self.state.playing() {
            self.set_playback_state(PlaybackState::Paused);
        }
    }

    pub fn stop(&self) {
        self.set_playback_state(PlaybackState::Stopped);
    }

    pub fn skip_previous(&self) {
        if self.queue.is_empty() {
            return;
        }

        if let Some(current_song) = self.state.current_song() {
            // We only skip to the previous song if we are
            // within a seek backward step, otherwise we just
            // restart the song
            if self.state.position() >= 10 {
                self.backend.seek_start();
                return;
            }

            if self.queue.is_first_song() {
                return;
            }

            debug!("Marking '{}' as not playing", current_song.uri());
            current_song.set_playing(false);
        }

        if let Some(prev_song) = self.queue.previous_song() {
            debug!("Playing previous: {}", prev_song.uri());

            let was_playing = self.state.playing();
            if was_playing {
                self.set_playback_state(PlaybackState::Paused);
            }

            for c in &self.controllers {
                c.set_song(&prev_song);
            }

            self.backend.set_song_uri(Some(&prev_song.uri()));
            self.backend.seek_start();

            debug!("Marking '{}' as playing", prev_song.uri());
            prev_song.set_playing(true);

            self.state.set_current_song(Some(prev_song));

            if was_playing {
                self.set_playback_state(PlaybackState::Playing);
            }
        }
    }

    pub fn skip_next(&self) {
        if self.queue.is_empty() {
            return;
        }

        if let Some(current_song) = self.state.current_song() {
            current_song.set_playing(false);
        }

        if let Some(next_song) = self.queue.next_song() {
            debug!("Playing next (skip-next): {}", next_song.uri());

            let was_playing = self.state.playing();
            if was_playing {
                self.set_playback_state(PlaybackState::Paused);
            }

            for c in &self.controllers {
                c.set_song(&next_song);
            }

            self.backend.set_song_uri(Some(&next_song.uri()));
            self.backend.seek_start();

            next_song.set_playing(true);

            self.state.set_current_song(Some(next_song));

            if was_playing {
                self.set_playback_state(PlaybackState::Playing);
            }
        } else {
            self.skip_to(0);
            self.set_playback_state(PlaybackState::Stopped);
        }
    }

    pub fn skip_to(&self, pos: u32) {
        if self.queue.is_empty() {
            return;
        }

        if Some(pos) == self.queue.current_song_index() {
            return;
        }

        if let Some(current_song) = self.state.current_song() {
            current_song.set_playing(false);
        }

        if let Some(song) = self.queue.skip_song(pos) {
            debug!("Playing next (skip-to): {}", song.uri());
            let was_playing = self.state.playing();
            if was_playing {
                self.set_playback_state(PlaybackState::Paused);
            }

            for c in &self.controllers {
                c.set_song(&song);
            }

            self.backend.set_song_uri(Some(&song.uri()));
            self.backend.seek_start();

            song.set_playing(true);

            self.state.set_current_song(Some(song));

            if was_playing {
                self.set_playback_state(PlaybackState::Playing);
            }
        } else {
            self.backend.set_song_uri(None);
            self.state.set_current_song(None);
            self.set_playback_state(PlaybackState::Stopped);
        }
    }

    fn seek(&self, offset: u64, direction: SeekDirection) {
        self.backend.seek(
            self.state.position(),
            self.state.duration(),
            offset,
            direction,
        );
    }

    pub fn seek_start(&self) {
        let position = self.state.position() + 1;
        self.backend.seek(
            position,
            self.state.duration(),
            position,
            SeekDirection::Backwards,
        );
    }

    pub fn seek_backwards(&self) {
        self.seek(10, SeekDirection::Backwards);
    }

    pub fn seek_forward(&self) {
        self.seek(10, SeekDirection::Forward);
    }

    pub fn seek_offset(&self, offset: i64) {
        let direction = if offset < 0 {
            SeekDirection::Backwards
        } else {
            SeekDirection::Forward
        };
        self.seek(offset.unsigned_abs(), direction);
    }

    pub fn seek_position_rel(&self, position: f64) {
        let duration = self.state.duration() as f64;
        let pos = (duration * position).clamp(0.0, duration);
        self.backend.seek_position(pos as u64);
    }

    pub fn seek_position_abs(&self, position: u64) {
        let pos = u64::max(position, self.state.duration());
        self.backend.seek_position(pos);
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn state(&self) -> &PlayerState {
        &self.state
    }

    pub fn waveform_generator(&self) -> &WaveformGenerator {
        &self.waveform_generator
    }

    pub fn set_current_song(&self, song: Option<Song>) {
        self.state.set_current_song(song);
    }

    fn update_position(&self, position: u64, notify: bool) {
        self.state.set_position(position);

        for c in &self.controllers {
            c.set_position(position, notify);
        }
    }

    fn update_volume(&self, volume: f64) {
        debug!("Updating volume to: {}", &volume);
        self.state.set_volume(volume);
    }

    pub fn set_volume(&self, volume: f64) {
        self.backend.set_volume(volume);
    }

    pub fn toggle_repeat_mode(&self) {
        let cur_mode = self.queue.repeat_mode();
        let new_mode = match cur_mode {
            RepeatMode::Consecutive => RepeatMode::RepeatAll,
            RepeatMode::RepeatAll => RepeatMode::RepeatOne,
            RepeatMode::RepeatOne => RepeatMode::Consecutive,
        };
        self.queue.set_repeat_mode(new_mode);

        for c in &self.controllers {
            c.set_repeat_mode(new_mode);
        }
    }

    fn update_repeat_mode(&self, repeat: RepeatMode) {
        if repeat != self.queue.repeat_mode() {
            self.queue.set_repeat_mode(repeat);

            for c in &self.controllers {
                c.set_repeat_mode(repeat);
            }
        }
    }

    fn present(&self) {
        if let Err(e) = self.app_sender.send_blocking(ApplicationAction::Present) {
            error!("Unable to send Present: {e}");
        }
    }

    pub fn clear_queue(&self) {
        self.stop();
        self.state.set_current_song(None);
        self.queue.clear();

        let mut cover_cache = CoverCache::global().lock().unwrap();
        cover_cache.clear();
    }

    pub fn remove_song(&self, song: &Song) {
        if song.playing() {
            self.skip_next();
        }

        self.queue.remove_song(song);

        if self.queue.is_empty() {
            self.state.set_current_song(None);
        }
    }

    pub fn set_replaygain(&self, replaygain: ReplayGainMode) {
        self.backend.set_replaygain(replaygain);
    }

    pub fn replaygain_available(&self) -> bool {
        self.backend.replaygain_available()
    }
}
