# Changes

All notable changes to this project will be documented in this file.

## [2024.1] - 2024-09-21

### Added

- Add a marquee label for song details [!143]
- Add more accessibility labels and descriptions
- Add Romanian translation
- Add Burmese translation
- Add Norwegian Bokmål translation
- Add Hindi translation

### Changed

- Adjust whitespace for DnD overlay [!153]
- Port to the new libadwaita widgets [!138]
- Update dependencies [!144, !155, !160]
- Translation updates

### Fixed

- Support higher scaling factors in the cover image [#344]
- Update application metadata [!145, !157, !164]

### Removed

## [0.10.3] - 2023-05-23

### Changed

- Translation updates
  - Hebrew
  - Polish
  - Finnish
  - Portuguese
  - Russian
  - German

### Fixed

- Resize the waveform view when interpolating states
- Fix cover art scaling [#324]
- Fix deprecation warnings at build time

## [0.10.2] - 2023-04-28

### Changed

- Use a better icon for the drop overlay [!135]
- Translation updates
  - Galician
  - Slovenian
  - Hungarian
  - Swedish
  - Turkish
  - Indonesian
  - French
  - Occitan

### Fixed

- Crash when adjusting volume while in background [#317]
- Crash when adding folder after sending app to the background [#319]

## [0.10.1] - 2023-04-25

### Fixed

- Background playback doesn't work [#314]

## [0.10.0] - 2023-04-24

### Added

- Allow restoring the playlist from the previous session [!120]
- Allow disabling background playback
- Add control to quickly muting/unmuting the audio
- Enable ashpd on FreeBSD [!130]
- New translations
  - Belarusian
  - French
  - Korean
  - Friulan
  - Czech

### Changed

- Update the dependency to the gtk-rs bindings
- Update the dependency to ashpd
- Update the dependency to lofty
- Use consistent labelling for adding single songs [#270]
- Use consistent terms for songs and playlist [#301]
- Improve rendering of cover art [#310]
- Separate the volume bar from the waveform display
- Translation updates
  - Ukrainian
  - German
  - Slovenian
  - Portuguese
  - Brazilian Portuguese
  - Georgian
  - Russian
  - Croatian
  - Danish
  - Turkish
  - Swedish
  - Hebrew
  - Polish
  - Indonesian
  - Occitan
  - Lithuanian
  - Occitan
  - Galician
  - Finnish
  - Basque
  - Korean
  - Persian
  - Spanish
  - Hungarian
  - Serbian

### Fixed

- Reset the playlist position when searching [!117]
- Use HIG-compliant capitalization in the shortcuts view [!115]
- Narrow playlist view [#230]
- Do not add duplicate songs to the playlist [#248]
- Update next track button state depending on repeat mode [!114]
- Fix styling of the search bar [#266]
- Stop waveform pipeline when dropping the generator [!107]
- Change the ReplayGain menu translatable context [#281]
- Always draw last bar in waveform view [!127]
- Fix waveform overdrawing in RTL layout [!128]
- Fix waveform overdrawing [!124]
- Improve the readability of the drop overlay [#288]

### Removed

## [0.9.2] - 2022-12-10

### Added

- Allow continuous seeking on the waveform widget [#99]
- New translations
  - Icelanding
  - Hungarian
  - British English
  - Greek

### Changed

- Use new about window from libadwaita
- Update build to use Cargo directly without a wrapper
- Update the dependency on lofty to 0.9.0
- Translation updates
  - Finnish
  - Persian
  - Portuguese
  - Turkish
  - Chinese (China)
  - Italian
  - Swedish
  - Brazilian Portuguese
  - Danish
  - Occitan
  - Slovenian
  - Hebrew
  - Indonesian
  - German
  - Russian
  - Serbian
  - Basque
  - Georgian
  - Dutch

### Fixed

- Recolor the folded playlist background
- Check for more file names for external covers [#247]
- Force the direction of the primary menu
- Remove toast for failed background portal requests
- Fully animate waveform between tracks [#254]

## [0.9.1] - 2022-08-30

### Added

- Add ReplayGain support [#75]
- Show hours to the playlist time
- Support external cover art files [#14]
- Add playing indicator in selection mode [#227]
- New translations
  - Serbian

### Changed

- Stabilise the shuffling behaviour [!104, #207]
- Update the version of lofty [#216]
- Translation updates
  - Russian
  - Portuguese
  - Persian
  - Polish
  - Ukrainian
  - Basque
  - Turkish
  - Italian
  - Finnish
  - Croatian
  - Occitan

### Fixed

- Clarify the notification text for unavailable files [#215]
- Increase specificity of the cover art UUID
- Make the playlist side bar narrower [#230]
- Check for unsigned overflow [#223]
- Properly mark playlist remaining time for translation [#225]

### Removed

- Remove the "sound" keyword from the desktop file [#224]

## [0.9.0] - 2022-08-05

### Added

- Add key shortcut for toggling playlist shuffle [!94]
- Add fuzzy search to the playlist [!96]
- New translations
  - Catalan
  - Lithuanian
  - Georgian
  - Italian

### Changed

- Only list audio MIME types supported by lofty
- Update the version of lofty we depend on
- Translation updates
  - Swedish
  - Portuguese
  - Basque
  - Russian
  - Polish
  - Persian
  - Ukrainian
  - Dutch
  - German
  - Indonesian
  - Chinese (China)
  - Hebrew
  - Brazilian Portuguese
  - Occitan
  - Finnish

### Fixed

- Add the directory MIME type to the list of supported typs [#202]
- Explicitly remove action bar background [!95]
- Make sure to maintain aspect ratio of cover art pixbuf [#208]

### Removed

- Remove MIME type associations for unsupported file types

## [0.8.1] - 2022-06-24

### Added

- New translations
  - Danish

### Changed

- Use the appropriate wording and style for tooltips [!89]
- Require version 0.4.8 of the gtk4 crate [!90]
- Translation updates
  - Ukrainian
  - German
  - Hebrew
  - Russian
  - Polish
  - Swedish
  - Occitan
  - Portuguese
  - Chinese (China)

### Fixed

- Allow selecting multiple files and folders [#71]
- Fix improper MPRIS reporting when paused [#201]
- Fix panic on playback state change with no window [!91]
- Maintain the playlist panel's width [#190]
- Use the display name as the song base UUID [#198]
- Handle nested dist folders properly
- Let Amberol run in the background without a window [!83]
- Sort files like Nautilus when adding a folder [#187]

## [0.8.0] - 2022-06-17

### Added

- Implement playlist search [#178]
- Restore the window state [!79]
- Support building and running on macOS [#179]
- Add cover cache object [!74]
- Allow running Amberol in the background [!72]
- New translations
  - Finnish
  - Portuguese
  - Nepali
  
### Changed

- Flip the waveform channels [!76]
- Notify the user when drag and drop gives us no files [#175]
- Update the dependency on lofty [#172]
- Translation updates
  - Polish
  - Ukrainian
  - Swedish
  - Hebrew
  - Brazilian Portuguese
  - German
  - Basque
  - Occitan
  - Persian
  - Russian
  - Chinese (China)

### Fixed

- Reset the player state when removing its last song [#170]
- Disable queue.clear action while adding songs [#163]
- Mark file selection dialog titles for translation [#164]
- Start playing when selecting the current row
- Set min-height for the song details [#155]
- Restore queue actions once loading ends [#160]

### Removed

## [0.7.0] - 2022-06-03

### Added

- Show the current song in the window title
- Add a warning for failed cover art loading
- Show toast when adding a single song [#136]
- New translation
  - Slovenian

### Changed

- Change the currently playing song indicator [#74]
- Improve vertical spacing of playback controls [!70]
- Make the shuffle model more predictable [!67]
- Refine the app icon [!66]
- Rework waveform colors [#119]
- Tweak the cover art style in the playlist [!63, #147]
- Use a single suggested action button [#145]
- Adjust scale and progress bar styles [#146]
- Translation updates
  - German
  - Polish
  - Ukrainian
  - Swedish
  - Russian
  - Turkish
  - Hebrew
  - Dutch

### Fixed

- Do not change position when scrubbing without a song [#151]
- Fix double select on playlist end [#149]
- Notify if no files/folders were selected [#148]
- Add a check for MPRIS cover art
- Fix handling cleared queues [#138]
- Use a weak reference when loading songs [#140]
- Update playlist length when removing a single song
- Switch window mode when opening a file

## [0.6.3] - 2022-05-21

### Added

- Add translations:
  - German

### Changed

- Translation updates:
  - Swedish

### Fixed

- Fix drag and drop on the initial landing page [!62]

## [0.6.2] - 2022-05-20

### Added

- Add "copy song details to clipboard" [!53]
- Add a cache for the waveforms, to speed up loading on songs we have
  already seen [#131]
- Add accessibility information to various custom widgets
- Add translations:
  - Chinese (China)

### Changed

- Add whole folder at once [!30]
- Provide user feedback during loading [!54]
- Translation updates:
  - Swedish
  - Ukrainian
  - Occitan
  - Basque
  - Turkish
  - Polish
  - Russian
  - Persian

### Fixed

- Ensure that the remaining time sign is consistent in RTL locales [#118]
- Improve the UI consistency when clearing the playlist
- Apply darkening to the playlist view unconditionally [#128]
- Fix the playlist end state [#132]
- Fix key navigation [#130]

## [0.6.1] - 2022-05-09

### Added

- Add translations for:
  - Dutch
  - Indonesian
  - Occitan
  - Spanish

### Changed

- Make the cover art image slightly bigger
- Update translations for:
  - Ukrainian
  - Swedish
  - Polish
  - Persian

### Fixed

- Improve the appearance of the initial landing page [#106]
- Stabilise the width of the playlist panel [#110]
- Rely on gdk-pixbuf instead of lofty for image format detection [#111]
- Multiple papercut style fixes [#105, #108]
- Fix selector for playlist background when folded [#107]

## [0.6.0] - 2022-05-06

### Added

- Use a selection mode for the playlist management [#81]
- Allow disabling UI recoloring
- Expose more song state through MPRIS
- Add better error messages in the UI
- Support RTL text direction in the waveform widget
- Add translations for:
  - Russian
  - Turkish
  - Brazilian Portuguese
  - Hebrew
  - Galician
  - Swedish
  - Basque
  - Persian
  - Ukrainian
  - Polish

### Changed

- Reset to the initial state when clearing the playlist [#101]
- Tone down the recoloring to improve legibility of text and controls [#97]
- Recolor only the main window [#104]
- Use better icon for playlist toggle button [#102]
- Use the cover art palette for the waveform view accent color [#61]
- Set the minimum and maximum width for the playlist view [#93]

### Fixed

- Fix the background recoloring gradient to use the whole cover art palette
- Reset the waveform generator and view on failure [#57]
- Darken the playlist background when unfolded [#85]
- Improve the tooltips for playback controls [#69]
- Fix extra spacing in the playlist view [#98]
- Fix elapsed song time in RTL locales [#95]
- Remove missing shortcuts from the shortcuts dialog [#96]

## [0.5.0] - 2022-04-29

### Added

- Improve fallback paths for song metadata

### Changed

- Move the playlist side panel to the left of the playback controls [#50]
- Make sure that the remove button in the playlist rows is accessible
  without hovering

### Fixed

- Align the waveform to the pixel grid [#76]

### Removed

- Drop the seek buttons, and rely on the waveform control [#59]

## [0.4.3] - 2022-04-26

### Added

- Add scrolling support to the volume control [#50]

### Fixed

- Fix behaviour of the waveform with short songs and avoid overdrawing [#68]
- Make the waveform control more legible [#52]
- Reset the shuffle state when clearing the playlist [#60]
- Keep the playlist visibility, folded or unfolded, in sync with the
  toggle button that controls it [#55]
- Fix a crash when manually advancing through the playlist [#54]

## [0.4.2] - 2022-04-22

### Fixed

- Fix the fallback cover art in the playlist

## [0.4.1] - 2022-04-22

### Fixed

- Don't skip songs without a cover art [#46]
- Clean up unnecessary overrides [Bilal Elmoussaoui, !32]

## [0.4.0] - 2022-04-22

### Added

- Add waveform display and quick navigation
- Allow queueing folders recursively
- Add initial status page at startup [#27]
- Add remove button to the playlist [#40]
- Show cover art in the playlist

### Changed

- Allow adding folders via drag and drop [#17]
- Allow shuffling only when the playlist contains more than one song [#15]
- Style the popover using a similar background as the main window [#12]
- Small style tweaks for the recoloring
- Reduce the height of the full window to fit in 768p displays [#16]
- Make the layout more mobile friendly [#28]
- Ship our own icon assets

### Fixed

- Fix an assertion failure when reaching the end of a shuffled playlist
- Scroll playlist to the current song [#29]
- Update dependency on lofty for m4a support [#22]
- Add divider above scrolling playlist [#26]
- Fix styling of the missing cover fallback image [#36]
- Set the album art metadata for MPRIS [#13]

## [0.3.0] - 2022-04-15

### Added

- Allow shuffling the contents of the playlist
- Support dropping multiple files
- Volume control
- Allow Amberol to be set as the default application for Music in
  the GNOME Settings

### Changed

- Miscellaneous cleanups [Christopher Davis, !10]
- Use idiomatic Rust as suggested by Clippy
- Improve handling the end of playlist state
- Skip songs that cannot be queried for metadata
- Switch to a portrait layout

### Fixed

- Stop playback when clearing the playlist
- Immediately play the song selected from the playlist
- Use the appropriate color format for the texture data [#7]
- Use the proper fallback asset for albums with no cover
- Start playing when opening a file [#8]

## [0.2.1] - 2022-04-11

### Changed

- Style tweaks [Jakub Steiner, !9]

### Fixed

- Handle songs with unset fields without panicking

## [0.2.0] - 2022-04-11

### Added

- Inhibit system suspend when playing

### Changed

- Tweak the behaviour of the window when toggling the playlist
- Improve the style of the window [Alexander Mikhaylenko, !7]
  - Deal with margins and padding
  - Style the playlist list view
  - Style the drag overlay

## [0.1.0] - 2022-04-08

Initial alpha release for Amberol

### Added

- Basic playback
- Playlist control:
  - Add single file
  - Add folder
  - Drag and drop
- Support opening files from the CLI
- Recolor the UI using the cover art palette
