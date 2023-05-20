use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_prototype_debug_lines::*;
use itertools::Itertools;
use portal_common::prelude::*;

const GRID_SIZE: f32 = 2.0;

#[derive(Resource, Default)]
struct EditingLevel(Level);

#[derive(Default, PartialEq, Eq)]
enum EditMode {
    #[default]
    Pan,
    Add,
    Remove,
}

#[derive(Resource, Default)]
struct EditorState {
    mode: EditMode,
    rounded_mouse: IVec2,
    bottom_height: f32,
    top_height: f32,
    points: Vec<IVec2>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(DebugLinesPlugin::default())
        .insert_resource(EditorState::default())
        .insert_resource(EditingLevel::default())
        .add_startup_system(setup)
        .add_systems((editor_ui, handle_input, draw))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.5,
            ..default()
        },
        ..default()
    });
}

fn handle_input(
    mut editor_state: ResMut<EditorState>,
    mut level: ResMut<EditingLevel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<Input<MouseButton>>,
) {
    if let Ok((camera, global_transform)) = camera_q.get_single() {
        let window = windows.single();
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(global_transform, cursor))
        {
            let rounded_pos = world_position.round().as_ivec2();

            match editor_state.mode {
                EditMode::Pan => {}
                EditMode::Add => {
                    let lean = 10.0;
                    if buttons.just_pressed(MouseButton::Right) {
                        let copied_array = editor_state.points.clone();
                        for (idx, point) in copied_array.iter().enumerate() {
                            if rounded_pos.x as f32 >= point.x as f32 - lean
                                && rounded_pos.x as f32 <= point.x as f32 + lean
                                && rounded_pos.y as f32 >= point.y as f32 - lean
                                && rounded_pos.y as f32 <= point.y as f32 + lean
                                && idx == copied_array.len() - 1
                            {
                                editor_state.points.remove(idx);
                            }
                        }
                    }
                    if buttons.just_pressed(MouseButton::Left) {
                        let len = editor_state.points.len();
                        if editor_state.points.is_empty() {
                            editor_state.points.push(rounded_pos);
                        } else if len == 1 {
                            if rounded_pos.x <= editor_state.points[0].x {
                                editor_state.points.clear();
                                return;
                            }
                            editor_state.points.push(rounded_pos);
                        } else {
                            // if let Some(last) = editor_state.points.last() {
                            let point = editor_state.points[0];
                            if rounded_pos.x as f32 >= point.x as f32 - lean
                                && rounded_pos.x as f32 <= point.x as f32 + lean
                                && rounded_pos.y as f32 >= point.y as f32 - lean
                                && rounded_pos.y as f32 <= point.y as f32 + lean
                            {
                                editor_state.points.push(point);
                                let mut sector = Sector::new(0.0, 40.0);
                                editor_state
                                    .points
                                    .iter()
                                    .tuple_windows()
                                    .for_each(|(a, b)| {
                                        let wall = Wall {
                                            points: [a.as_vec2(), b.as_vec2()],
                                            color: PixColor(255, 255, 255, 255),
                                            uv: Vec2::ONE,
                                        };
                                        sector.walls.push(wall);
                                    });
                                level.0.sectors.push(sector);
                                editor_state.points.clear();
                            } else {
                                editor_state.points.push(rounded_pos);
                            }
                            // }
                        }
                    }
                }
                EditMode::Remove => {}
            }
        }
    }
}

fn editor_ui(mut contexts: EguiContexts, mut editor_state: ResMut<EditorState>) {
    egui::SidePanel::left("Editor").show(contexts.ctx_mut(), |ui| {
        ui.label("Editor Mode");
        ui.radio_value(&mut editor_state.mode, EditMode::Pan, "Pan");
        ui.radio_value(&mut editor_state.mode, EditMode::Add, "Add");
        ui.radio_value(&mut editor_state.mode, EditMode::Remove, "Remove");
    });
}

fn draw(
    editor_state: Res<EditorState>,
    mut lines: ResMut<DebugLines>,
    level: ResMut<EditingLevel>,
) {
    for point in editor_state.points.iter() {
        draw_point(&mut lines, point.clone().as_vec2());
    }
    for sector in level.0.sectors.iter() {
        for wall in sector.walls.iter() {
            lines.line(wall.points[0].extend(0.0), wall.points[1].extend(0.0), 0.0);
        }
    }
}

fn draw_point(lines: &mut DebugLines, pos: Vec2) {
    let offset = 2.5;
    let start = pos - Vec2::new(offset, offset);
    let end = pos - Vec2::new(offset, -offset);
    let duration = 0.0; // Duration of 0 will show the line for 1 frame.
    lines.line(start.extend(0.0), end.extend(0.0), duration);
    let start = pos - Vec2::new(offset, -offset);
    let end = pos - Vec2::new(-offset, -offset);
    lines.line(start.extend(0.0), end.extend(0.0), duration);
    let start = pos - Vec2::new(-offset, -offset);
    let end = pos - Vec2::new(-offset, offset);
    lines.line(start.extend(0.0), end.extend(0.0), duration);
    let start = pos - Vec2::new(-offset, offset);
    let end = pos - Vec2::new(offset, offset);
    lines.line(start.extend(0.0), end.extend(0.0), duration);
}
