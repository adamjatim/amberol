// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};

use async_channel::Sender;
use glib::clone;
use gtk::{gio, glib, prelude::*};
use log::error;
use mpris_server::{LoopStatus, Metadata, PlaybackStatus, Player, Time};

use crate::{
    audio::{Controller, PlaybackAction, PlaybackState, RepeatMode, Song},
    config::APPLICATION_ID,
};

#[derive(Debug)]
pub struct MprisController {
    mpris: Rc<OnceCell<Player>>,
    song: RefCell<Option<Song>>,
}

impl MprisController {
    pub fn new(sender: Sender<PlaybackAction>) -> Self {
        let builder = Player::builder(APPLICATION_ID)
            .identity("Amberol")
            .desktop_entry(APPLICATION_ID)
            .can_raise(true)
            .can_play(false)
            .can_pause(true)
            .can_seek(true)
            .can_go_next(true)
            .can_go_previous(true)
            .can_set_fullscreen(false);

        let mpris = Rc::new(OnceCell::new());

        glib::spawn_future_local(clone!(
            #[weak]
            mpris,
            #[strong]
            sender,
            async move {
                match builder.build().await {
                    Err(err) => error!("Failed to create MPRIS server: {:?}", err),
                    Ok(player) => {
                        setup_signals(sender, &player);
                        let mpris_task = player.run();
                        let _ = mpris.set(player);
                        mpris_task.await;
                    }
                }
            }
        ));

        Self {
            mpris,
            song: RefCell::new(None),
        }
    }

    fn update_metadata(&self) {
        let mut metadata = Metadata::new();

        if let Some(song) = self.song.take() {
            metadata.set_artist(Some(vec![song.artist()]));
            metadata.set_title(Some(song.title()));
            metadata.set_album(Some(song.album()));

            let length = Time::from_secs(song.duration() as i64);
            metadata.set_length(Some(length));

            // MPRIS should really support passing a bytes buffer for
            // the cover art, instead of requiring this ridiculous
            // charade
            if let Some(cache) = song.cover_cache() {
                let file = gio::File::for_path(cache);
                match file.query_info(
                    "standard::type",
                    gio::FileQueryInfoFlags::NONE,
                    gio::Cancellable::NONE,
                ) {
                    Ok(info) if info.file_type() == gio::FileType::Regular => {
                        metadata.set_art_url(Some(file.uri()));
                    }
                    _ => metadata.set_art_url(None::<String>),
                }
            }

            self.song.replace(Some(song));
        }

        glib::spawn_future_local(clone!(
            #[weak(rename_to = mpris)]
            self.mpris,
            async move {
                if let Some(mpris) = mpris.get() {
                    if let Err(err) = mpris.set_metadata(metadata).await {
                        error!("Unable to set MPRIS metadata: {err:?}");
                    }
                }
            }
        ));
    }
}

impl Controller for MprisController {
    fn set_playback_state(&self, state: &PlaybackState) {
        let status = match state {
            PlaybackState::Playing => PlaybackStatus::Playing,
            PlaybackState::Paused => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        };

        glib::spawn_future_local(clone!(
            #[weak(rename_to = mpris)]
            self.mpris,
            async move {
                if let Some(mpris) = mpris.get() {
                    if let Err(err) = mpris.set_can_play(true).await {
                        error!("Unable to set MPRIS play capability: {err:?}");
                    }
                    if let Err(err) = mpris.set_playback_status(status).await {
                        error!("Unable to set MPRIS playback status: {err:?}");
                    }
                }
            }
        ));
    }

    fn set_song(&self, song: &Song) {
        self.song.replace(Some(song.clone()));
        self.update_metadata();
    }

    fn set_position(&self, position: u64, notify: bool) {
        let pos = Time::from_secs(position as i64);
        if let Some(mpris) = self.mpris.get() {
            mpris.set_position(pos);
        }
        if notify {
            glib::spawn_future_local(clone!(
                #[weak(rename_to = mpris)]
                self.mpris,
                async move {
                    if let Some(mpris) = mpris.get() {
                        if let Err(err) = mpris.seeked(pos).await {
                            error!("Unable to emit MPRIS Seeked: {err:?}");
                        }
                    }
                }
            ));
        }
    }

    fn set_repeat_mode(&self, repeat: RepeatMode) {
        let status = match repeat {
            RepeatMode::Consecutive => LoopStatus::None,
            RepeatMode::RepeatOne => LoopStatus::Track,
            RepeatMode::RepeatAll => LoopStatus::Playlist,
        };

        glib::spawn_future_local(clone!(
            #[weak(rename_to = mpris)]
            self.mpris,
            async move {
                if let Some(mpris) = mpris.get() {
                    if let Err(err) = mpris.set_loop_status(status).await {
                        error!("Unable to set MPRIS loop status: {err:?}");
                    }
                }
            }
        ));
    }
}

fn setup_signals(sender: Sender<PlaybackAction>, mpris: &Player) {
    mpris.connect_play_pause(clone!(
        #[strong]
        sender,
        move |player| {
            match player.playback_status() {
                PlaybackStatus::Paused => {
                    if let Err(e) = sender.send_blocking(PlaybackAction::Play) {
                        error!("Unable to send Play: {e}");
                    }
                }
                PlaybackStatus::Stopped => {
                    if let Err(e) = sender.send_blocking(PlaybackAction::Stop) {
                        error!("Unable to send Stop: {e}");
                    }
                }
                _ => {
                    if let Err(e) = sender.send_blocking(PlaybackAction::Pause) {
                        error!("Unable to send Pause: {e}");
                    }
                }
            };
        }
    ));

    mpris.connect_play(clone!(
        #[strong]
        sender,
        move |_| {
            if let Err(e) = sender.send_blocking(PlaybackAction::Play) {
                error!("Unable to send Play: {e}");
            }
        }
    ));

    mpris.connect_stop(clone!(
        #[strong]
        sender,
        move |_| {
            if let Err(e) = sender.send_blocking(PlaybackAction::Stop) {
                error!("Unable to send Stop: {e}");
            }
        }
    ));

    mpris.connect_pause(clone!(
        #[strong]
        sender,
        move |_| {
            if let Err(e) = sender.send_blocking(PlaybackAction::Pause) {
                error!("Unable to send Pause: {e}");
            }
        }
    ));

    mpris.connect_previous(clone!(
        #[strong]
        sender,
        move |_| {
            if let Err(e) = sender.send_blocking(PlaybackAction::SkipPrevious) {
                error!("Unable to send SkipPrevious: {e}");
            }
        }
    ));

    mpris.connect_next(clone!(
        #[strong]
        sender,
        move |_| {
            if let Err(e) = sender.send_blocking(PlaybackAction::SkipNext) {
                error!("Unable to send SkipNext: {e}");
            }
        }
    ));

    mpris.connect_raise(clone!(
        #[strong]
        sender,
        move |_| {
            if let Err(e) = sender.send_blocking(PlaybackAction::Raise) {
                error!("Unable to send Raise: {e}");
            }
        }
    ));

    mpris.connect_set_loop_status(clone!(
        #[strong]
        sender,
        move |_, status| {
            let mode = match status {
                LoopStatus::None => RepeatMode::Consecutive,
                LoopStatus::Track => RepeatMode::RepeatOne,
                LoopStatus::Playlist => RepeatMode::RepeatAll,
            };

            if let Err(e) = sender.send_blocking(PlaybackAction::Repeat(mode)) {
                error!("Unable to send Repeat({mode}): {e}");
            }
        }
    ));

    mpris.connect_seek(clone!(
        #[strong]
        sender,
        move |_, offset| {
            let offset = offset.as_secs();
            if let Err(e) = sender.send_blocking(PlaybackAction::Seek(offset)) {
                error!("Unable to send Seek({offset}): {e}");
            }
        }
    ));
}
