use bevy::a11y::AccessibilityPlugin;
use bevy::app::PluginGroupBuilder;
use bevy::core_pipeline::CorePipelinePlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::input::InputPlugin;
use bevy::log::LogPlugin;
use bevy::pbr::PbrPlugin;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::RenderPlugin;
use bevy::sprite::SpritePlugin;
use bevy::time::TimePlugin;
use bevy::winit::WinitPlugin;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_pixels::prelude::*;
use portal_common::prelude::*;
use rand::prelude::*;

#[derive(Component)]
struct Viewpoint;

#[derive(SystemParam)]
struct PixelHandler<'w, 's> {
    // commands: Commands<'w, 's>,
    pixel_wrapper: Query<'w, 's, &'static mut PixelsWrapper>,
    options_query: Query<'w, 's, &'static PixelsOptions>,
}

impl<'w, 's> PixelHandler<'w, 's> {
    fn clear(&mut self, color: PixColor) {
        let Ok(mut wrapper) = self.pixel_wrapper.get_single_mut() else { return };

        let frame = wrapper.pixels.frame_mut();

        frame.copy_from_slice(&[color.0, color.1, color.2, color.3].repeat(frame.len() / 4));
    }

    fn height(&self) -> u32 {
        let Ok(options) = self.options_query.get_single() else { return 0 };
        options.height
    }

    fn width(&self) -> u32 {
        let Ok(options) = self.options_query.get_single() else { return 0 };
        options.width
    }

    fn set_pixel(&mut self, position: UVec2, color: PixColor) {
        let Ok(options) = self.options_query.get_single() else { return };
        if position.x < options.width && position.y < options.height {
            let Ok(mut wrapper) = self.pixel_wrapper.get_single_mut() else { return };
            let frame_width_bytes = (options.width * 4) as usize;
            let frame: &mut [u8] = wrapper.pixels.frame_mut();
            let x_offset = (position.x * 4) as usize;
            let final_y = position.y as i32 - options.height as i32;
            let y_offset = final_y.abs() as usize * frame_width_bytes;
            let i = x_offset + y_offset;
            let j = i + 4;
            frame[i..j].copy_from_slice(&[color.0, color.1, color.2, color.3]);
        }
    }
}

pub struct BevyPlugins;

impl PluginGroup for BevyPlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        group = group
            .add(LogPlugin::default())
            .add(TaskPoolPlugin::default())
            .add(TypeRegistrationPlugin::default())
            .add(FrameCountPlugin::default())
            .add(TimePlugin::default())
            .add(TransformPlugin::default())
            .add(HierarchyPlugin::default())
            .add(DiagnosticsPlugin::default())
            .add(InputPlugin::default())
            .add(WindowPlugin::default())
            .add(AccessibilityPlugin)
            .add(WinitPlugin::default());
        group
    }
}

fn main() {
    App::new()
        .add_plugins(BevyPlugins)
        .add_plugin(PixelsPlugin::default())
        .add_startup_system(setup_pixel_options)
        .add_startup_system(setup)
        .add_system(move_player)
        .add_system(clear.in_set(PixelsSet::Draw))
        .add_system(draw.in_set(PixelsSet::Draw))
        .run();
}

fn setup_pixel_options(mut options_query: Query<&mut PixelsOptions>) {
    let Ok(mut options) = options_query.get_single_mut() else { return };

    *options = PixelsOptions {
        width: 160,
        height: 120,
        // scale_factor: f32,
        auto_resize_buffer: false,
        ..default()
    };
}

fn clear(mut pixel_handler: PixelHandler) {
    pixel_handler.clear(PixColor(0, 0, 0, 255));
}

fn setup(mut commands: Commands) {
    commands.spawn((Transform::from_xyz(70.0, 20.0, -110.0), Viewpoint));
    let mut level = Level::default();
    let mut sector = Sector::new(0.0, 10.0);
    sector.add_wall(
        Vec2::new(0.0, 25.0),
        Vec2::new(0.0, 0.0),
        PixColor(128, 128, 128, 255),
    );
    sector.add_wall(
        Vec2::new(25.0, 25.0),
        Vec2::new(0.0, 25.0),
        PixColor(100, 100, 100, 255),
    );
    sector.add_wall(
        Vec2::new(25.0, 0.0),
        Vec2::new(25.0, 25.0),
        PixColor(128, 128, 128, 255),
    );
    sector.add_wall(
        Vec2::new(0.0, 0.0),
        Vec2::new(25.0, 0.0),
        PixColor(100, 100, 100, 255),
    );
    level.sectors.push(sector);
    let mut sector = Sector::new(10.0, 40.0);
    sector.add_wall(
        Vec2::new(30.0, 50.0),
        Vec2::new(30.0, 30.0),
        PixColor(200, 0, 0, 255),
    );
    sector.add_wall(
        Vec2::new(50.0, 50.0),
        Vec2::new(30.0, 50.0),
        PixColor(255, 0, 0, 255),
    );
    sector.add_wall(
        Vec2::new(50.0, 30.0),
        Vec2::new(50.0, 50.0),
        PixColor(200, 0, 0, 255),
    );
    sector.add_wall(
        Vec2::new(30.0, 30.0),
        Vec2::new(50.0, 30.0),
        PixColor(255, 0, 0, 255),
    );
    level.sectors.push(sector);
    commands.spawn(level);
}

fn move_player(
    mut player_query: Query<&mut Transform, With<Viewpoint>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let (angle_up, angle, _) = transform.rotation.to_euler(EulerRot::XYZ);
        let dt = time.delta_seconds();
        let forward = transform.forward();
        let right = transform.left();
        let speed = 50.0;
        let rotation_speed = 25.0;
        if keys.pressed(KeyCode::W) {
            transform.translation -= forward * dt * speed;
        }
        if keys.pressed(KeyCode::S) {
            transform.translation += forward * dt * speed;
        }
        if keys.pressed(KeyCode::D) {
            transform.translation -= right * dt * speed;
        }
        if keys.pressed(KeyCode::A) {
            transform.translation += right * dt * speed;
        }
        if keys.pressed(KeyCode::Space) {
            transform.translation.y += 4.0 * dt * speed;
        }

        if keys.pressed(KeyCode::C) {
            transform.translation.y -= 4.0 * dt * speed;
        }

        if keys.pressed(KeyCode::Right) {
            transform.rotate_y(0.05 * dt * rotation_speed);
        }

        if keys.pressed(KeyCode::Left) {
            transform.rotate_y(-0.05 * dt * rotation_speed);
        }

        // if keys.pressed(KeyCode::Up) {
        //     transform.rotate_x(-0.05);
        // }

        // if keys.pressed(KeyCode::Down) {
        //     transform.rotate_x(0.05);
        // }
    }
}

fn draw(
    mut pixel_handler: PixelHandler,
    player_query: Query<&Transform, With<Viewpoint>>,
    // mut sectors: ResMut<Sectors>,
    mut level_query: Query<&mut Level>,
) {
    if let Ok(transform) = player_query.get_single() {
        let (angle_up, angle, _) = transform.rotation.to_euler(EulerRot::XYZ);
        let player_cos = angle.cos();
        let player_sin = angle.sin();
        if let Some(mut level) = level_query.iter_mut().next() {
            bubble_sort(&mut level.sectors);
            level.sectors.reverse();
            for sector in level.sectors.iter_mut() {
                let mut x_points = vec![0; pixel_handler.width() as usize];
                sector.depth = 0.0;
                if transform.translation.y < sector.floor {
                    sector.surface = Surface::Bottom;
                } else if transform.translation.y > sector.roof {
                    sector.surface = Surface::Top;
                } else {
                    sector.surface = Surface::Normal;
                }
                for i in 0..2 {
                    for wall in sector.walls.iter() {
                        let mut local_wall = [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)];

                        // Offset bottom two points of wall by player position
                        let mut x1 = wall.points[0].x - transform.translation.x;
                        let mut z1 = wall.points[0].y - transform.translation.z;
                        let mut x2 = wall.points[1].x - transform.translation.x;
                        let mut z2 = wall.points[1].y - transform.translation.z;

                        if i == 0 {
                            let mut swp = x1;
                            x1 = x2;
                            x2 = swp;
                            swp = z1;
                            z1 = z2;
                            z2 = swp;
                        }

                        local_wall[0].x = x1 * player_cos - z1 * player_sin;
                        local_wall[1].x = x2 * player_cos - z2 * player_sin;

                        local_wall[0].z = z1 * player_cos + x1 * player_sin;
                        local_wall[1].z = z2 * player_cos + x2 * player_sin;

                        local_wall[0].y = sector.floor - transform.translation.y
                            + (angle_up.to_degrees() * local_wall[0].z / 32.0);
                        local_wall[1].y = sector.floor - transform.translation.y
                            + (angle_up.to_degrees() * local_wall[1].z / 32.0);

                        let top_y1 = sector.roof - transform.translation.y
                            + (angle_up.to_degrees() * local_wall[0].z / 32.0);
                        let top_y2 = sector.roof - transform.translation.y
                            + (angle_up.to_degrees() * local_wall[1].z / 32.0);

                        sector.depth += Vec2::ZERO.distance(Vec2::new(
                            (local_wall[0].x + local_wall[1].x) / 2.0,
                            (local_wall[0].z + local_wall[1].z) / 2.0,
                        ));

                        if local_wall[0].z < 1.0 && local_wall[1].z < 1.0 {
                            continue;
                        }

                        let mut b1 = IVec3::new(
                            local_wall[0].x as i32,
                            local_wall[0].y as i32,
                            local_wall[0].z as i32,
                        );
                        let mut b2 = IVec3::new(
                            local_wall[1].x as i32,
                            local_wall[1].y as i32,
                            local_wall[1].z as i32,
                        );
                        let mut t1 = IVec3::new(
                            local_wall[0].x as i32,
                            top_y1 as i32,
                            local_wall[0].z as i32,
                        );
                        let mut t2 = IVec3::new(
                            local_wall[1].x as i32,
                            top_y2 as i32,
                            local_wall[1].z as i32,
                        );

                        if local_wall[0].z < 1.0 {
                            clip_behind(&mut b1, &mut b2);
                            clip_behind(&mut t1, &mut t2);
                        }

                        if local_wall[1].z < 1.0 {
                            clip_behind(&mut b2, &mut b1);
                            clip_behind(&mut t2, &mut t1);
                        }

                        let (scr_x1, scr_y1) = (
                            b1.x as f32 * 200.0 / b1.z as f32 + (pixel_handler.width() / 2) as f32,
                            b1.y as f32 * 200.0 / b1.z as f32 + (pixel_handler.height() / 2) as f32,
                        );

                        let (scr_x2, scr_y2) = (
                            b2.x as f32 * 200.0 / b2.z as f32 + (pixel_handler.width() / 2) as f32,
                            b2.y as f32 * 200.0 / b2.z as f32 + (pixel_handler.height() / 2) as f32,
                        );

                        let (_, scr_y3) = (
                            t1.x as f32 * 200.0 / t1.z as f32 + (pixel_handler.width() / 2) as f32,
                            t1.y as f32 * 200.0 / t1.z as f32 + (pixel_handler.height() / 2) as f32,
                        );

                        let (_, scr_y4) = (
                            t2.x as f32 * 200.0 / t2.z as f32 + (pixel_handler.width() / 2) as f32,
                            t2.y as f32 * 200.0 / t2.z as f32 + (pixel_handler.height() / 2) as f32,
                        );

                        draw_wall(
                            IVec3::new(scr_x1 as i32, scr_y1 as i32, scr_y3 as i32),
                            IVec3::new(scr_x2 as i32, scr_y2 as i32, scr_y4 as i32),
                            &mut pixel_handler,
                            wall.color,
                            sector.surface,
                            (sector.roof_col, sector.floor_col),
                            &mut x_points,
                        );
                    }
                    sector.depth /= sector.walls.len() as f32;
                    match sector.surface {
                        Surface::Top => {
                            sector.surface = Surface::TopReverse;
                        }
                        Surface::Bottom => {
                            sector.surface = Surface::BottomReverse;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn bubble_sort<T: Ord>(arr: &mut [T]) {
    for i in 0..arr.len() {
        for j in 0..arr.len() - 1 - i {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
            }
        }
    }
}

fn clip_behind(position_one: &mut IVec3, position_two: &mut IVec3) {
    let da = position_one.z;
    let db = position_two.z;

    let mut d = da - db;
    if db == 0 {
        d = 1;
    }

    let s = da / d;

    position_one.x += s * (position_two.x - (position_one.x));
    position_one.z += s * (position_two.z - (position_one.z));
    if position_one.z == 0 {
        position_one.z = 1;
    }
    position_one.y += s * (position_two.y - (position_one.y));
}

fn draw_wall(
    position_one: IVec3,
    position_two: IVec3,
    pixel_handler: &mut PixelHandler,
    color: PixColor,
    surface: Surface,
    (roof_col, floor_col): (PixColor, PixColor),
    x_points: &mut Vec<i32>,
) {
    let mut position_one = position_one.clone();
    let mut position_two = position_two.clone();

    if let Some(dyb) = position_two.y.checked_sub(position_one.y) {
        let dzb = position_two.z - position_one.z;
        let mut dx = position_two.x - position_one.x;
        if dx == 0 {
            dx = 1;
        }

        if position_one.x < 1 {
            position_one.x = 1;
        }

        if position_two.x < 1 {
            position_two.x = 1;
        }

        if position_one.x > (pixel_handler.width() - 1) as i32 {
            position_one.x = (pixel_handler.width() - 1) as i32;
        }

        if position_two.x > (pixel_handler.width() - 1) as i32 {
            position_two.x = (pixel_handler.width() - 1) as i32;
        }
        for x in position_one.x..position_two.x {
            let mut y1 = dyb * (x - position_one.x) / dx + position_one.y;
            let mut y2 = dzb * (x - position_one.x) / dx + position_one.z;

            if y1 < 1 {
                y1 = 1;
            }

            if y2 < 1 {
                y2 = 1;
            }

            if y1 > (pixel_handler.height() - 1) as i32 {
                y1 = (pixel_handler.height() - 1) as i32;
            }

            if y2 > (pixel_handler.height() - 1) as i32 {
                y2 = (pixel_handler.height() - 1) as i32;
            }

            if surface == Surface::Bottom {
                x_points.insert(x as usize, y1);
                continue;
            }
            if surface == Surface::Top {
                x_points.insert(x as usize, y2);
                continue;
            }
            if surface == Surface::BottomReverse {
                for i in x_points[x as usize]..y1 {
                    pixel_handler.set_pixel(UVec2::new(x as u32, i as u32), floor_col);
                }
            }
            if surface == Surface::TopReverse {
                for i in y2..x_points[x as usize] {
                    pixel_handler.set_pixel(UVec2::new(x as u32, i as u32), roof_col);
                }
            }

            for y in y1..y2 {
                pixel_handler.set_pixel(UVec2::new(x as u32, y as u32), color);
            }
        }
    }
}
