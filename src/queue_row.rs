// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};

use adw::subclass::prelude::*;
use glib::clone;
use gtk::{gdk, gio, glib, prelude::*, CompositeTemplate};

use crate::{audio::Song, cover_picture::CoverPicture};

mod imp {
    use glib::{ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString, Value};
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/bassi/Amberol/queue-row.ui")]
    pub struct QueueRow {
        // Template widgets
        #[template_child]
        pub row_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub song_cover_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub song_cover_image: TemplateChild<CoverPicture>,
        #[template_child]
        pub song_title_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub song_artist_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub song_playing_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub selection_title_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub selection_artist_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub selected_button: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub selection_playing_image: TemplateChild<gtk::Image>,

        pub song: RefCell<Option<Song>>,
        pub playing: Cell<bool>,
        pub selection_mode: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QueueRow {
        const NAME: &'static str = "AmberolQueueRow";
        type Type = super::QueueRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.set_css_name("queuerow");
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for QueueRow {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.obj().init_widgets();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<Song>("song").build(),
                    ParamSpecString::builder("song-artist").build(),
                    ParamSpecString::builder("song-title").build(),
                    ParamSpecObject::builder::<gdk::Texture>("song-cover").build(),
                    ParamSpecBoolean::builder("playing").build(),
                    ParamSpecBoolean::builder("selection-mode").build(),
                    ParamSpecBoolean::builder("selected").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "song" => {
                    let song = value.get::<Option<Song>>().unwrap();
                    self.song.replace(song);
                }
                "song-artist" => {
                    let p = value.get::<&str>().expect("The value needs to be a string");
                    self.obj().set_song_artist(p);
                }
                "song-title" => {
                    let p = value.get::<&str>().expect("The value needs to be a string");
                    self.obj().set_song_title(p);
                }
                "song-cover" => {
                    let p = value.get::<gdk::Texture>().ok();
                    self.obj().set_song_cover(p);
                }
                "playing" => {
                    let p = value
                        .get::<bool>()
                        .expect("The value needs to be a boolean");
                    self.obj().set_playing(p);
                }
                "selection-mode" => {
                    let p = value
                        .get::<bool>()
                        .expect("The value needs to be a boolean");
                    self.obj().set_selection_mode(p);
                }
                "selected" => {
                    let p = value
                        .get::<bool>()
                        .expect("The value needs to be a boolean");
                    self.selected_button.set_active(p);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "song" => self.song.borrow().to_value(),
                "song-artist" => self.song_artist_label.text().to_value(),
                "song-title" => self.song_title_label.text().to_value(),
                "song-cover" => self.song_cover_image.cover().to_value(),
                "playing" => self.playing.get().to_value(),
                "selection-mode" => self.selection_mode.get().to_value(),
                "selected" => self.selected_button.is_active().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for QueueRow {}
}

glib::wrapper! {
    pub struct QueueRow(ObjectSubclass<imp::QueueRow>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for QueueRow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl QueueRow {
    pub fn new() -> Self {
        Self::default()
    }

    fn init_widgets(&self) {
        self.imp().selected_button.connect_active_notify(clone!(
            #[strong(rename_to = this)]
            self,
            move |button| {
                if let Some(ref song) = *this.imp().song.borrow() {
                    song.set_selected(button.is_active());
                }
                this.notify("selected");
            }
        ));
    }

    fn set_playing(&self, playing: bool) {
        if playing != self.imp().playing.replace(playing) {
            self.update_mode();
            self.notify("playing");
        }
    }

    fn set_selection_mode(&self, selection_mode: bool) {
        if selection_mode != self.imp().selection_mode.replace(selection_mode) {
            self.update_mode();
            self.notify("selection-mode");
        }
    }

    fn update_mode(&self) {
        let imp = self.imp();
        if imp.selection_mode.get() {
            imp.row_stack.set_visible_child_name("selection-mode");
            let opacity = if imp.playing.get() { 1.0 } else { 0.0 };
            imp.selection_playing_image.set_opacity(opacity);
        } else {
            imp.row_stack.set_visible_child_name("song-details");
            let opacity = if imp.playing.get() { 1.0 } else { 0.0 };
            imp.song_playing_image.set_opacity(opacity);
        }
    }

    fn set_song_title(&self, title: &str) {
        let imp = self.imp();
        imp.song_title_label.set_text(Some(title));
        imp.selection_title_label.set_text(Some(title));
    }

    fn set_song_artist(&self, artist: &str) {
        let imp = self.imp();
        imp.song_artist_label.set_text(Some(artist));
        imp.selection_artist_label.set_text(Some(artist));
    }

    fn set_song_cover(&self, cover: Option<gdk::Texture>) {
        let imp = self.imp();
        if let Some(texture) = cover {
            imp.song_cover_image.set_cover(Some(&texture));
            imp.song_cover_stack.set_visible_child_name("cover");
        } else {
            imp.song_cover_image.set_cover(None);
            imp.song_cover_stack.set_visible_child_name("no-cover");
        }
    }

    pub fn song(&self) -> Option<Song> {
        self.imp().song.borrow().clone()
    }
}
