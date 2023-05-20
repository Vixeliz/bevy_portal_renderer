use bevy_pixel_buffer::prelude::*;
use std::cmp::Ordering;

use bevy::prelude::Component;
use bevy::prelude::Vec2;
use bevy::utils::FloatOrd;

#[derive(Component, Debug, Clone, Copy)]
pub struct PixColor(pub u8, pub u8, pub u8, pub u8);

impl From<Pixel> for PixColor {
    fn from(item: Pixel) -> Self {
        PixColor(item.r, item.g, item.b, item.a)
    }
}

impl From<PixColor> for Pixel {
    fn from(item: PixColor) -> Self {
        Pixel {
            r: item.0,
            g: item.1,
            b: item.2,
            a: item.3,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Wall {
    pub points: [Vec2; 2],
    pub color: PixColor, // height: f32,
    pub uv: Vec2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Top,
    Bottom,
    Normal,
}

pub struct Sector {
    pub walls: Vec<Wall>,
    pub center: Vec2,
    pub depth: f32,
    pub roof: f32, // Top and bottom height of walls
    pub floor: f32,
    pub roof_col: PixColor,
    pub floor_col: PixColor,
    pub surface: Surface,
    pub x_points: Vec<u32>,
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
    pub fn new(floor: f32, roof: f32) -> Self {
        Self {
            roof,
            floor,
            depth: 0.0,
            center: Vec2::ZERO,
            walls: Vec::default(),
            roof_col: PixColor(0, 0, 255, 255),
            floor_col: PixColor(0, 255, 0, 255),
            surface: Surface::Normal,
            x_points: vec![0; 240 * 360],
        }
    }

    pub fn add_wall(&mut self, bottom_one: Vec2, bottom_two: Vec2, color: PixColor) {
        self.walls.push(Wall {
            points: [bottom_one, bottom_two],
            color,
            uv: Vec2::new(5.0, 1.0), // Should be calculated in editor
                                     // uv: Vec2::ONE,
        });
    }
}

#[derive(Component, Default)]
pub struct Level {
    pub sectors: Vec<Sector>,
}
