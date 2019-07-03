
use piston_window::*;
use piston_window::draw_state::Blend;
use std::ops::{Add, Mul};
use piston_window::math::{rotate_radians, translate};

use super::consts::WIDTH;
use super::consts::HEIGHT;

pub struct Entity<'a>{
    pub pos: [f64; 2],
    pub velo: [f64; 2],
    pub health: i32,
    pub rotation: f32,
    pub blend: Option<Blend>,
    pub texture: &'a G2dTexture
}

// We cannot directly define custom operators on external types, so we wrap the matrix
// int a tuple struct.
pub struct Matrix<T>(pub vecmath::Matrix2x3<T>);

// This is such a silly way to operator overload to enable matrix multiplication with
// operator *.
impl<T> Mul for Matrix<T>
    where T: Copy + Add<T, Output = T> + Mul<T, Output = T> {
    type Output = Self;
    fn mul(self, o: Self) -> Self{
        Matrix(vecmath::row_mat2x3_mul(self.0, o.0))
    }
}

impl<'a> Entity<'a>{
    pub fn new(pos: [f64; 2], velo: [f64; 2], texture: &'a G2dTexture) -> Self{
        Self{
            pos: pos,
            velo: velo,
            health: 1,
            rotation: 0.,
            blend: None,
            texture: texture
        }
    }

    pub fn health(mut self, health: i32) -> Self{
        self.health = health;
        self
    }

    pub fn blend(mut self, blend: Blend) -> Self{
        self.blend = Some(blend);
        self
    }

    pub fn rotation(mut self, rotation: f32) -> Self{
        self.rotation = rotation;
        self
    }

    pub fn animate(&mut self) -> bool{
        let pos = &mut self.pos;
        for i in 0..2 {
            pos[i] = pos[i] + self.velo[i];
        }
        if self.health <= 0 || pos[0] < 0. || (WIDTH as f64) < pos[0] || pos[1] < 0. || (HEIGHT as f64) < pos[1] {
            false
        }
        else{
            true
        }
    }

    pub fn draw_tex(&self, context: &Context, g: &mut G2d){
        let pos = &self.pos;
        let tex2 = self.texture;
        let centerize = translate([-(tex2.get_width() as f64 / 2.), -(tex2.get_height() as f64 / 2.)]);
        let rotmat = rotate_radians(self.rotation as f64);
        let translate = translate(*pos);
        let draw_state = if let Some(blend_mode) = self.blend { context.draw_state.blend(blend_mode) } else { context.draw_state };
        let image   = Image::new().rect([0., 0., tex2.get_width() as f64, tex2.get_height() as f64]);
        image.draw(tex2, &draw_state, (Matrix(context.transform) * Matrix(translate) * Matrix(rotmat) * Matrix(centerize)).0, g);
    }
}

pub type TempEntity<'a> = Entity<'a>;

pub const MAX_FRAMES: u32 = 8;
pub const PLAYBACK_RATE: u32 = 3;

impl<'a> TempEntity<'a>{
    pub fn animate_temp(&mut self) -> bool{
        self.health -= 1;
        self.animate()
    }

    pub fn draw_temp(&self, context: &Context, g: &mut G2d){
        let pos = &self.pos;
        let tex2 = self.texture;
        let centerize = translate([-(16. / 2.), -(tex2.get_height() as f64 / 2.)]);
        let rotmat = rotate_radians(self.rotation as f64);
        let translate = translate(*pos);
        let frame = MAX_FRAMES - (self.health as u32 / PLAYBACK_RATE) as u32;
        let draw_state = if let Some(blend_mode) = self.blend { context.draw_state.blend(blend_mode) } else { context.draw_state };
        let image   = Image::new().rect([0f64, 0f64, 16., tex2.get_height() as f64])
            .src_rect([frame as f64 * 16., 0., 16., tex2.get_height() as f64]);
        image.draw(tex2, &draw_state, (Matrix(context.transform) * Matrix(translate) * Matrix(rotmat) * Matrix(centerize)).0, g);
    }
}

