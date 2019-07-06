
use piston_window::*;
use piston_window::draw_state::Blend;
use std::ops::{Add, Mul};
use piston_window::math::{rotate_radians, translate};
use vecmath::*;

use super::consts::*;

/// The base structure of all Entities.  Implements common methods.
pub struct Entity<'a>{
    pub id: u32,
    pub pos: [f64; 2],
    pub velo: [f64; 2],
    pub health: i32,
    pub rotation: f32,
    pub blend: Option<Blend>,
    pub texture: &'a G2dTexture
}

pub enum DeathReason{
    RangeOut,
    Killed
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
    pub fn new(id_gen: &mut u32, pos: [f64; 2], velo: [f64; 2], texture: &'a G2dTexture) -> Self{
        *id_gen += 1;
        Self{
            id: *id_gen,
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

    /// Returns None if the Entity survived this frame.
    /// Otherwise returns Some(reason) where reason is DeathReason.
    pub fn animate(&mut self) -> Option<DeathReason>{
        let pos = &mut self.pos;
        for i in 0..2 {
            pos[i] = pos[i] + self.velo[i];
        }
        if self.health <= 0 {
            Some(DeathReason::Killed)
        }
        else if pos[0] < 0. || (WIDTH as f64) < pos[0] || pos[1] < 0. || (HEIGHT as f64) < pos[1] {
            Some(DeathReason::RangeOut)
        }
        else{
            None
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

pub type Enemy<'a> = Entity<'a>;

pub struct BulletBase<'a>(pub Entity<'a>, pub bool);

pub enum Projectile<'a>{
    Bullet(BulletBase<'a>),
    Missile{base: BulletBase<'a>, target: u32, trail: Vec<[f64; 2]>}
}

const MISSILE_DETECTION_RANGE: f64 = 256.;
const MISSILE_HOMING_SPEED: f64 = 0.25;
const MISSILE_TRAIL_LENGTH: usize = 20;

impl<'a> Projectile<'a>{
    pub fn get_base<'b>(&'b self) -> &'b BulletBase{
        match &self {
            &Projectile::Bullet(base) => base,
            &Projectile::Missile{base, target: _, trail: _} => base
        }
    }
    // pub fn get_base_mut(&'a mut self) -> &'a mut BulletBase{
    //     match &mut self {
    //         &mut Projectile::Bullet(base) => &mut base,
    //         &mut Projectile::Missile(base, _) => &mut base
    //     }
    // }

    pub fn get_type(&self) -> &str{
        match &self{
            &Projectile::Bullet(_) => "Bullet",
            &Projectile::Missile{base: _, target: _, trail: _} => "Missile"
        }
    }

    fn animate_common(mut base: &mut BulletBase, enemies: &mut Vec<Enemy>) -> Option<DeathReason>{
        let &mut BulletBase(ent, team) = &mut base;
        if *team {
            for e in enemies.iter_mut() {
                if ent.pos[0] - BULLET_SIZE < e.pos[0] + ENEMY_SIZE && e.pos[0] - ENEMY_SIZE < ent.pos[0] + BULLET_SIZE &&
                    ent.pos[1] - BULLET_SIZE < e.pos[1] + ENEMY_SIZE && e.pos[1] - ENEMY_SIZE < ent.pos[1] + BULLET_SIZE {
                    e.health -= ent.health;
                    ent.health = 0;
                    break;
                }
            }
        }
        ent.animate()
    }

    pub fn animate_bullet(&mut self, enemies: &mut Vec<Enemy>) -> Option<DeathReason>{
        Self::animate_common(match self {
            Projectile::Bullet(base) => base,
            Projectile::Missile{base, target, trail} => {
                if *target == 0 {
                    let best = enemies.iter().fold((0, 1e5), |bestpair, e| {
                        let dist = vec2_len(vec2_sub(base.0.pos, e.pos));
                        if dist < MISSILE_DETECTION_RANGE && dist < bestpair.1 {
                            (e.id, dist)
                        }
                        else{
                            bestpair
                        }
                    });
                    *target = best.0;
                }
                else if let Some(target_ent) = enemies.iter().find(|e| e.id == *target) {
                    let norm = vec2_normalized(vec2_sub(target_ent.pos, base.0.pos));
                    let desired_velo = vec2_scale(norm, MISSILE_SPEED);
                    let desired_diff = vec2_sub(desired_velo, base.0.velo);
                    if std::f64::EPSILON < vec2_square_len(desired_diff) {
                        base.0.velo = if vec2_square_len(desired_diff) < MISSILE_HOMING_SPEED * MISSILE_HOMING_SPEED {
                            desired_velo
                        }
                        else{
                            let desired_diff_norm = vec2_normalized(desired_diff);
                            vec2_add(base.0.velo, vec2_scale(desired_diff_norm, MISSILE_HOMING_SPEED))
                        };
                        let angle = base.0.velo[1].atan2(base.0.velo[0]);
                        base.0.rotation = (angle + std::f64::consts::FRAC_PI_2) as f32;
                        let (s, c) = angle.sin_cos();
                        base.0.velo[0] = MISSILE_SPEED * c;
                        base.0.velo[1] = MISSILE_SPEED * s;
                    }
                }
                else{
                    *target = 0
                }
                if MISSILE_TRAIL_LENGTH < trail.len() {
                    trail.remove(0);
                }
                trail.push(base.0.pos);
                base
            }
        }, enemies)
    }

    pub fn draw(&self, c: &Context, g: &mut G2d){
        if let Projectile::Missile{base: _, target: _, trail} = self {
            let mut iter = trail.iter().enumerate();
            if let Some(mut prev) = iter.next() {
                for e in iter {
                    line([0.75, 0.75, 0.75, e.0 as f32 / MISSILE_TRAIL_LENGTH as f32],
                        e.0 as f64 / MISSILE_TRAIL_LENGTH as f64,
                        [prev.1[0], prev.1[1], e.1[0], e.1[1]], c.transform, g);
                    prev = e;
                }
            }
        }
        self.get_base().0.draw_tex(c, g);
    }
}



pub struct TempEntity<'a>{
    pub base: Entity<'a>,
    pub max_frames: u32,
    pub width: u32,
    pub playback_rate: u32
}

impl<'a> TempEntity<'a>{
    #[allow(dead_code)]
    pub fn max_frames(mut self, max_frames: u32) -> Self{
        self.max_frames = max_frames;
        self
    }
    pub fn animate_temp(&mut self) -> Option<DeathReason> {
        self.base.health -= 1;
        self.base.animate()
    }

    pub fn draw_temp(&self, context: &Context, g: &mut G2d){
        let pos = &self.base.pos;
        let tex2 = self.base.texture;
        let centerize = translate([-(16. / 2.), -(tex2.get_height() as f64 / 2.)]);
        let rotmat = rotate_radians(self.base.rotation as f64);
        let translate = translate(*pos);
        let frame = self.max_frames - (self.base.health as u32 / self.playback_rate) as u32;
        let draw_state = if let Some(blend_mode) = self.base.blend { context.draw_state.blend(blend_mode) } else { context.draw_state };
        let image   = Image::new().rect([0f64, 0f64, self.width as f64, tex2.get_height() as f64])
            .src_rect([frame as f64 * self.width as f64, 0., self.width as f64, tex2.get_height() as f64]);
        image.draw(tex2, &draw_state, (Matrix(context.transform) * Matrix(translate) * Matrix(rotmat) * Matrix(centerize)).0, g);
    }
}

