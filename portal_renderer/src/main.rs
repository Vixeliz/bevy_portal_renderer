use std::f32::consts::PI;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_pixel_buffer::prelude::*;
use portal_common::prelude::*;

#[derive(Component)]
struct Viewpoint;

#[derive(SystemParam)]
struct PixelHandler<'w, 's> {
    // commands: Commands<'w, 's>,
    pixel_wrapper: QueryPixelBuffer<'w, 's>,
}

impl<'w, 's> PixelHandler<'w, 's> {
    fn clear(&mut self, color: PixColor) {
        self.pixel_wrapper.frame().per_pixel(|_, _| color);
    }

    fn height(&mut self) -> u32 {
        self.pixel_wrapper.frame().size().y
    }

    fn width(&mut self) -> u32 {
        self.pixel_wrapper.frame().size().x
    }

    fn set_pixel(&mut self, position: UVec2, color: PixColor) {
        if position.x < self.width() && position.y < self.height() {
            let final_y = position.y as i32 - self.height() as i32;
            let final_y = final_y.abs();
            self.pixel_wrapper
                .frame()
                .set(UVec2::new(position.x, final_y as u32), color)
                .ok();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PixelBufferPlugin)
        .add_startup_system(
            PixelBufferBuilder::new()
                .with_size(PixelBufferSize::pixel_size(UVec2::new(4, 4)))
                .with_fill(Fill::window())
                .setup(),
        )
        .add_startup_system(setup)
        .add_system(move_player)
        .add_system(clear.before(draw))
        .add_system(draw)
        .run();
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
        let (angle_up, angle, z_angle) = transform.rotation.to_euler(EulerRot::XYZ);
        // Bevy returns half for some reason
        let angle = angle * 2.0;
        let dt = time.delta_seconds();
        let speed = 50.0;
        let rotation_speed = 25.0;
        let rotation_speed_up = 15.0;
        let dx = angle.sin();
        let dz = angle.cos();

        if keys.pressed(KeyCode::W) {
            transform.translation.x += dx;
            transform.translation.z += dz;
        }
        if keys.pressed(KeyCode::S) {
            transform.translation.x -= dx;
            transform.translation.z -= dz;
        }
        if keys.pressed(KeyCode::D) {
            transform.translation.x += dz;
            transform.translation.z -= dx;
        }
        if keys.pressed(KeyCode::A) {
            transform.translation.x -= dz;
            transform.translation.z += dx;
        }
        if keys.pressed(KeyCode::Space) {
            transform.translation.y += 4.0 * dt * speed;
        }

        if keys.pressed(KeyCode::C) {
            transform.translation.y -= 4.0 * dt * speed;
        }
        let mut local_angle = angle.clone();
        let mut local_angle_up = angle_up.clone();
        if keys.pressed(KeyCode::Right) {
            local_angle += 0.05 * dt * rotation_speed;
        }

        if keys.pressed(KeyCode::Left) {
            local_angle -= 0.05 * dt * rotation_speed;
        }

        if keys.pressed(KeyCode::Up) {
            local_angle_up -= 0.05 * dt * rotation_speed_up;
        }

        if keys.pressed(KeyCode::Down) {
            local_angle_up += 0.05 * dt * rotation_speed_up;
        }

        // Wrap around
        if local_angle < -PI {
            local_angle = PI;
        }
        if local_angle > PI {
            local_angle = -PI;
        }

        local_angle_up = local_angle_up.clamp(-PI / 2.0, PI / 2.0);

        let local_angle = local_angle / 2.0;

        transform.rotation = Quat::from_euler(EulerRot::XYZ, local_angle_up, local_angle, z_angle);
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
        let angle = angle * 2.0;
        let angle_up = angle_up * 2.0;
        let player_cos = angle.cos();
        let player_sin = angle.sin();
        if let Some(mut level) = level_query.iter_mut().next() {
            // Sort the levels sectors from back to front
            bubble_sort(&mut level.sectors);
            level.sectors.reverse();
            for sector in level.sectors.iter_mut() {
                // Temp vector to hold row of pixels for filling bottom or top
                let mut x_points = vec![0; pixel_handler.width() as usize];

                // Reset sector depth
                sector.depth = 0.0;

                // Set what surface we are rendering based off of player location relative to this sector
                if transform.translation.y < sector.floor {
                    sector.surface = Surface::Bottom;
                } else if transform.translation.y > sector.roof {
                    sector.surface = Surface::Top;
                } else {
                    sector.surface = Surface::Normal;
                }

                // Two loops are needed for filling in top and bottoms
                for i in 0..2 {
                    // Loop through the sector walls
                    for wall in sector.walls.iter() {
                        // Temporary local_wall varibale
                        let mut local_wall = [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)];

                        // Offset bottom two points of wall by player position
                        /*
                            In 3d graphics its more we move the world instead of the camera.
                            So first we must move all the world using the camera pos
                        */
                        let mut x1 = wall.points[0].x - transform.translation.x;
                        let mut z1 = wall.points[0].y - transform.translation.z;
                        let mut x2 = wall.points[1].x - transform.translation.x;
                        let mut z2 = wall.points[1].y - transform.translation.z;

                        // If first loop we swap variables
                        if i == 0 {
                            let mut swp = x1;
                            x1 = x2;
                            x2 = swp;
                            swp = z1;
                            z1 = z2;
                            z2 = swp;
                        }

                        // Use player rotation to adjust the points of the wall
                        local_wall[0].x = x1 * player_cos - z1 * player_sin;
                        local_wall[1].x = x2 * player_cos - z2 * player_sin;

                        local_wall[0].z = z1 * player_cos + x1 * player_sin;
                        local_wall[1].z = z2 * player_cos + x2 * player_sin;

                        // Translate the height based off of the sector, player location, and angle
                        local_wall[0].y = sector.floor - transform.translation.y
                            + (angle_up.to_degrees() * local_wall[0].z / 32.0);
                        local_wall[1].y = sector.floor - transform.translation.y
                            + (angle_up.to_degrees() * local_wall[1].z / 32.0);

                        let top_y1 = sector.roof - transform.translation.y
                            + (angle_up.to_degrees() * local_wall[0].z / 32.0);
                        let top_y2 = sector.roof - transform.translation.y
                            + (angle_up.to_degrees() * local_wall[1].z / 32.0);

                        // Add this walls depth to the sector
                        if i == 0 {
                            sector.depth += Vec2::ZERO.distance(Vec2::new(
                                (local_wall[0].x + local_wall[0].z) / 2.0,
                                (local_wall[1].x + local_wall[1].x) / 2.0,
                            ));
                        }

                        // If the local wall is behind the player we don't draw it
                        if local_wall[0].z < 0.0 && local_wall[1].z < 0.0 {
                            continue;
                        }

                        // New points to use for clipping bottom 1 and 2 and top 1 and 2
                        let mut b1 = Vec3::new(local_wall[0].x, local_wall[0].y, local_wall[0].z);
                        let mut b2 = Vec3::new(local_wall[1].x, local_wall[1].y, local_wall[1].z);
                        let mut t1 = Vec3::new(local_wall[0].x, top_y1, local_wall[0].z);
                        let mut t2 = Vec3::new(local_wall[1].x, top_y2, local_wall[1].z);

                        // Clip walls that are behind player at least partly
                        if local_wall[0].z < 0.0 {
                            clip_behind(&mut b1, &mut b2);
                            clip_behind(&mut t1, &mut t2);
                        }

                        if local_wall[1].z < 0.0 {
                            clip_behind(&mut b2, &mut b1);
                            clip_behind(&mut t2, &mut t1);
                        }

                        // Transform wall to screen coordinates
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

                        // Finally draw the wall
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

                    if i == 0 {
                        // Get the average depth
                        sector.depth /= sector.walls.len() as f32;
                    }

                    // Reverse the surface for tops or bottoms
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

fn clip_behind(position_one: &mut Vec3, position_two: &mut Vec3) {
    // Store the distance planes which are these two points
    let da = position_one.z;
    let db = position_two.z;

    // Get distance of planes
    let mut d = da - db;
    // Prevent divide by zero
    if db == 0.0 {
        d = 1.0;
    }

    // How much the plane is intersecting ranging from 0 to 1
    let s = da / d;

    // Finally using intersection factor set the points to the appropiate place
    position_one.x += s * (position_two.x - (position_one.x));
    position_one.z += s * (position_two.z - (position_one.z));
    if position_one.z == 0.0 {
        position_one.z = 1.0;
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

    // Get distance between points
    let dyb = position_two.y.wrapping_sub(position_one.y);
    let dzb = position_two.z.wrapping_sub(position_one.z);
    let mut dx = position_two.x.wrapping_sub(position_one.x);

    // Prevent divide by zero
    if dx == 0 {
        dx = 1;
    }

    // Clip sides of screen
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

    // Loop over the lines we have to draw for this wall
    for x in position_one.x..position_two.x {
        // Get screen y from the distances
        // Figure out a better way to prevent overflows
        let mut y1 = (dyb as i64 * (x as i64 - position_one.x as i64) / dx as i64
            + position_one.y as i64) as i32;
        let mut y2 = (dzb as i64 * (x as i64 - position_one.x as i64) / dx as i64
            + position_one.z as i64) as i32;
        // Clip top and bottom of screen
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

        // Handle surfaces for top and bottom on first pass we save the points but don't draw
        // second pass we actually draw
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

        // Finally always draw the normal wall
        for y in y1..y2 {
            pixel_handler.set_pixel(UVec2::new(x as u32, y as u32), color);
        }
    }
}
