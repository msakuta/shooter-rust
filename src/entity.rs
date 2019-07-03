
use piston_window::*;
use piston_window::draw_state::Blend;
use std::ops::{Add, Mul};

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
        let mut centerize = vecmath::mat2x3_id();
        centerize[0][2] = -(tex2.get_width() as f64 / 2.);
        centerize[1][2] = -(tex2.get_height() as f64 / 2.);
        let mut mytran = vecmath::mat2x3_id();
        let rotmat = piston_window::math::rotate_radians(self.rotation as f64);
        mytran[0][2] = pos[0];
        mytran[1][2] = pos[1];
        let draw_state = if let Some(blend_mode) = self.blend { context.draw_state.blend(blend_mode) } else { context.draw_state };
        let image   = Image::new().rect([0., 0., tex2.get_width() as f64, tex2.get_height() as f64]);
        image.draw(tex2, &draw_state, (Matrix(context.transform) * Matrix(mytran) * Matrix(rotmat) * Matrix(centerize)).0, g);
    }
}
