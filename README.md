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
    - [ ] Create the downloading website with system detection and view of all extensions (using the `pointy-extensions` json file)
    - [ ] Create a template repo containing a publish action which builds for all available plattforms
- [x] Add settings with: editing shortcut and autostart option
- [x] Add AppState for AppConfig an AppData Path
- [x] Add initial app config by reading config to app setup and State
- [ ] Create icon
- [x] Add updater
- [ ] Only Linux: Make it work, then test hidden window
- [ ] Only macos: Focus back last focused window on the hiding of pointy's selector
- [x] Only Windows: Currently crashing on startup, make it work
- [x] Add icon mind. hold duration -> no unwanted selections (through a hover outline filling thingy)
