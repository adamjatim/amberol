// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

// Audio playback is slightly more complicated than simple UI handling code.
//
// We want to bind the state of the UI to the state of the player through
// well established patterns of property bindings and signal emissions, but
// the player can be updated from different threads and by different contexts
// which has an effect on the types that are in play. Of course, we can work
// around it by telling GStreamer that we want to receive signals in the
// default main context, but it's always largely iffy to do so as it requires
// working against the type system through the use of wrappers like Rc and
// Fragile.

// To avoid this mess, the best practice is to use message passing.
//
// In this particular case, the design of the audio playback interface is
// split into the following components:
//
// AudioPlayer: the main object managing the audio playback
// ├── PlayerState: the state tracker GObject used by the UI
// ├── Queue: the playlist tracker GListModel
// ├── GstBackend: a GstPlayer wrapper
// ╰── controllers: external bits of code that interact with the state
//     ╰── MprisController: an MPRIS wrapper
//
// The AudioPlayer object creates a glib::Sender/Receiver channel pair, and
// passes the sender to the controllers; whenever the controllers update their
// state, they will use the glib::Sender to notify the AudioPlayer, which will
// update the PlayerState.
//
// The GstBackend uses a similar sender/receiver pair to communicate with the
// AudioPlayer whenever the GStreamer state changes.
//
// The UI side connects to the PlayerState object for state tracking; all
// changes to the state object happen in the main context by design.
//
// Playback actions are proxied to the AudioPlayer object from the controllers.

mod controller;
pub use controller::Controller;

mod cover_cache;
pub use cover_cache::CoverCache;

mod inhibit_controller;
mod mpris_controller;
pub use inhibit_controller::InhibitController;
pub use mpris_controller::MprisController;

mod gst_backend;
pub use gst_backend::GstBackend;

mod player;
mod queue;
mod shuffle;
mod song;
mod state;
mod waveform_generator;

pub use player::{
    AudioPlayer, PlaybackAction, PlaybackState, RepeatMode, ReplayGainMode, SeekDirection,
};
pub use queue::Queue;
pub use shuffle::ShuffleListModel;
pub use song::Song;
pub use state::PlayerState;
pub use waveform_generator::WaveformGenerator;
