// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};

use gtk::{gdk, glib, prelude::*, subclass::prelude::*};

use crate::audio::{PlaybackState, Song};

mod imp {
    use glib::{
        ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecDouble, ParamSpecObject, ParamSpecString,
        ParamSpecUInt64,
    };
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug)]
    pub struct PlayerState {
        pub playback_state: Cell<PlaybackState>,
        pub position: Cell<u64>,
        pub current_song: RefCell<Option<Song>>,
        pub volume: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlayerState {
        const NAME: &'static str = "AmberolPlayerState";
        type Type = super::PlayerState;

        fn new() -> Self {
            Self {
                playback_state: Cell::new(PlaybackState::Stopped),
                position: Cell::new(0),
                current_song: RefCell::new(None),
                volume: Cell::new(1.0),
            }
        }
    }

    impl ObjectImpl for PlayerState {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new("playing", "", "", false, ParamFlags::READABLE),
                    ParamSpecUInt64::new("position", "", "", 0, u64::MAX, 0, ParamFlags::READABLE),
                    ParamSpecObject::new("song", "", "", Song::static_type(), ParamFlags::READABLE),
                    ParamSpecString::new("title", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("artist", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("album", "", "", None, ParamFlags::READABLE),
                    ParamSpecUInt64::new("duration", "", "", 0, u64::MAX, 0, ParamFlags::READABLE),
                    ParamSpecObject::new(
                        "cover",
                        "",
                        "",
                        gdk::Texture::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecDouble::new("volume", "", "", 0.0, 1.0, 1.0, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "playing" => obj.playing().to_value(),
                "position" => obj.position().to_value(),
                "song" => self.current_song.borrow().to_value(),
                "volume" => obj.volume().to_value(),

                // These are proxies for Song properties
                "title" => obj.title().to_value(),
                "artist" => obj.artist().to_value(),
                "album" => obj.album().to_value(),
                "duration" => obj.duration().to_value(),
                "cover" => obj.cover().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

// PlayerState is a GObject that we can use to bind to
// widgets and other objects; it contains the current
// state of the audio player: song metadata, playback
// position and duration, etc.
glib::wrapper! {
    pub struct PlayerState(ObjectSubclass<imp::PlayerState>);
}

impl PlayerState {
    pub fn title(&self) -> Option<String> {
        if let Some(song) = &*self.imp().current_song.borrow() {
            return Some(song.title());
        }

        None
    }

    pub fn artist(&self) -> Option<String> {
        if let Some(song) = &*self.imp().current_song.borrow() {
            return Some(song.artist());
        }

        None
    }

    pub fn album(&self) -> Option<String> {
        if let Some(song) = &*self.imp().current_song.borrow() {
            return Some(song.album());
        }

        None
    }

    pub fn duration(&self) -> u64 {
        if let Some(song) = &*self.imp().current_song.borrow() {
            return song.duration();
        }

        0
    }

    pub fn cover(&self) -> Option<gdk::Texture> {
        if let Some(song) = &*self.imp().current_song.borrow() {
            return song.cover_texture();
        }

        None
    }

    pub fn playing(&self) -> bool {
        let playback_state = self.imp().playback_state.get();
        matches!(playback_state, PlaybackState::Playing)
    }

    pub fn set_playback_state(&self, playback_state: &PlaybackState) -> bool {
        let old_state = self.imp().playback_state.replace(*playback_state);
        if old_state != *playback_state {
            self.notify("playing");
            return true;
        }

        false
    }

    pub fn current_song(&self) -> Option<Song> {
        (*self.imp().current_song.borrow()).as_ref().cloned()
    }

    pub fn set_current_song(&self, song: Option<Song>) {
        self.imp().current_song.replace(song);
        self.imp().position.replace(0);
        self.notify("song");
        self.notify("title");
        self.notify("artist");
        self.notify("album");
        self.notify("duration");
        self.notify("cover");
        self.notify("position");
    }

    pub fn position(&self) -> u64 {
        self.imp().position.get()
    }

    pub fn set_position(&self, position: u64) {
        self.imp().position.replace(position);
        self.notify("position");
    }

    pub fn volume(&self) -> f64 {
        self.imp().volume.get()
    }

    pub fn set_volume(&self, volume: f64) {
        let old_volume = self.imp().volume.replace(volume);
        // We only care about two digits of precision, to avoid
        // notification cycles when we update the volume with a
        // similar value coming from the volume control
        let old_rounded = format!("{:.2}", old_volume);
        let new_rounded = format!("{:.2}", volume);
        if old_rounded != new_rounded {
            self.notify("volume");
        }
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        glib::Object::new::<Self>(&[]).expect("Unable to create PlayerState instance")
    }
}
