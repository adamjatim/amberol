// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Instant,
};

use adw::subclass::prelude::*;
#[cfg(target_os = "linux")]
use ashpd::{desktop::background, WindowIdentifier};
use glib::{clone, closure_local, FromVariant};
use gtk::{gdk, gio, glib, prelude::*, CompositeTemplate};
use gtk_macros::stateful_action;
use log::{debug, warn};

use crate::{
    audio::{AudioPlayer, ReplayGainMode, Song, WaveformGenerator},
    config::APPLICATION_ID,
    drag_overlay::DragOverlay,
    i18n::{i18n, i18n_k, ni18n_f, ni18n_k},
    playback_control::PlaybackControl,
    playlist_view::PlaylistView,
    queue_row::QueueRow,
    search::FuzzyFilter,
    song_details::SongDetails,
    sort::FuzzySorter,
    utils,
    waveform_view::WaveformView,
};

pub enum WindowMode {
    InitialView,
    MainView,
}

mod imp {
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecEnum, Value};
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(CompositeTemplate)]
    #[template(resource = "/io/bassi/Amberol/window.ui")]
    pub struct Window {
        // Template widgets
        #[template_child]
        pub drag_overlay: TemplateChild<DragOverlay>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub song_details: TemplateChild<SongDetails>,
        #[template_child]
        pub playback_control: TemplateChild<PlaybackControl>,
        #[template_child]
        pub queue_revealer: TemplateChild<adw::Flap>,
        #[template_child]
        pub playlist_view: TemplateChild<PlaylistView>,
        #[template_child]
        pub add_folder_button: TemplateChild<gtk::Button>,

        pub provider: gtk::CssProvider,
        pub waveform: WaveformGenerator,
        pub settings: gio::Settings,

        pub playlist_shuffled: Cell<bool>,
        pub playlist_visible: Cell<bool>,
        pub playlist_selection: Cell<bool>,
        pub playlist_search: Cell<bool>,
        pub replaygain_mode: Cell<ReplayGainMode>,

        pub playlist_filtermodel: RefCell<Option<gio::ListModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "AmberolWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("win.play", None, move |win, _, _| {
                debug!("Window::win.play()");
                win.player().toggle_play();
            });
            klass.install_action("win.seek-backwards", None, move |win, _, _| {
                debug!("Window::win.seek-backwards");
                win.player().seek_backwards();
            });
            klass.install_action("win.seek-forward", None, move |win, _, _| {
                debug!("Window::win.seek-forward");
                win.player().seek_forward();
            });
            klass.install_action("win.previous", None, move |win, _, _| {
                debug!("Window::win.previous()");
                win.player().skip_previous();
            });
            klass.install_action("win.next", None, move |win, _, _| {
                debug!("Window::win.next()");
                win.player().skip_next();
            });
            klass.install_action("queue.repeat-mode", None, move |win, _, _| {
                debug!("Window::queue.repeat()");
                win.player().toggle_repeat_mode();
            });
            klass.install_action("queue.add-song", None, move |win, _, _| {
                debug!("Window::win.add-song()");
                win.add_song();
            });
            klass.install_action("queue.add-folder", None, move |win, _, _| {
                debug!("Window::win.add-folder()");
                win.add_folder();
            });
            klass.install_action("win.copy", None, move |win, _, _| {
                debug!("Window::win.copy()");
                win.copy_song();
            });
            klass.install_action("queue.clear", None, move |win, _, _| {
                debug!("Window::queue.clear()");
                win.clear_queue();
            });
            klass.install_property_action("queue.toggle", "playlist-visible");
            klass.install_property_action("queue.shuffle", "playlist-shuffled");
            klass.install_property_action("queue.select", "playlist-selection");
            klass.install_property_action("queue.search", "playlist-search");
            klass.install_property_action("win.replaygain", "replaygain-mode");

            klass.install_action("win.skip-to", Some("u"), move |win, _, param| {
                if let Some(pos) = param.and_then(u32::from_variant) {
                    win.player().skip_to(pos);
                    win.player().play();
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                song_details: TemplateChild::default(),
                queue_revealer: TemplateChild::default(),
                toast_overlay: TemplateChild::default(),
                drag_overlay: TemplateChild::default(),
                playback_control: TemplateChild::default(),
                main_stack: TemplateChild::default(),
                status_page: TemplateChild::default(),
                add_folder_button: TemplateChild::default(),
                playlist_view: TemplateChild::default(),
                playlist_shuffled: Cell::new(false),
                playlist_visible: Cell::new(true),
                playlist_selection: Cell::new(false),
                playlist_search: Cell::new(false),
                playlist_filtermodel: RefCell::default(),
                replaygain_mode: Cell::new(ReplayGainMode::default()),
                waveform: WaveformGenerator::default(),
                provider: gtk::CssProvider::new(),
                settings: utils::settings_manager(),
            }
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if APPLICATION_ID.ends_with("Devel") {
                obj.add_css_class("devel");
            }
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new(
                        "playlist-shuffled",
                        "",
                        "",
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new("playlist-visible", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new(
                        "playlist-selection",
                        "",
                        "",
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new("playlist-search", "", "", false, ParamFlags::READWRITE),
                    ParamSpecEnum::new(
                        "replaygain-mode",
                        "",
                        "",
                        ReplayGainMode::static_type(),
                        ReplayGainMode::default() as i32,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "playlist-shuffled" => obj.set_playlist_shuffled(value.get::<bool>().unwrap()),
                "playlist-visible" => obj.set_playlist_visible(value.get::<bool>().unwrap()),
                "playlist-selection" => obj.set_playlist_selection(value.get::<bool>().unwrap()),
                "playlist-search" => obj.set_playlist_search(value.get::<bool>().unwrap()),
                "replaygain-mode" => obj.set_replaygain(value.get::<ReplayGainMode>().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "playlist-shuffled" => obj.playlist_shuffled().to_value(),
                "playlist-visible" => obj.playlist_visible().to_value(),
                "playlist-selection" => obj.playlist_selection().to_value(),
                "playlist-search" => obj.playlist_search().to_value(),
                "replaygain-mode" => obj.replaygain().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Window {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        let win = glib::Object::new::<Window>(&[("application", application)])
            .expect("Failed to create Window");

        win.setup_waveform();
        win.setup_actions();
        win.setup_playlist();
        win.setup_drop_target();
        win.setup_provider();
        win.bind_state();
        win.bind_queue();
        win.connect_signals();
        win.restore_window_state();
        win.set_initial_state();

        #[cfg(target_os = "linux")]
        win.request_background();

        win
    }

    fn player(&self) -> Rc<AudioPlayer> {
        self.application()
            .unwrap()
            .downcast::<crate::application::Application>()
            .unwrap()
            .player()
            .clone()
    }

    fn setup_actions(&self) {
        let enable_recoloring = self.imp().settings.boolean("enable-recoloring");
        stateful_action!(
            self,
            "enable-recoloring",
            enable_recoloring,
            clone!(@weak self as this => move |action, _| {
                let state = action.state().unwrap();
                let action_state: bool = state.get().unwrap();
                let enable_recoloring = !action_state;
                action.set_state(&enable_recoloring.to_variant());

                this.imp()
                    .settings
                    .set_boolean("enable-recoloring", enable_recoloring)
                    .expect("Unable to store setting");
            })
        );
    }

    fn setup_waveform(&self) {
        self.imp().waveform.connect_notify_local(
            Some("has-peaks"),
            clone!(@weak self as win => move |gen, _| {
                let peaks = gen.peaks();
                win.imp().playback_control.waveform_view().set_peaks(peaks);
            }),
        );
    }

    fn restore_window_state(&self) {
        let settings = utils::settings_manager();
        let width = settings.int("window-width");
        let height = settings.int("window-height");
        self.set_default_size(width, height);
    }

    fn reset_queue(&self) {
        self.set_playlist_visible(false);
        self.set_playlist_shuffled(false);
        self.set_playlist_selection(false);
        self.update_waveform(None);
        self.update_style(None);
    }

    fn clear_queue(&self) {
        self.player().clear_queue();
    }

    fn playlist_visible(&self) -> bool {
        self.imp().playlist_visible.get()
    }

    fn set_playlist_visible(&self, visible: bool) {
        if visible != self.imp().playlist_visible.replace(visible) {
            self.imp().queue_revealer.set_reveal_flap(visible);
            self.notify("playlist-visible");
        }
    }

    fn playlist_shuffled(&self) -> bool {
        self.imp().playlist_shuffled.get()
    }

    fn set_playlist_shuffled(&self, shuffled: bool) {
        let imp = self.imp();

        if shuffled != imp.playlist_shuffled.replace(shuffled) {
            let player = self.player();
            let queue = player.queue();
            let state = player.state();

            let reset_song = queue.is_first_song() && !state.playing();

            queue.set_shuffled(shuffled);

            if reset_song {
                self.player().skip_to(0);
            }

            self.notify("playlist-shuffled");
        }
    }

    fn playlist_selection(&self) -> bool {
        self.imp().playlist_selection.get()
    }

    fn set_playlist_selection(&self, selection: bool) {
        let imp = self.imp();

        if selection != imp.playlist_selection.replace(selection) {
            if !selection {
                let player = self.player();
                let queue = player.queue();
                queue.unselect_all_songs();
            }

            self.imp()
                .playlist_view
                .queue_actionbar()
                .set_revealed(selection);

            self.notify("playlist-selection");
        }
    }

    fn playlist_search(&self) -> bool {
        self.imp().playlist_search.get()
    }

    fn set_playlist_search(&self, search: bool) {
        let imp = self.imp();

        if search != imp.playlist_search.replace(search) {
            imp.playlist_view.set_search(search);
            self.notify("playlist-search");
        }
    }

    fn add_song(&self) {
        let app = gio::Application::default()
            .expect("Failed to retrieve application singleton")
            .downcast::<gtk::Application>()
            .unwrap();
        let win = app
            .active_window()
            .unwrap()
            .downcast::<gtk::Window>()
            .unwrap();
        let dialog = gtk::FileChooserNative::builder()
            .accept_label(&i18n("_Add Song"))
            .cancel_label(&i18n("_Cancel"))
            .modal(true)
            .title(&i18n("Open File"))
            .action(gtk::FileChooserAction::Open)
            .select_multiple(true)
            .transient_for(&win)
            .build();

        let filter = gtk::FileFilter::new();
        gtk::FileFilter::set_name(&filter, Some(&i18n("Audio files")));
        filter.add_mime_type("audio/*");
        dialog.add_filter(&filter);

        dialog.connect_response(
            clone!(@strong dialog, @weak self as win => move |_, response| {
                if response == gtk::ResponseType::Accept {
                    let files = dialog.files();
                    if files.n_items() == 0 {
                        win.add_toast(i18n("Unable to access files"));
                    } else {
                        win.add_files_to_queue(&files);
                    }
                }
            }),
        );
        dialog.show();
    }

    fn add_folder(&self) {
        let app = gio::Application::default()
            .expect("Failed to retrieve application singleton")
            .downcast::<gtk::Application>()
            .unwrap();
        let win = app
            .active_window()
            .unwrap()
            .downcast::<gtk::Window>()
            .unwrap();
        let dialog = gtk::FileChooserNative::builder()
            .accept_label(&i18n("_Add Folder"))
            .cancel_label(&i18n("_Cancel"))
            .modal(true)
            .title(&i18n("Open Folder"))
            .action(gtk::FileChooserAction::SelectFolder)
            .select_multiple(true)
            .transient_for(&win)
            .build();

        dialog.connect_response(
            clone!(@strong dialog, @weak self as win => move |_, response| {
                if response == gtk::ResponseType::Accept {
                    let files = dialog.files();
                    if files.n_items() == 0 {
                        win.add_toast(i18n("Unable to access folders"));
                    } else {
                        win.add_files_to_queue(&dialog.files());
                    }
                }
            }),
        );
        dialog.show();
    }

    fn queue_songs(&self, queue: Vec<gio::File>) {
        if queue.is_empty() {
            self.add_toast(i18n("No available song found"));
            return;
        }

        self.switch_mode(WindowMode::MainView);

        // Disable actions on the queue; loading is "atomic"
        self.action_set_enabled("queue.add-song", false);
        self.action_set_enabled("queue.add-folder", false);
        self.action_set_enabled("queue.clear", false);

        self.set_playlist_visible(true);

        self.imp().playlist_view.begin_loading();

        // Begin the trace
        let now = Instant::now();

        // Turn the list of files into songs one at a time into the main loop
        let n_files = queue.len() as u32;

        let mut files = queue.into_iter();
        let mut songs = Vec::new();
        let mut cur_file: u32 = 0;

        glib::idle_add_local(
            clone!(@weak self as win => @default-return glib::Continue(false), move || {
                files.next()
                    .map(|f| {
                        win.imp().playlist_view.update_loading(cur_file, n_files);
                        match Song::from_uri(f.uri().as_str()) {
                            Ok(s) => {
                                songs.push(s);
                                cur_file += 1;
                            }
                            Err(_) => ()
                        }
                    })
                    .map(|_| glib::Continue(true))
                    .unwrap_or_else(|| {
                        debug!("Total loading time for {} files: {} ms", n_files, now.elapsed().as_millis());

                        // Re-enable the actions
                        win.action_set_enabled("queue.add-song", true);
                        win.action_set_enabled("queue.add-folder", true);
                        win.action_set_enabled("queue.clear", true);

                        if songs.is_empty() {
                            win.add_toast(i18n("No songs found"));
                        } else {
                            let player = win.player();
                            let queue =  player.queue();
                            let was_empty = queue.is_empty();

                            win.imp().playlist_view.end_loading();

                            // Bulk add to avoid hammering the UI with list model updates
                            queue.add_songs(&songs);

                            debug!("Queue was empty: {}, new size: {}", was_empty, queue.n_songs());
                            if was_empty {
                                win.player().skip_to(0);
                            }

                            // Allow jumping to the song we just added
                            if songs.len() == 1 {
                                // If we added a single song, and the queue was empty, we
                                // dispense with the pleasantries and we start playing
                                // immediately; otherwise, we let the user choose whether
                                // to jump to the newly added song
                                if was_empty {
                                    win.player().play()
                                } else {
                                    win.add_skip_to_toast(
                                        i18n("Added a new song"),
                                        i18n("Play"),
                                        queue.n_songs() - 1,
                                    );
                                }
                            } else {
                                let msg = ni18n_f(
                                    // Translators: the `{}` must be left unmodified;
                                    // it will be expanded to the number of songs added
                                    // to the playlist
                                    "Added one song",
                                    "Added {} songs",
                                    songs.len() as u32,
                                    &[&songs.len().to_string()],
                                );

                                win.add_toast(msg);
                            }
                        }

                        glib::Continue(false)
                    })
            }),
        );
    }

    fn add_files_to_queue(&self, model: &gio::ListModel) {
        let mut queue: Vec<gio::File> = vec![];

        for pos in 0..model.n_items() {
            let file = model.item(pos).unwrap().downcast::<gio::File>().unwrap();

            if let Ok(info) = file.query_info(
                "standard::name,standard::display-name,standard::type,standard::content-type",
                gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
                gio::Cancellable::NONE,
            ) {
                match info.file_type() {
                    gio::FileType::Regular => {
                        if let Some(content_type) = info.content_type() {
                            if gio::content_type_is_mime_type(&content_type, "audio/*") {
                                debug!("Adding file '{}' to the queue", file.uri());
                                queue.push(file);
                            }
                        }
                    }
                    gio::FileType::Directory => {
                        debug!("Adding folder '{}' to the queue", file.uri());
                        let files = utils::load_files_from_folder(&file, true);
                        queue.extend(files);
                    }
                    _ => (),
                }
            }
        }

        self.queue_songs(queue);
    }

    // Bind the PlayerState to the UI
    fn bind_state(&self) {
        let imp = self.imp();
        let player = self.player();
        let state = player.state();

        // Use the PlayerState:playing property to control the play/pause button
        self.update_play_button();
        state.connect_notify_local(
            Some("playing"),
            clone!(@weak self as win => move |_, _| {
                win.set_playlist_selection(false);
                win.update_play_button();
            }),
        );
        // Update the position labels
        self.update_position_labels();
        state.connect_notify_local(
            Some("position"),
            clone!(@weak self as win => move |_, _| {
                win.update_position_labels();
            }),
        );
        // Update the UI
        self.update_song();
        state.connect_notify_local(
            Some("song"),
            clone!(@weak self as win => move |_, _| {
                win.update_song();
            }),
        );
        // Update the cover, if any is available
        self.update_cover();
        state.connect_notify_local(
            Some("cover"),
            clone!(@weak self as win => move |_, _| {
                win.update_cover();
            }),
        );
        // Bind the song properties to the UI
        state
            .bind_property("title", &imp.song_details.get().title_label(), "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        state
            .bind_property("artist", &imp.song_details.get().artist_label(), "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        state
            .bind_property("album", &imp.song_details.get().album_label(), "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        state
            .bind_property(
                "volume",
                &imp.playback_control.get().volume_control(),
                "volume",
            )
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
    }

    // Bind the Queue to the UI
    fn bind_queue(&self) {
        let player = self.player();
        let queue = player.queue();

        queue.connect_notify_local(
            Some("n-songs"),
            clone!(@weak self as win => move |queue, _| {
                debug!("queue.n_songs() = {}", queue.n_songs());
                if queue.is_empty() {
                    win.set_initial_state();
                    win.reset_queue();
                } else {
                    win.action_set_enabled("queue.toggle", true);
                    win.action_set_enabled("queue.shuffle", queue.n_songs() > 1);

                    win.action_set_enabled("win.play", true);
                    win.action_set_enabled("win.previous", true);
                    win.action_set_enabled("win.next", queue.n_songs() > 1);
                }

                if queue.n_songs() == 1 {
                    win.player().skip_next();
                }

                win.update_playlist_time();
            }),
        );
        queue.connect_notify_local(
            Some("repeat-mode"),
            clone!(@weak self as win => move |queue, _| {
                win.imp().playback_control.set_repeat_mode(queue.repeat_mode());
            }),
        );
        queue.connect_notify_local(
            Some("current"),
            clone!(@weak self as win => move |queue, _| {
                if queue.is_last_song() {
                    win.action_set_enabled("win.next", false);
                } else {
                    win.action_set_enabled("win.next", true);
                }
            }),
        );
    }

    fn connect_signals(&self) {
        self.imp().queue_revealer.connect_notify_local(
            Some("folded"),
            clone!(@weak self as win => move |flap, _| {
                win.set_playlist_visible(flap.reveals_flap());
                if flap.is_folded() {
                    win.imp().playlist_view.back_button().set_visible(win.playlist_visible());
                } else {
                    win.imp().playlist_view.back_button().set_visible(false);
                }
            }),
        );

        self.imp().queue_revealer.connect_notify_local(
            Some("reveal-flap"),
            clone!(@weak self as win => move |flap, _| {
                win.set_playlist_visible(flap.reveals_flap());
                if flap.is_folded() {
                    win.imp().playlist_view.back_button().set_visible(win.playlist_visible());
                } else {
                    win.imp().playlist_view.back_button().set_visible(false);
                }
            }),
        );

        let volume_control = self.imp().playback_control.volume_control();
        volume_control.connect_notify_local(
            Some("volume"),
            clone!(@weak self as win => move |control, _| {
                win.player().set_volume(control.volume());
            }),
        );

        let waveform_view = self.imp().playback_control.waveform_view();
        waveform_view.connect_closure(
            "position-changed",
            false,
            closure_local!(@watch self as win => move |_wv: WaveformView, position: f64| {
                debug!("New position: {}", position);
                let player = win.player();
                let state = player.state();
                if state.current_song().is_some() {
                    win.player().seek_position_rel(position);
                    win.player().play();
                }
            }),
        );

        self.imp()
            .playlist_view
            .queue_select_all_button()
            .connect_clicked(clone!(@weak self as win => move |_| {
                if win.playlist_search() {
                    if let Some(ref model) = *win.imp().playlist_filtermodel.borrow() {
                        for idx in 0..model.n_items() {
                            let item = model.item(idx).unwrap();
                            let song = item.downcast_ref::<Song>().unwrap();
                            song.set_selected(true);
                        }
                    }
                } else {
                    let player = win.player();
                    let queue = player.queue();
                    for idx in 0..queue.n_songs() {
                        let song = queue.song_at(idx).unwrap();
                        song.set_selected(true);
                    }
                }
            }));

        self.imp()
            .playlist_view
            .queue_remove_button()
            .connect_clicked(clone!(@weak self as win => move |_| {
                let player = win.player();
                let queue = player.queue();
                let mut remove_songs: Vec<Song> = Vec::new();
                // Collect all songs to be removed first, since we can't
                // remove objects from the model while we're iterating it
                for idx in 0..queue.n_songs() {
                    let song = queue.song_at(idx).unwrap();
                    if song.selected() {
                        remove_songs.push(song);
                    }
                }

                for song in remove_songs {
                    win.remove_song(&song);
                }
            }));

        self.imp()
            .playlist_view
            .playlist_searchbar()
            .connect_notify_local(
                Some("search-mode-enabled"),
                clone!(@weak self as win => move |searchbar, _| {
                    win.set_playlist_search(searchbar.is_search_mode());
                }),
            );

        self.imp().settings.connect_changed(
            Some("enable-recoloring"),
            clone!(@weak self as this => move |settings, _| {
                debug!("GSettings:enable-recoloring: {}", settings.boolean("enable-recoloring"));
                let player = this.player();
                let state = player.state();
                this.update_style(state.current_song().as_ref());
            }),
        );
        let _dummy = self.imp().settings.boolean("enable-recoloring");

        self.connect_close_request(move |window| {
            debug!("Saving window state");
            let width = window.default_size().0;
            let height = window.default_size().1;

            let settings = utils::settings_manager();
            settings
                .set_int("window-width", width)
                .expect("Unable to store window-width");
            settings
                .set_int("window-height", height)
                .expect("Unable to stop window-height");

            glib::signal::Inhibit(false)
        });

        self.imp()
            .playlist_view
            .playlist_searchbar()
            .set_key_capture_widget(Some(self.upcast_ref::<gtk::Widget>()));
    }

    // The initial state of the playback actions
    fn set_initial_state(&self) {
        let player = self.player();

        let queue = player.queue();
        self.action_set_enabled("win.play", !queue.is_empty());
        self.action_set_enabled("win.previous", !queue.is_empty());
        self.action_set_enabled("win.next", !queue.is_last_song());

        self.action_set_enabled("queue.toggle", !queue.is_empty());
        self.action_set_enabled("queue.shuffle", queue.n_songs() > 1);
        self.action_set_enabled("win.replaygain", self.player().replaygain_available());

        let replaygain = self.imp().settings.enum_("replay-gain").into();
        self.set_replaygain(replaygain);

        // Manually set player state, because set_replaygain
        // only updates player state when the value changes.
        self.player().set_replaygain(replaygain);

        self.imp()
            .playback_control
            .set_repeat_mode(queue.repeat_mode());
        self.set_playlist_shuffled(queue.is_shuffled());

        // Manually update the icon on the initial empty state
        // to avoid generating the UI definition file at build
        // time
        self.imp().status_page.set_icon_name(Some(APPLICATION_ID));

        let state = player.state();
        if state.playing() || !queue.is_empty() {
            self.switch_mode(WindowMode::MainView);
        } else {
            self.switch_mode(WindowMode::InitialView);
        }
    }

    fn setup_playlist(&self) {
        let imp = self.imp();

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(clone!(@weak self as win => move |_, list_item| {
            let row = QueueRow::default();
            list_item.set_child(Some(&row));

            row.connect_notify_local(
                Some("selected"),
                clone!(@weak win => move |_, _| {
                    win.update_selected_count();
                }),
            );

            win
                .bind_property("playlist-selection", &row, "selection-mode")
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build();

            list_item
                .bind_property("item", &row, "song")
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build();

            list_item
                .property_expression("item")
                .chain_property::<Song>("artist")
                .bind(&row, "song-artist", gtk::Widget::NONE);
            list_item
                .property_expression("item")
                .chain_property::<Song>("title")
                .bind(&row, "song-title", gtk::Widget::NONE);
            list_item
                .property_expression("item")
                .chain_property::<Song>("cover")
                .bind(&row, "song-cover", gtk::Widget::NONE);
            list_item
                .property_expression("item")
                .chain_property::<Song>("playing")
                .bind(&row, "playing", gtk::Widget::NONE);
            list_item
                .property_expression("item")
                .chain_property::<Song>("selected")
                .bind(&row, "selected", gtk::Widget::NONE);
        }));
        imp.playlist_view
            .queue_view()
            .set_factory(Some(&factory.upcast::<gtk::ListItemFactory>()));

        let player = self.player();
        let queue = player.queue();

        let filter = FuzzyFilter::new();
        let filter_model = gtk::FilterListModel::new(Some(queue.model()), Some(&filter));
        let sorter = FuzzySorter::new();
        let sorter_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));
        let selection = gtk::NoSelection::new(Some(&sorter_model));
        imp.playlist_view
            .queue_view()
            .set_model(Some(selection.upcast_ref::<gtk::SelectionModel>()));
        imp.playlist_view.queue_view().connect_activate(
            clone!(@weak self as win, @weak selection => move |_, pos| {
                let song = selection
                    .upcast::<gio::ListModel>()
                    .item(pos)
                    .unwrap()
                    .downcast::<Song>()
                    .unwrap();

                let player = win.player();
                let queue = player.queue();

                let mut real_pos = None;
                for i in 0..queue.model().n_items() {
                    if let Some(item) = queue.model().item(i) {
                        let s = item.downcast::<Song>().unwrap();
                        if s.equals(&song) {
                            real_pos = Some(i);
                            break;
                        }
                    }
                }

                if let Some(real_pos) = real_pos {
                    if win.playlist_selection() {
                        queue.select_song_at(real_pos);
                    } else if queue.current_song_index() != Some(real_pos) {
                        win.player().skip_to(real_pos);
                        win.player().play();
                    } else if !win.player().state().playing() {
                        win.player().play();
                    }
                }
            }),
        );

        imp.playlist_filtermodel
            .replace(Some(sorter_model.upcast::<gio::ListModel>()));

        imp.playlist_view
            .playlist_searchentry()
            .bind_property("text", &filter, "search")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();

        imp.playlist_view
            .playlist_searchentry()
            .bind_property("text", &sorter, "search")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
    }

    fn setup_drop_target(&self) {
        let drop_target = gtk::DropTarget::builder()
            .name("file-drop-target")
            .actions(gdk::DragAction::COPY)
            .formats(&gdk::ContentFormats::for_type(gdk::FileList::static_type()))
            .build();

        drop_target.connect_drop(
            clone!(@weak self as win => @default-return false, move |_, value, _, _| {
                if let Ok(file_list) = value.get::<gdk::FileList>() {
                    if file_list.files().is_empty() {
                        win.add_toast(i18n("Unable to access dropped files"));
                        return false;
                    }

                    let model = gio::ListStore::new(gio::File::static_type());
                    for f in file_list.files() {
                        model.append(&f);
                    }
                    win.add_files_to_queue(model.upcast_ref::<gio::ListModel>());
                    return true;
                }

                false
            }),
        );

        self.imp().drag_overlay.set_drop_target(&drop_target);
    }

    fn update_play_button(&self) {
        let player = self.player();
        let state = player.state();
        let play_button = self.imp().playback_control.play_button();
        if state.playing() {
            play_button.set_icon_name("media-playback-pause-symbolic");
        } else {
            play_button.set_icon_name("media-playback-start-symbolic");
        }
    }

    fn update_position_labels(&self) {
        let imp = self.imp();
        let player = self.player();
        let state = player.state();
        if state.current_song().is_some() {
            let elapsed = state.position();
            let duration = state.duration();
            let remaining = duration.checked_sub(elapsed).unwrap_or_default();
            imp.playback_control.set_elapsed(Some(elapsed));
            imp.playback_control.set_remaining(Some(remaining));

            let position = state.position() as f64 / state.duration() as f64;
            imp.playback_control.set_position(position);
        } else {
            imp.playback_control.set_elapsed(None);
            imp.playback_control.set_remaining(None);
            imp.playback_control.set_position(0.0);
        }
    }

    fn update_song(&self) {
        let player = self.player();
        let state = player.state();
        self.scroll_playlist_to_song();
        self.update_playlist_time();
        self.update_title(state.current_song().as_ref());
        self.update_waveform(state.current_song().as_ref());
        self.update_style(state.current_song().as_ref());
    }

    fn update_cover(&self) {
        let player = self.player();
        let state = player.state();
        let song_details = self.imp().song_details.get();
        if let Some(cover) = state.cover() {
            song_details.album_image().set_cover(Some(&cover));
            song_details.show_cover_image(true);
        } else {
            song_details.album_image().set_cover(None);
            song_details.show_cover_image(false);
        }
    }

    fn update_playlist_time(&self) {
        let player = self.player();
        let queue = player.queue();
        let n_songs = queue.n_songs();
        if let Some(current) = queue.current_song_index() {
            let mut remaining_time = 0;
            for pos in 0..n_songs {
                let song = queue.song_at(pos).unwrap();
                if pos >= current {
                    remaining_time += song.duration();
                }
            }

            let remaining_min = ((remaining_time - (remaining_time % 60)) / 60) as u32;
            let remaining_hrs = ((remaining_min - (remaining_min % 60)) / 60) as u32;

            let remaining_str = if remaining_hrs > 0 {
                // Translators: the `{}` must be left unmodified, and
                // it will be replaced by the number of minutes remaining
                // in the string "N hours M minutes remaining"
                let minutes = ni18n_f(
                    "{} minute",
                    "{} minutes",
                    remaining_min % 60,
                    &[&(remaining_min % 60).to_string()],
                );

                // Translators: `{hours}` and `{minutes}` must be left
                // unmodified, and they will be replaced by number of
                // hours and by the translated number of minutes
                // remaining, respectively
                ni18n_k(
                    "{hours} hour {minutes} remaining",
                    "{hours} hours {minutes} remaining",
                    remaining_hrs,
                    &[("hours", &remaining_hrs.to_string()), ("minutes", &minutes)],
                )
            } else {
                ni18n_f(
                    // Translators: the '{}' must be left unmodified, and
                    // it will be replaced by the number of minutes remaining
                    // in the playlist
                    "{} minute remaining",
                    "{} minutes remaining",
                    remaining_min,
                    &[&remaining_min.to_string()],
                )
            };

            self.imp()
                .playlist_view
                .queue_length_label()
                .set_label(&remaining_str);
            self.imp().playlist_view.queue_length_label().show();
        } else {
            self.imp().playlist_view.queue_length_label().hide();
        }
    }

    fn scroll_playlist_to_song(&self) {
        let queue_view = self.imp().playlist_view.queue_view();
        if let Some(current_idx) = self.player().queue().current_song_index() {
            debug!("Scrolling playlist to {}", current_idx);
            queue_view
                .upcast_ref::<gtk::Widget>()
                .activate_action("list.scroll-to-item", Some(&current_idx.to_variant()))
                .expect("Failed to activate action");
        }
    }

    fn setup_provider(&self) {
        let imp = self.imp();
        if let Some(display) = gdk::Display::default() {
            gtk::StyleContext::add_provider_for_display(&display, &imp.provider, 400);
        }
    }

    fn update_style(&self, song: Option<&Song>) {
        let imp = self.imp();

        if !imp.settings.boolean("enable-recoloring") {
            imp.provider.load_from_data(&[]);
            imp.main_stack.remove_css_class("main-window");
            return;
        }

        if let Some(song) = song {
            if let Some(bg_colors) = song.cover_palette() {
                let mut css = String::new();

                let n_colors = bg_colors.len();
                for (i, color) in bg_colors.iter().enumerate().take(n_colors) {
                    let s = format!("@define-color background_color_{} {};", i, color);
                    css.push_str(&s);
                }

                for i in n_colors - 1..5 {
                    css.push_str(&format!(
                        "@define-color background_color_{} @window_bg_color;",
                        i
                    ));
                }

                // We compute the complementary of the dominant color in the palette; then we
                // try to find the closest color in the palette that we can use
                let complementary = utils::complementary_color(&bg_colors[0]);
                let mut near_color: Option<gdk::RGBA> = None;
                let mut min_near: f32 = f32::MAX;
                for color in bg_colors {
                    let delta_e = utils::color_distance(&color, &complementary);
                    if delta_e < min_near {
                        min_near = delta_e;
                        near_color = Some(color);
                    }
                }

                if let Some(near_color) = near_color {
                    css.push_str(&format!(
                        "@define-color complementary_color {};",
                        near_color
                    ));
                } else {
                    css.push_str(&format!(
                        "@define-color complementary_color {};",
                        complementary
                    ));
                }

                imp.provider.load_from_data(css.as_bytes());
                if !imp.main_stack.has_css_class("main-window") {
                    imp.main_stack.add_css_class("main-window");
                }

                self.action_set_enabled("win.enable-recoloring", true);

                return;
            }
        }

        imp.provider.load_from_data(&[]);
        imp.main_stack.remove_css_class("main-window");
        self.action_set_enabled("win.enable-recoloring", false);
    }

    fn update_waveform(&self, song: Option<&Song>) {
        let imp = self.imp();

        // Reset the widget
        imp.playback_control.waveform_view().set_peaks(None);

        if let Some(song) = song {
            imp.waveform.set_song(Some(song.clone()));
        } else {
            imp.waveform.set_song(None);
        }
    }

    fn update_title(&self, song: Option<&Song>) {
        if let Some(song) = song {
            self.set_title(Some(&format!("{} - {}", song.artist(), song.title())));
        } else {
            self.set_title(Some("Amberol"));
        }
    }

    fn update_selected_count(&self) {
        let player = self.player();
        let queue = player.queue();
        let n_selected = queue.n_selected_songs();

        let selected_str = if n_selected == 0 {
            i18n("No song selected")
        } else {
            ni18n_f(
                // Translators: The '{}' must be left unmodified, and
                // it is expanded to the number of songs selected
                "{} song selected",
                "{} songs selected",
                n_selected,
                &[&n_selected.to_string()],
            )
        };

        self.imp()
            .playlist_view
            .queue_selected_label()
            .set_label(&selected_str);
    }

    pub fn open_files(&self, files: &[gio::File]) {
        if files.is_empty() {
            self.add_toast(i18n("Unable to access files"));
            return;
        }

        let model = gio::ListStore::new(gio::File::static_type());
        for f in files {
            model.append(f);
        }
        self.add_files_to_queue(model.upcast_ref::<gio::ListModel>());
    }

    pub fn remove_song(&self, song: &Song) {
        self.player().remove_song(song);
        self.update_selected_count();
        self.update_playlist_time();
    }

    pub fn add_toast(&self, msg: String) {
        let toast = adw::Toast::new(&msg);
        self.imp().toast_overlay.add_toast(&toast);
    }

    pub fn add_skip_to_toast(&self, msg: String, button: String, pos: u32) {
        let toast = adw::Toast::new(&msg);
        toast.set_button_label(Some(&button));
        toast.set_action_name(Some("win.skip-to"));
        toast.set_action_target_value(Some(&pos.to_variant()));
        self.imp().toast_overlay.add_toast(&toast);
    }

    fn copy_song(&self) {
        let player = self.player();
        let state = player.state();
        if let Some(song) = state.current_song() {
            let s = i18n_k(
                // Translators: `{title}` and `{artist}` must be left
                // untranslated; they will expand to the title and
                // artist of the currently playing song, respectively
                "Currently playing “{title}” by “{artist}”",
                &[("title", &song.title()), ("artist", &song.artist())],
            );
            self.clipboard().set_text(&s);
        }
    }

    pub fn switch_mode(&self, mode: WindowMode) {
        let stack = self.imp().main_stack.get();
        match mode {
            WindowMode::InitialView => {
                stack.set_visible_child_name("initial-view");
                self.set_default_widget(Some(&self.imp().add_folder_button.get()));
            }
            WindowMode::MainView => {
                stack.set_visible_child_name("main-view");
                self.set_default_widget(Some(&self.imp().playback_control.play_button()));
            }
        };
    }

    pub fn set_replaygain(&self, replaygain: ReplayGainMode) {
        let imp = self.imp();

        if replaygain != imp.replaygain_mode.replace(replaygain) {
            self.player().set_replaygain(replaygain);
            self.imp()
                .settings
                .set_enum("replay-gain", replaygain.into())
                .expect("Unable to store setting");

            self.notify("replaygain-mode");
        }
    }

    pub fn replaygain(&self) -> ReplayGainMode {
        self.imp().replaygain_mode.get()
    }

    #[cfg(target_os = "linux")]
    async fn portal_request_background(&self) {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;

        match background::request(
            &identifier,
            &i18n("Amberol needs to run in the background to play music"),
            false,
            None::<&[&str]>,
            true,
        )
        .await
        {
            Ok(response) => {
                debug!("Background request successful: {:?}", response);
                self.application().unwrap().hold()
            }
            Err(err) => {
                warn!("Background request denied: {}", err);
                self.add_toast(i18n("Amberol cannot run in the background"));
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn request_background(&self) {
        let ctx = glib::MainContext::default();
        ctx.spawn_local(clone!(@weak self as win => async move {
            win.portal_request_background().await
        }));
    }
}
