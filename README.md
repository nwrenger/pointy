# pointy

pointy is a very useful clipboard extension everybody needs. It's currently being build and in no usable state!

## Current Todos

- [ ] Extension System
  - [x] General implementation
  - [x] Add Disabling/Enabling Extensions
  - [x] UI (as far as I want it to be)
  - [x] Add Versioning
  - [x] Auto updating of already downloaded extensions
    - [x] Add publish action for in-house extensions
  - [ ] Online Hosting on Github (or something similar) with linking of setting option "Download Extensions" and good ui for easy download
    - [ ] Create for that a `pointy-extensions` repo with a big json file containing all extensions and the ability to add new ones via a bot using gh issues
    - [ ] Implement downloading in the settings menu with view of all extensions
      - [x] frontend design
      - [ ] using the `pointy-extensions` json file
      - [ ] backend && working functions
    - [ ] Create a template repo containing a publish action which builds for all available plattforms
- [ ] Create project website (for downloading and extension docs)
- [x] Add settings with: editing shortcut and autostart option
- [x] Add AppState for Config an AppData Path
- [x] Add initial app config by reading config to app setup and State
- [ ] Create icon
- [x] Add updater
  - [ ] Persist permissions on macos after an update
- [ ] Only Linux: Make it work, then test hidden window
  - need to install `libappindicator` and app indicator support (like on gnome with `gnome-shell-extension-appindicator`)
  - more i don't know
- [ ] Only macos (maybe): Focus back last focused window on the hiding of pointy's selector
- [x] Only Windows: Currently crashing on startup, make it work
- [x] Add icon mind. hold duration -> no unwanted selections (through a hover outline filling thingy)
