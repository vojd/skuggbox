# UI Backend

Responsible for how egui draws on a GL context.

The code in this repo is a version of [egui glow](https://github.com/emilk/egui/tree/master/crates/egui_glow/)

To use on linux, run

```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev
```

All credit to [emilk](https://github.com/emilk/) for making this copy/pasteable :D 


The UI code in this directory is only intended to be the actual integration to Egui.
The application specific UI code does not live in here but rather in the ui.rs mod.
