// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use glib::{clone, Sender};
use gst::prelude::*;
use gtk::glib;
use gtk_macros::send;
use log::{debug, error, warn};

use crate::audio::{PlaybackAction, ReplayGainMode, SeekDirection};

#[derive(Debug)]
pub struct GstBackend {
    sender: Sender<PlaybackAction>,
    gst_player: gst_player::Player,
    replaygain: Option<GstReplayGain>,
}

#[derive(Debug)]
pub struct GstReplayGain {
    rg_filter_bin: gst::Element,
    rg_volume: gst::Element,
}

impl GstReplayGain {
    pub fn new() -> Result<GstReplayGain, Box<dyn std::error::Error>> {
        let rg_volume = gst::ElementFactory::make("rgvolume", Some("rg volume"))?;
        let rg_limiter = gst::ElementFactory::make("rglimiter", Some("rg limiter"))?;

        let filter_bin = gst::Bin::new(Some("filter bin"));
        filter_bin.add(&rg_volume)?;
        filter_bin.add(&rg_limiter)?;
        rg_volume.link(&rg_limiter)?;

        let pad_src = rg_limiter.static_pad("src").unwrap();
        pad_src.set_active(true).unwrap();
        let ghost_src = gst::GhostPad::with_target(Some("src"), &pad_src)?;
        filter_bin.add_pad(&ghost_src)?;

        let pad_sink = rg_volume.static_pad("sink").unwrap();
        pad_sink.set_active(true).unwrap();
        let ghost_sink = gst::GhostPad::with_target(Some("sink"), &pad_sink)?;
        filter_bin.add_pad(&ghost_sink)?;

        Ok(Self {
            rg_filter_bin: filter_bin.upcast(),
            rg_volume,
        })
    }

    pub fn set_mode(&self, playbin: gst::Element, replaygain: ReplayGainMode) {
        let identity = gst::ElementFactory::make("identity", Some("identity")).unwrap();

        let (filter, album_mode) = match replaygain {
            ReplayGainMode::Album => (self.rg_filter_bin.as_ref(), true),
            ReplayGainMode::Track => (self.rg_filter_bin.as_ref(), false),
            ReplayGainMode::Off => (&identity, true),
        };

        self.rg_volume.set_property("album-mode", album_mode);
        playbin.set_property("audio-filter", filter);
    }
}

impl GstBackend {
    pub fn new(sender: Sender<PlaybackAction>) -> Self {
        let dispatcher = gst_player::PlayerGMainContextSignalDispatcher::new(None);
        let gst_player = gst_player::Player::new(
            None,
            Some(&dispatcher.upcast::<gst_player::PlayerSignalDispatcher>()),
        );
        gst_player.set_video_track_enabled(false);

        let mut config = gst_player.config();
        config.set_position_update_interval(250);
        gst_player.set_config(config).unwrap();

        let res = Self {
            sender,
            gst_player,
            replaygain: GstReplayGain::new().ok(),
        };

        res.setup_signals();

        res
    }

    fn setup_signals(&self) {
        self.gst_player.connect_warning(move |_, warn| {
            warn!("GStreamer warning: {}", warn);
        });

        self.gst_player
            .connect_end_of_stream(clone!(@strong self.sender as sender => move |_| {
                send!(sender, PlaybackAction::PlayNext);
            }));

        self.gst_player.connect_position_updated(
            clone!(@strong self.sender as sender => move |_, clock| {
                if let Some(clock) = clock {
                    send!(sender, PlaybackAction::UpdatePosition(clock.seconds()));
                }
            }),
        );

        self.gst_player.connect_volume_changed(
            clone!(@strong self.sender as sender => move |player| {
                let volume = gst_audio::StreamVolume::convert_volume(
                    gst_audio::StreamVolumeFormat::Linear,
                    gst_audio::StreamVolumeFormat::Cubic,
                    player.volume(),
                );
                send!(sender, PlaybackAction::VolumeChanged(volume));
            }),
        );
    }

    pub fn set_song_uri(&self, uri: Option<&str>) {
        // FIXME: https://gitlab.freedesktop.org/gstreamer/gstreamer/-/issues/1124
        if uri.is_some() {
            self.gst_player.set_uri(uri);
        }
    }

    pub fn seek(&self, position: u64, duration: u64, offset: u64, direction: SeekDirection) {
        let offset = gst::ClockTime::from_seconds(offset);
        let position = gst::ClockTime::from_seconds(position);
        let duration = gst::ClockTime::from_seconds(duration);

        let destination = match direction {
            SeekDirection::Backwards if position >= offset => position.checked_sub(offset),
            SeekDirection::Backwards if position < offset => Some(gst::ClockTime::from_seconds(0)),
            SeekDirection::Forward if !duration.is_zero() && position + offset <= duration => {
                position.checked_add(offset)
            }
            SeekDirection::Forward if !duration.is_zero() && position + offset > duration => {
                Some(duration)
            }
            _ => None,
        };

        if let Some(destination) = destination {
            self.gst_player.seek(destination);
        }
    }

    pub fn seek_position(&self, position: u64) {
        self.gst_player.seek(gst::ClockTime::from_seconds(position));
    }

    pub fn seek_start(&self) {
        self.gst_player.seek(gst::ClockTime::from_seconds(0));
    }

    pub fn play(&self) {
        self.gst_player.play();
    }

    pub fn pause(&self) {
        self.gst_player.pause();
    }

    pub fn stop(&self) {
        self.gst_player.stop();
    }

    pub fn set_volume(&self, volume: f64) {
        let linear_volume = gst_audio::StreamVolume::convert_volume(
            gst_audio::StreamVolumeFormat::Cubic,
            gst_audio::StreamVolumeFormat::Linear,
            volume,
        );
        debug!("Setting volume to: {}", &linear_volume);
        self.gst_player.set_volume(linear_volume);
    }

    pub fn set_replaygain(&self, replaygain: ReplayGainMode) {
        self.replaygain
            .as_ref()
            .map(|r| r.set_mode(self.gst_player.pipeline(), replaygain));
    }

    pub fn replaygain_available(&self) -> bool {
        self.replaygain.is_some()
    }
}
