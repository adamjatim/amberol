// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::Cell;

use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use crate::audio::{RepeatMode, ShuffleListModel, Song};

mod imp {
    use glib::{ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecObject, ParamSpecUInt, Value};
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, Default)]
    pub struct Queue {
        pub model: ShuffleListModel,
        pub store: gio::ListStore,
        pub repeat_mode: Cell<RepeatMode>,
        pub current_pos: Cell<Option<u32>>,
        pub shuffled: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Queue {
        const NAME: &'static str = "AmberolQueue";
        type Type = super::Queue;

        fn new() -> Self {
            let store = gio::ListStore::new(Song::static_type());
            let model = ShuffleListModel::new(Some(&store));

            Self {
                store,
                model,
                repeat_mode: Cell::new(RepeatMode::default()),
                current_pos: Cell::new(None),
                shuffled: Cell::new(false),
            }
        }
    }

    impl ObjectImpl for Queue {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "current",
                        "",
                        "",
                        Song::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecEnum::new(
                        "repeat-mode",
                        "",
                        "",
                        RepeatMode::static_type(),
                        0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecUInt::new("n-songs", "", "", 0, u32::MAX, 0, ParamFlags::READABLE),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "current" => obj.current_song().to_value(),
                "repeat-mode" => self.repeat_mode.get().to_value(),
                "n-songs" => self.store.n_items().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Queue(ObjectSubclass<imp::Queue>);
}

impl Default for Queue {
    fn default() -> Self {
        glib::Object::new::<Self>(&[]).expect("Failed to create Queue object")
    }
}

impl Queue {
    pub fn n_songs(&self) -> u32 {
        self.imp().model.n_items()
    }

    pub fn is_empty(&self) -> bool {
        self.imp().model.n_items() == 0
    }

    pub fn model(&self) -> &gio::ListModel {
        self.imp().model.as_ref()
    }

    pub fn song_at(&self, pos: u32) -> Option<Song> {
        if let Some(song) = self.imp().model.item(pos) {
            return Some(song.downcast::<Song>().unwrap());
        }

        None
    }

    pub fn current_song(&self) -> Option<Song> {
        if let Some(pos) = self.imp().current_pos.get() {
            return self.song_at(pos);
        }

        None
    }

    pub fn set_current_song(&self, song: Option<Song>) {
        if let Some(song) = song {
            for i in 0..self.n_songs() {
                let s = self.song_at(i).unwrap();
                if song.equals(&s) {
                    self.imp().current_pos.replace(Some(i));
                    self.notify("current");
                    return;
                }
            }
        } else {
            self.imp().current_pos.replace(None);
            self.notify("current");
        }
    }

    pub fn current_song_index(&self) -> Option<u32> {
        self.imp().current_pos.get()
    }

    pub fn add_song(&self, song: &Song) -> bool {
        if !song.equals(&Song::default()) {
            // Add song to the backing store
            self.imp().store.append(song);
            self.notify("n-songs");
            true
        } else {
            false
        }
    }

    pub fn add_songs(&self, songs: &[impl IsA<glib::Object>]) {
        self.imp()
            .store
            .splice(self.imp().model.n_items(), 0, songs);
        self.notify("n-songs");
    }

    pub fn remove_song(&self, song: &Song) {
        let was_shuffled = self.imp().model.shuffled();
        let n_songs = self.n_songs();
        for pos in 0..n_songs {
            let s = self
                .imp()
                .store
                .item(pos)
                .unwrap()
                .downcast::<Song>()
                .unwrap();
            if s.equals(song) {
                self.imp().store.remove(pos);
                break;
            }
        }

        if n_songs != self.n_songs() {
            if was_shuffled {
                self.imp().model.reshuffle(0);
            }
            self.notify("n-songs");
        }

        if self.is_empty() {
            self.imp().current_pos.replace(None);
        }
    }

    pub fn clear(&self) {
        self.imp().current_pos.replace(None);
        self.imp().store.remove_all();
        self.notify("n-songs");
    }

    pub fn skip_song(&self, pos: u32) -> Option<Song> {
        self.imp().current_pos.replace(Some(pos));
        self.notify("current");
        self.song_at(pos)
    }

    pub fn previous_song(&self) -> Option<Song> {
        if let Some(current_pos) = self.imp().current_pos.get() {
            if current_pos > 0 {
                let prev = current_pos - 1;
                self.imp().current_pos.replace(Some(prev));
                self.notify("current");
                return self.song_at(current_pos - 1);
            }
        }

        None
    }

    pub fn next_song(&self) -> Option<Song> {
        let store = &self.imp().model;

        let n_songs = store.n_items();
        if n_songs == 0 {
            return None;
        }

        let repeat_mode = self.imp().repeat_mode.get();
        if let Some(current) = self.current_song_index() {
            let next: Option<u32> = match repeat_mode {
                RepeatMode::Consecutive if current < n_songs - 1 => Some(current + 1),
                RepeatMode::Consecutive if current == n_songs - 1 => None,
                RepeatMode::RepeatOne => Some(current),
                RepeatMode::RepeatAll if current < n_songs - 1 => Some(current + 1),
                RepeatMode::RepeatAll if current == n_songs - 1 => Some(0),
                _ => None,
            };

            if let Some(next) = next {
                self.imp().current_pos.replace(Some(next));
                self.notify("current");
                self.song_at(next)
            } else {
                self.imp().current_pos.replace(None);
                self.notify("current");
                None
            }
        } else {
            // Return the first song
            self.imp().current_pos.replace(Some(0));
            self.notify("current");
            self.song_at(0)
        }
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.imp().repeat_mode.get()
    }

    pub fn set_repeat_mode(&self, repeat_mode: RepeatMode) {
        let old_mode = self.imp().repeat_mode.replace(repeat_mode);
        if old_mode != repeat_mode {
            self.notify("repeat-mode");
        }
    }

    pub fn is_first_song(&self) -> bool {
        if let Some(current_pos) = self.imp().current_pos.get() {
            return current_pos == 0;
        }

        false
    }

    pub fn is_last_song(&self) -> bool {
        let n_items = self.imp().model.n_items();

        if let Some(current_pos) = self.imp().current_pos.get() {
            if n_items > 0 {
                return current_pos == n_items - 1;
            }
        }

        false
    }

    pub fn is_shuffled(&self) -> bool {
        self.imp().shuffled.get()
    }

    pub fn set_shuffled(&self, shuffled: bool) {
        if shuffled != self.imp().shuffled.replace(shuffled) {
            if shuffled {
                let current_pos = self.imp().current_pos.get().unwrap_or(0);
                self.imp().model.reshuffle(current_pos);
            } else {
                let current_pos = self.current_song_index().unwrap_or(0);
                let current_song = self.song_at(current_pos);
                self.imp().model.unshuffle();
                self.set_current_song(current_song);
            }
        }
    }

    pub fn select_song_at(&self, index: u32) {
        if let Some(song) = self.imp().model.item(index) {
            let song = song.downcast_ref::<Song>().unwrap();
            let is_selected = !song.selected();
            song.set_selected(is_selected);
        }
    }

    pub fn unselect_all_songs(&self) {
        for i in 0..self.imp().store.n_items() {
            let song = self.imp().store.item(i).unwrap();
            song.downcast_ref::<Song>().unwrap().set_selected(false);
        }
    }

    pub fn n_selected_songs(&self) -> u32 {
        let mut count = 0;
        for i in 0..self.imp().store.n_items() {
            let song = self.imp().store.item(i).unwrap();
            if song.downcast_ref::<Song>().unwrap().selected() {
                count += 1;
            }
        }

        count
    }
}
