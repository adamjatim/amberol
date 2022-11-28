// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, CompositeTemplate};

use crate::{
    audio::RepeatMode, i18n::i18n, utils, volume_control::VolumeControl,
    waveform_view::WaveformView,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/bassi/Amberol/playback-control.ui")]
    pub struct PlaybackControl {
        // Template widgets
        #[template_child]
        pub start_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub center_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub end_box: TemplateChild<gtk::Box>,

        #[template_child]
        pub waveform_view: TemplateChild<WaveformView>,

        #[template_child]
        pub elapsed_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub remaining_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub previous_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub play_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub next_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub volume_control: TemplateChild<VolumeControl>,

        #[template_child]
        pub playlist_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub shuffle_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub repeat_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackControl {
        const NAME: &'static str = "AmberolPlaybackControl";
        type Type = super::PlaybackControl;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("playbackcontrol");
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            VolumeControl::static_type();
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackControl {
        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.menu_button.set_primary(true);
        }
    }

    impl WidgetImpl for PlaybackControl {}
}

glib::wrapper! {
    pub struct PlaybackControl(ObjectSubclass<imp::PlaybackControl>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for PlaybackControl {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create PlaybackControl")
    }
}

impl PlaybackControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play_button(&self) -> gtk::Button {
        self.imp().play_button.get()
    }

    pub fn repeat_button(&self) -> gtk::Button {
        self.imp().repeat_button.get()
    }

    pub fn volume_control(&self) -> VolumeControl {
        self.imp().volume_control.get()
    }

    pub fn waveform_view(&self) -> WaveformView {
        self.imp().waveform_view.get()
    }

    pub fn set_remaining(&self, remaining: Option<u64>) {
        if let Some(remaining) = remaining {
            self.imp()
                .remaining_label
                .set_text(&utils::format_remaining_time(remaining as i64));
        } else {
            self.imp().remaining_label.set_text("0:00");
        }
    }

    pub fn set_elapsed(&self, elapsed: Option<u64>) {
        if let Some(elapsed) = elapsed {
            self.imp()
                .elapsed_label
                .set_text(&utils::format_time(elapsed as i64));
        } else {
            self.imp().elapsed_label.set_text("0:00");
        }
    }

    pub fn set_position(&self, position: f64) {
        self.imp().waveform_view.set_position(position);
    }

    pub fn set_repeat_mode(&self, repeat_mode: RepeatMode) {
        let repeat_button = self.imp().repeat_button.get();
        match repeat_mode {
            RepeatMode::Consecutive => {
                repeat_button.set_icon_name("media-playlist-consecutive-symbolic");
                repeat_button.set_tooltip_text(Some(&i18n("Enable Repeat")));
            }
            RepeatMode::RepeatAll => {
                repeat_button.set_icon_name("media-playlist-repeat-symbolic");
                repeat_button.set_tooltip_text(Some(&i18n("Repeat All Tracks")));
            }
            RepeatMode::RepeatOne => {
                repeat_button.set_icon_name("media-playlist-repeat-song-symbolic");
                repeat_button.set_tooltip_text(Some(&i18n("Repeat the Current Track")));
            }
        }
    }
}
