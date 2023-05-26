use crate::{Action, AppState, PlayMode, ShaderService};

pub fn top_bar(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    actions: &mut Vec<Action>,
    shader_service: &ShaderService,
) {
    ui.horizontal(|ui| {
        // show the current time / beat
        // TODO: Change "time" to "beat" when we can switch timing mode
        let time = format!("time: {:6.2}", app_state.playback_time);
        ui.label(time);

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

        ui.spacing();

        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
            ui.color_edit_button_rgb(&mut app_state.scene_vars.color_a);
        });
    });
}
