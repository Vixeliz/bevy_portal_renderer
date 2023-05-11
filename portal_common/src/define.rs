use std::cmp::Ordering;

use bevy::{prelude::*, utils::FloatOrd};

#[derive(Clone, Copy)]
pub struct Wall {
    pub points: [Vec2; 2],
    pub color: Color, // height: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Top,
    Bottom,
    Normal,
    TopReverse,
    BottomReverse,
}

pub struct Sector {
    pub walls: Vec<Wall>,
    pub center: Vec2,
    pub depth: f32,
    pub roof: f32, // Top and bottom height of walls
    pub floor: f32,
    pub roof_col: Color,
    pub floor_col: Color,
    pub surface: Surface,
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
            roof_col: Color(0, 0, 255, 255),
            floor_col: Color(0, 255, 0, 255),
            surface: Surface::Normal,
        }
    }

    pub fn add_wall(&mut self, bottom_one: Vec2, bottom_two: Vec2, color: Color) {
        self.walls.push(Wall {
            points: [bottom_one, bottom_two],
            color,
        });
    }
}

#[derive(Component, Default)]
pub struct Level {
    pub sectors: Vec<Sector>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);
