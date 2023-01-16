use crate::{Action, AppState, PlayMode, ShaderService};

pub fn top_bar(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    actions: &mut Vec<Action>,
    shader_service: &ShaderService,
) {
    ui.horizontal(|ui| {
        if ui.button("⏹").clicked() {
            actions.push(Action::TogglePlayPause);
            actions.push(Action::TimeStop);
        }
        // rewind
        if ui.button("⏪").clicked() {
            actions.push(Action::TimeRewind(1.0))
        }
        // play/pause
        let play_mode_label = match app_state.play_mode {
            PlayMode::Playing => "⏸",
            PlayMode::Paused => "▶",
        };
        if ui.button(play_mode_label).clicked() {
            actions.push(Action::TogglePlayPause)
        }

        // fast forward
        if ui.button("⏩").clicked() {
            actions.push(Action::TimeForward(1.0))
        }

        ui.spacing();
        // show camera mode
        let cam_mode_str = match shader_service.use_camera_integration {
            true => "dev cam",
            false => "shader cam",
        };
        ui.label(format!("Camera mode: {}", cam_mode_str));
    });
}

pub fn left_pane(
    ui: &mut egui::Ui,
    _app_state: &mut AppState,
    _actions: &mut [Action],
    _shader_service: &ShaderService,
) {
    ui.vertical(|ui| {
        ui.label("left pane");
    });
}
