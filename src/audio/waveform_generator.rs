// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;

use glib::clone;
use gst::prelude::*;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use log::{debug, warn};

use crate::audio::{Controller, PlaybackState, RepeatMode, Song};

mod imp {
    use glib::{ParamSpec, ParamSpecBoolean, Value};
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, Default)]
    pub struct WaveformGenerator {
        pub song: RefCell<Option<Song>>,
        pub peaks: RefCell<Option<Vec<(f64, f64)>>>,
        pub pipeline: RefCell<Option<(gst::Element, gst::bus::BusWatchGuard)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WaveformGenerator {
        const NAME: &'static str = "WaveformGenerator";
        type Type = super::WaveformGenerator;
    }

    impl ObjectImpl for WaveformGenerator {
        fn dispose(&self) {
            if let Some((pipeline, _bus_watch)) = self.pipeline.take() {
                pipeline.send_event(gst::event::Eos::new());
                match pipeline.set_state(gst::State::Null) {
                    Ok(_) => {}
                    Err(err) => warn!("Unable to set existing pipeline to Null state: {}", err),
                }
            }
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecBoolean::builder("has-peaks").read_only().build()]);

            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "has-peaks" => self.obj().peaks().is_some().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct WaveformGenerator(ObjectSubclass<imp::WaveformGenerator>);
}

impl Default for WaveformGenerator {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl Controller for WaveformGenerator {
    fn set_playback_state(&self, _playback_state: &PlaybackState) {}

    fn set_song(&self, song: &Song) {
        self.imp().song.replace(Some(song.clone()));
        self.load_peaks();
    }

    fn set_position(&self, _position: u64, _notify: bool) {}
    fn set_repeat_mode(&self, _mode: RepeatMode) {}
}

impl WaveformGenerator {
    pub fn new() -> Self {
        WaveformGenerator::default()
    }

    pub fn peaks(&self) -> Option<Vec<(f64, f64)>> {
        (*self.imp().peaks.borrow()).as_ref().cloned()
    }

    fn save_peaks(&self) {
        if let Some(peaks) = self.peaks() {
            let song = match self.imp().song.borrow().as_ref() {
                Some(s) => s.clone(),
                None => {
                    self.notify("has-peaks");
                    return;
                }
            };

            if let Some(uuid) = song.uuid() {
                let mut cache = glib::user_cache_dir();
                cache.push("amberol");
                cache.push("waveforms");
                glib::mkdir_with_parents(&cache, 0o755);

                cache.push(format!("{}.json", uuid));

                let j = serde_json::to_string(&peaks).unwrap();
                let file = gio::File::for_path(&cache);
                file.replace_contents_async(
                    j,
                    None,
                    false,
                    gio::FileCreateFlags::NONE,
                    gio::Cancellable::NONE,
                    move |_| {
                        debug!("Waveform cached at: {:?}", &cache);
                    },
                );
            }
        }

        self.notify("has-peaks");
    }

    fn load_peaks(&self) {
        let song = match self.imp().song.borrow().as_ref() {
            Some(s) => s.clone(),
            None => return,
        };

        if let Some(uuid) = song.uuid() {
            let mut cache = glib::user_cache_dir();
            cache.push("amberol");
            cache.push("waveforms");
            cache.push(format!("{}.json", uuid));

            let file = gio::File::for_path(&cache);
            file.load_contents_async(
                gio::Cancellable::NONE,
                clone!(
                    #[strong(rename_to = this)]
                    self,
                    move |res| {
                        match res {
                            Ok((bytes, _tag)) => {
                                let p: Vec<(f64, f64)> =
                                    serde_json::from_slice(&bytes[..]).unwrap();
                                this.imp().peaks.replace(Some(p));
                                this.notify("has-peaks");
                            }
                            Err(err) => {
                                debug!("Could not read waveform cache file: {}", err);
                                this.generate_peaks();
                            }
                        }
                    }
                ),
            );
        }
    }

    fn generate_peaks(&self) {
        if let Some((pipeline, _bus_watch)) = self.imp().pipeline.take() {
            // Stop any running pipeline, and ensure that we have nothing to
            // report
            self.imp().peaks.replace(None);
            pipeline.send_event(gst::event::Eos::new());
            match pipeline.set_state(gst::State::Null) {
                Ok(_) => {}
                Err(err) => warn!("Unable to set existing pipeline to Null state: {}", err),
            }
        }

        let song = match self.imp().song.borrow().as_ref() {
            Some(s) => s.clone(),
            None => {
                self.imp().peaks.replace(None);
                self.notify("has-peaks");
                return;
            }
        };

        // Reset the peaks vector
        let peaks: Vec<(f64, f64)> = Vec::new();
        self.imp().peaks.replace(Some(peaks));

        let pipeline_str = "uridecodebin name=uridecodebin ! audioconvert ! audio/x-raw,channels=2 ! level name=level interval=250000000 ! fakesink name=faked";
        let pipeline = match gst::parse::launch(pipeline_str) {
            Ok(pipeline) => pipeline,
            Err(err) => {
                warn!("Unable to generate the waveform: {}", err);
                self.imp().peaks.replace(None);
                self.notify("has-peaks");
                return;
            }
        };

        let uridecodebin = pipeline
            .downcast_ref::<gst::Bin>()
            .unwrap()
            .by_name("uridecodebin")
            .unwrap();
        uridecodebin.set_property("uri", song.uri());

        let fakesink = pipeline
            .downcast_ref::<gst::Bin>()
            .unwrap()
            .by_name("faked")
            .unwrap();
        fakesink.set_property("qos", false);
        fakesink.set_property("sync", false);

        let bus = pipeline
            .bus()
            .expect("Pipeline without bus. Shouldn't happen!");

        debug!("Adding bus watch");
        let bus_watch = bus
            .add_watch_local(clone!(
                #[weak(rename_to = this)]
                self,
                #[weak]
                pipeline,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move |_, msg| {
                    use gst::MessageView;

                    match msg.view() {
                        MessageView::Eos(..) => {
                            debug!("End of waveform stream");
                            pipeline
                                .set_state(gst::State::Null)
                                .expect("Unable to set 'null' state");
                            // We're done
                            this.imp().pipeline.replace(None);
                            this.save_peaks();
                            return glib::ControlFlow::Break;
                        }
                        MessageView::Error(err) => {
                            warn!("Pipeline error: {:?}", err);
                            pipeline
                                .set_state(gst::State::Null)
                                .expect("Unable to set 'null' state");
                            // We're done
                            this.imp().pipeline.replace(None);
                            this.save_peaks();
                            return glib::ControlFlow::Break;
                        }
                        MessageView::Element(element) => {
                            if let Some(s) = element.structure() {
                                if s.has_name("level") {
                                    let peaks_array = s.get::<&glib::ValueArray>("peak").unwrap();
                                    let v1 = peaks_array[0].get::<f64>().unwrap();
                                    let v2 = peaks_array[1].get::<f64>().unwrap();
                                    // Normalize peaks between 0 and 1
                                    let peak1 = f64::powf(10.0, v1 / 20.0);
                                    let peak2 = f64::powf(10.0, v2 / 20.0);
                                    if let Some(ref mut peaks) = *this.imp().peaks.borrow_mut() {
                                        peaks.push((peak1, peak2));
                                    }
                                }
                            }
                        }
                        _ => (),
                    };

                    glib::ControlFlow::Continue
                }
            ))
            .expect("failed to add bus watch");

        match pipeline.set_state(gst::State::Playing) {
            Ok(_) => {
                self.imp().pipeline.replace(Some((pipeline, bus_watch)));
            }
            Err(err) => {
                warn!("Unable to generate the waveform: {}", err);
                pipeline
                    .set_state(gst::State::Null)
                    .expect("Pipeline reset failed");
                self.imp().peaks.replace(None);
                self.notify("has-peaks");
            }
        };
    }
}
