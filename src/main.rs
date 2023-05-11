use std::cmp::Ordering;

use bevy::{ecs::system::SystemParam, prelude::*, utils::FloatOrd};
use bevy_pixels::prelude::*;
use rand::prelude::*;

#[derive(Clone, Copy)]
struct Wall {
    points: [Vec2; 2],
    color: Color, // height: f32,
}

struct Sector {
    walls: Vec<Wall>,
    center: Vec2,
    depth: f32,
    roof: f32, // Top and bottom height of walls
    floor: f32,
}

impl Ord for Sector {
    fn cmp(&self, other: &Self) -> Ordering {
        FloatOrd(self.depth).cmp(&FloatOrd(other.depth))
    }
}

impl PartialOrd for Sector {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Sector {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth
    }
}

impl Eq for Sector {}

impl Sector {
    fn new(floor: f32, roof: f32) -> Self {
        Self {
            roof,
            floor,
            depth: 0.0,
            center: Vec2::ZERO,
            walls: Vec::default(),
        }
    }

    fn add_wall(&mut self, bottom_one: Vec2, bottom_two: Vec2, color: Color) {
        self.walls.push(Wall {
            points: [bottom_one, bottom_two],
            color,
        });
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct Sectors(Vec<Sector>);

#[derive(Component)]
struct Viewpoint;

#[derive(Component, Debug, Clone, Copy)]
struct Color(u8, u8, u8, u8);

#[derive(SystemParam)]
struct PixelHandler<'w, 's> {
    // commands: Commands<'w, 's>,
    pixel_wrapper: Query<'w, 's, &'static mut PixelsWrapper>,
    options_query: Query<'w, 's, &'static PixelsOptions>,
}

impl<'w, 's> PixelHandler<'w, 's> {
    fn clear(&mut self, color: Color) {
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

    fn set_pixel(&mut self, position: UVec2, color: Color) {
        let Ok(options) = self.options_query.get_single() else { return };
        if position.x < options.width && position.y < options.height {
            let Ok(mut wrapper) = self.pixel_wrapper.get_single_mut() else { return };
            let frame_width_bytes = (options.width * 4) as usize;
            let frame: &mut [u8] = wrapper.pixels.frame_mut();
            let x_offset = (position.x * 4) as usize;
            let y_offset = position.y as usize * frame_width_bytes;
            let i = x_offset + y_offset;
            let j = i + 4;
            frame[i..j].copy_from_slice(&[color.0, color.1, color.2, color.3]);
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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
        width: 320,
        height: 240,
        // scale_factor: f32,
        auto_resize_buffer: false,
        ..default()
    };
}

fn clear(mut pixel_handler: PixelHandler) {
    pixel_handler.clear(Color(0, 0, 0, 255));
}

fn setup(mut commands: Commands) {
    commands.spawn((Transform::from_xyz(70.0, 20.0, -110.0), Viewpoint));
    let mut sectors = Sectors::default();
    let mut sector = Sector::new(10.0, 40.0);
    sector.add_wall(
        Vec2::new(0.0, 25.0),
        Vec2::new(0.0, 0.0),
        Color(128, 128, 128, 255),
    );
    sector.add_wall(
        Vec2::new(25.0, 25.0),
        Vec2::new(0.0, 25.0),
        Color(100, 100, 100, 255),
    );
    sector.add_wall(
        Vec2::new(25.0, 0.0),
        Vec2::new(25.0, 25.0),
        Color(128, 128, 128, 255),
    );
    sector.add_wall(
        Vec2::new(0.0, 0.0),
        Vec2::new(25.0, 0.0),
        Color(100, 100, 100, 255),
    );
    sectors.push(sector);
    let mut sector = Sector::new(10.0, 40.0);
    sector.add_wall(
        Vec2::new(30.0, 50.0),
        Vec2::new(30.0, 30.0),
        Color(200, 0, 0, 255),
    );
    sector.add_wall(
        Vec2::new(50.0, 50.0),
        Vec2::new(30.0, 50.0),
        Color(255, 0, 0, 255),
    );
    sector.add_wall(
        Vec2::new(50.0, 30.0),
        Vec2::new(50.0, 50.0),
        Color(200, 0, 0, 255),
    );
    sector.add_wall(
        Vec2::new(30.0, 30.0),
        Vec2::new(50.0, 30.0),
        Color(255, 0, 0, 255),
    );
    sectors.push(sector);
    commands.insert_resource(sectors);
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
            transform.translation.y -= 4.0 * dt * speed;
        }

        if keys.pressed(KeyCode::C) {
            transform.translation.y += 4.0 * dt * speed;
        }

        if keys.pressed(KeyCode::Right) {
            transform.rotate_y(0.05 * dt * rotation_speed);
        }

        if keys.pressed(KeyCode::Left) {
            transform.rotate_y(-0.05 * dt * rotation_speed);
        }

        // if keys.pressed(KeyCode::Up) {
        //     transform.rotate_x(0.05);
        // }

        // if keys.pressed(KeyCode::Down) {
        //     transform.rotate_x(-0.05);
        // }
    }
}

fn draw(
    mut pixel_handler: PixelHandler,
    player_query: Query<&Transform, With<Viewpoint>>,
    mut sectors: ResMut<Sectors>,
) {
    if let Ok(transform) = player_query.get_single() {
        let (angle_up, angle, _) = transform.rotation.to_euler(EulerRot::XYZ);
        let player_cos = angle.cos();
        let player_sin = angle.sin();
        bubble_sort(&mut sectors);
        sectors.reverse();
        for sector in sectors.iter_mut() {
            sector.depth = 0.0;
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
                    );
                }
                sector.depth /= sector.walls.len() as f32;
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
    color: Color,
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

            for y in y1..y2 {
                pixel_handler.set_pixel(UVec2::new(x as u32, y as u32), color);
            }
        }
    }
}
