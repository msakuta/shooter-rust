
use piston_window::*;
use piston_window::draw_state::Blend;
use std::ops::{Add, Mul};
use piston_window::math::{rotate_radians, translate, scale};
use vecmath::*;

use super::consts::*;

pub struct Assets{
    pub bg: G2dTexture,
    pub weapons_tex: G2dTexture,
    pub boss_tex: G2dTexture,
    pub enemy_tex: G2dTexture,
    pub player_tex: G2dTexture,
    pub shield_tex: G2dTexture,
    pub ebullet_tex: G2dTexture,
    pub bullet_tex: G2dTexture,
    pub missile_tex: G2dTexture,
    pub explode_tex: G2dTexture,
    pub explode2_tex: G2dTexture,
    pub sphere_tex: G2dTexture,
    pub power_tex: G2dTexture,
    pub power2_tex: G2dTexture,
}

impl Assets{
    pub fn new(window: &mut PistonWindow) -> (Self, Glyphs) {
        let assets_loader = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets").unwrap();

        let ref font = assets_loader.join("FiraSans-Regular.ttf");
        let factory = window.factory.clone();
        let glyphs = Glyphs::new(font, factory, TextureSettings::new()).unwrap();

        let mut load_texture = |name| {
            Texture::from_path(
                &mut window.factory,
                &assets_loader.join(name),
                Flip::None,
                &TextureSettings::new()
            ).unwrap()
        };

        (Self{
            bg: load_texture("bg.png"),
            weapons_tex: load_texture("weapons.png"),
            boss_tex: load_texture("boss.png"),
            enemy_tex: load_texture("enemy.png"),
            player_tex: load_texture("player.png"),
            shield_tex: load_texture("shield.png"),
            ebullet_tex: load_texture("ebullet.png"),
            bullet_tex: load_texture("bullet.png"),
            missile_tex: load_texture("missile.png"),
            explode_tex: load_texture("explode.png"),
            explode2_tex: load_texture("explode2.png"),
            sphere_tex: load_texture("sphere.png"),
            power_tex: load_texture("power.png"),
            power2_tex: load_texture("power2.png"),
        }, glyphs)
    }
}

/// The base structure of all Entities.  Implements common methods.
pub struct Entity{
    pub id: u32,
    pub pos: [f64; 2],
    pub velo: [f64; 2],
    pub health: i32,
    pub rotation: f32,
    pub blend: Option<Blend>,
}

pub enum DeathReason{
    RangeOut,
    Killed,
    HitPlayer
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

impl Entity{
    pub fn new(id_gen: &mut u32, pos: [f64; 2], velo: [f64; 2]) -> Self{
        *id_gen += 1;
        Self{
            id: *id_gen,
            pos: pos,
            velo: velo,
            health: 1,
            rotation: 0.,
            blend: None,
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
        // Only remove if the velocity is going outward
        else if pos[0] < 0. && self.velo[0] < 0. || (WIDTH as f64) < pos[0] && 0. < self.velo[0]
            || pos[1] < 0. && self.velo[1] < 0. || (HEIGHT as f64) < pos[1] && 0. < self.velo[1] {
            Some(DeathReason::RangeOut)
        }
        else{
            None
        }
    }

    pub fn draw_tex(&self, context: &Context, g: &mut G2d, texture: &G2dTexture){
        let pos = &self.pos;
        let tex2 = texture;
        let centerize = translate([-(tex2.get_width() as f64 / 2.), -(tex2.get_height() as f64 / 2.)]);
        let rotmat = rotate_radians(self.rotation as f64);
        let translate = translate(*pos);
        let draw_state = if let Some(blend_mode) = self.blend { context.draw_state.blend(blend_mode) } else { context.draw_state };
        let image   = Image::new().rect([0., 0., tex2.get_width() as f64, tex2.get_height() as f64]);
        image.draw(tex2, &draw_state, (Matrix(context.transform) * Matrix(translate) * Matrix(rotmat) * Matrix(centerize)).0, g);
    }

    pub fn hits_player(&self, player: &Self) -> Option<DeathReason> {
        let e = &player;
        if self.pos[0] - BULLET_SIZE < e.pos[0] + ENEMY_SIZE && e.pos[0] - ENEMY_SIZE < self.pos[0] + BULLET_SIZE &&
            self.pos[1] - BULLET_SIZE < e.pos[1] + ENEMY_SIZE && e.pos[1] - ENEMY_SIZE < self.pos[1] + BULLET_SIZE {
            Some(DeathReason::HitPlayer)
        }
        else{ None }
    }
}

#[test]
fn test_death_reason() {
    let mut id_gen: u32 = 0;

    // Tests for killed
    let mut ent = Entity::new(&mut id_gen, [10., 20.], [1., 2., ]).health(0);
    assert!(if let Some(DeathReason::Killed) = ent.animate() { true } else { false });

    // Tests for out of range on left edge
    let mut ent2 = Entity::new(&mut id_gen, [0., 20.], [-1., 6.]).health(10);
    assert!(if let Some(DeathReason::RangeOut) = ent2.animate() { true } else { false });

    // Allow incoming entity even if it's out of range
    let mut ent3 = Entity::new(&mut id_gen, [-1., 20.], [1., 6.]).health(10);
    assert!(if let None = ent3.animate() { true } else { false });

    // Tests for out of range on right edge
    let mut ent4 = Entity::new(&mut id_gen, [WIDTH as f64, 20.], [1., 6.]).health(10);
    assert!(if let Some(DeathReason::RangeOut) = ent4.animate() { true } else { false });

    // Allow incoming entity even if it's out of range
    let mut ent5 = Entity::new(&mut id_gen, [WIDTH as f64 + 1., 20.], [-1., 6.]).health(10);
    assert!(if let None = ent5.animate() { true } else { false });

    // Tests for out of range on top edge
    let mut ent6 = Entity::new(&mut id_gen, [20., 0.], [1., -1.]).health(10);
    assert!(if let Some(DeathReason::RangeOut) = ent6.animate() { true } else { false });

    // Allow incoming entity even if it's out of range
    let mut ent7 = Entity::new(&mut id_gen, [20., -1.], [1., 1.]).health(10);
    assert!(if let None = ent7.animate() { true } else { false });

    // Tests for out of range on bottom edge
    let mut ent4 = Entity::new(&mut id_gen, [20., HEIGHT as f64], [10., 1.]).health(10);
    assert!(if let Some(DeathReason::RangeOut) = ent4.animate() { true } else { false });

    // Allow incoming entity even if it's out of range
    let mut ent5 = Entity::new(&mut id_gen, [20., HEIGHT as f64 + 1.], [-1., -1.]).health(10);
    assert!(if let None = ent5.animate() { true } else { false });
}

pub struct Player{
    pub base: Entity,
    pub score: u32,
    pub kills: u32,
    pub power: u32,
    pub lives: u32,
    /// invincibility time caused by death or bomb
    pub invtime: u32
}

impl Player{
    pub fn new(base: Entity) -> Self{
        Self{base, score: 0, kills: 0, power: 0, lives: PLAYER_LIVES, invtime: 0}
    }

    pub fn move_up(&mut self){
        if PLAYER_SIZE <= self.base.pos[1] - PLAYER_SPEED {
            self.base.pos[1] -= PLAYER_SPEED;
        }
    }

    pub fn move_down(&mut self){
        if self.base.pos[1] + PLAYER_SPEED < HEIGHT as f64 - PLAYER_SIZE {
            self.base.pos[1] += PLAYER_SPEED;
        }
    }

    pub fn move_left(&mut self){
        if PLAYER_SIZE <= self.base.pos[0] - PLAYER_SPEED {
            self.base.pos[0] -= PLAYER_SPEED;
        }
    }

    pub fn move_right(&mut self){
        if self.base.pos[0] + PLAYER_SPEED < WIDTH as f64 - PLAYER_SIZE {
            self.base.pos[0] += PLAYER_SPEED;
        }
    }

    pub fn reset(&mut self){
        self.base.pos = [240., 400.];
        self.score = 0;
        self.kills = 0;
        self.power = 0;
        self.lives = PLAYER_LIVES;
        self.invtime = 0;
    }

    pub fn power_level(&self) -> u32{
        self.power >> 4
    }

    pub fn difficulty_level(&self) -> u32{
        self.score / 256
    }
}

#[test]
fn test_player_hit() {
    let mut id_gen: u32 = 0;

    // Tests for killed
    let ent = Entity::new(&mut id_gen, [10., 20.], [1., 2., ]).health(0);

    let mut player = Player::new(Entity::new(&mut id_gen, [10. + ENEMY_SIZE + 0.5, 20.], [0., 1.]));
    assert!(if let Some(DeathReason::HitPlayer) = ent.hits_player(&player.base) { true } else { false });

    player.base.pos[0] += BULLET_SIZE;
    assert!(if let None = ent.hits_player(&player.base) { true } else { false });
}


pub struct ShieldedBoss{
    pub base: Entity,
    pub shield_health: i32
}

impl ShieldedBoss{
    pub fn new(id_gen: &mut u32, pos: [f64; 2], velo: [f64; 2]) -> Self{
        Self{base: Entity::new(id_gen, pos, velo).health(64), shield_health: 64}
    }
}

pub enum Enemy{
    Enemy1(Entity),
    Boss(Entity),
    ShieldedBoss(ShieldedBoss)
}

impl Enemy{
    pub fn get_base<'b>(&'b self) -> &'b Entity{
        match &self {
            &Enemy::Enemy1(base) | &Enemy::Boss(base) => &base,
            &Enemy::ShieldedBoss(boss) => &boss.base
        }
    }

    // pub fn get_base_mut<'b, 'c: 'b + 'a>(&'c mut self) -> &'b mut Entity{
    //     let mref: &mut Self = self;
    //     match mref {
    //         Enemy::Enemy1(ref mut base) | Enemy::Boss(ref mut base) => base,
    //         Enemy::ShieldedBoss(ref mut boss) => &mut boss.base
    //     }
    // }

    pub fn get_id(&self) -> u32{
        self.get_base().id
    }

    pub fn damage(&mut self, val: i32){
        match self {
            Enemy::Enemy1(ref mut base) | Enemy::Boss(ref mut base) => base.health -= val,
            Enemy::ShieldedBoss(ref mut boss) => {
                if boss.shield_health < 16 {
                    boss.base.health -= val
                }
                else {
                    boss.shield_health -= val
                }
            }
        }
    }

    pub fn animate<'b>(&'b mut self, time: u32) -> Option<DeathReason>{
        match self {
            Enemy::Enemy1(ref mut base) | Enemy::Boss(ref mut base) => base.animate(),
            Enemy::ShieldedBoss(ref mut boss) => {
                if boss.shield_health < 64 && time % 8 == 0 {
                    boss.shield_health += 1;
                }
                boss.base.animate()
            }
        }
        //let a = self.get_base_mut();
        //a.animate()
    }

    pub fn draw(&self, context: &Context, g: &mut G2d, assets: &Assets){
        self.get_base().draw_tex(context, g, match self {
            Enemy::Enemy1(_) => &assets.enemy_tex,
            Enemy::Boss(_) | Enemy::ShieldedBoss(_) => &assets.boss_tex
        });
        if let Enemy::ShieldedBoss(ref boss) = self {
            let pos = &boss.base.pos;
            let tex2 = &assets.shield_tex;
            let centerize = translate([-(tex2.get_width() as f64 / 2.), -(tex2.get_height() as f64 / 2.)]);
            let rotmat = rotate_radians(0 as f64);
            let scalemat = scale(boss.shield_health as f64 / 64., boss.shield_health as f64 / 64.);
            let translate = translate(*pos);
            let draw_state = context.draw_state;
            let image   = Image::new().rect([0., 0., tex2.get_width() as f64, tex2.get_height() as f64]);
            image.draw(tex2, &draw_state, (Matrix(context.transform) * Matrix(translate) * Matrix(scalemat) * Matrix(rotmat) * Matrix(centerize)).0, g);
        }
    }

    pub fn test_hit(&self, rect: [f64; 4]) -> bool{
        let rect2 = self.get_bb();
        rect[0] < rect2[2] && rect2[0] < rect[2] && rect[1] < rect2[3] && rect2[1] < rect[3]
    }

    pub fn get_bb(&self) -> [f64; 4]{
        let size = if let Enemy::ShieldedBoss(boss) = self { boss.shield_health as f64 } else { ENEMY_SIZE };
        let e = self.get_base();
        [e.pos[0] - size, e.pos[1] - size, e.pos[0] + size, e.pos[1] + size]
    }

    pub fn is_boss(&self) -> bool {
        match self {
            Enemy::Boss(_) | Enemy::ShieldedBoss(_) => true,
            _ => false
        }
    }
}

pub struct BulletBase(pub Entity);

pub enum Projectile{
    Bullet(BulletBase),
    EnemyBullet(BulletBase),
    Missile{base: BulletBase, target: u32, trail: Vec<[f64; 2]>}
}

const MISSILE_DETECTION_RANGE: f64 = 256.;
const MISSILE_HOMING_SPEED: f64 = 0.25;
const MISSILE_TRAIL_LENGTH: usize = 20;

impl Projectile{
    pub fn get_base<'b>(&'b self) -> &'b BulletBase{
        match &self {
            &Projectile::Bullet(base) | &Projectile::EnemyBullet(base) => base,
            &Projectile::Missile{base, target: _, trail: _} => base
        }
    }
    // pub fn get_base_mut(&'a mut self) -> &'a mut BulletBase{
    //     match &mut self {
    //         &mut Projectile::Bullet(base) | &mut Projectile::Bullet(base) => &mut base,
    //         &mut Projectile::Missile(base, _) => &mut base
    //     }
    // }

    pub fn get_type(&self) -> &str{
        match &self{
            &Projectile::Bullet(_) | &Projectile::EnemyBullet(_) => "Bullet",
            &Projectile::Missile{..} => "Missile"
        }
    }

    fn animate_player_bullet(mut base: &mut BulletBase, enemies: &mut Vec<Enemy>, mut _player: &mut Entity) -> Option<DeathReason>{
        let bbox = Self::get_bb_base(base);
        let &mut BulletBase(ent) = &mut base;
        for enemy in enemies.iter_mut() {
            if enemy.test_hit(bbox) {
                enemy.damage(ent.health);
                ent.health = 0;
                break;
            }
        }
        ent.animate()
    }

    fn animate_enemy_bullet(mut base: &mut BulletBase, _enemies: &mut Vec<Enemy>, mut player: &mut Entity) -> Option<DeathReason>{
        let &mut BulletBase(ent) = &mut base;
        if let Some(death_reason) = ent.hits_player(player) {
            player.health -= ent.health;
            return Some(death_reason)
        }
        ent.animate()
    }

    pub fn animate_bullet(&mut self, enemies: &mut Vec<Enemy>, player: &mut Entity) -> Option<DeathReason>{
        match self {
            Projectile::Bullet(base) => {
                Self::animate_player_bullet(base, enemies, player)
            },
            Projectile::EnemyBullet(base) => {
                Self::animate_enemy_bullet(base, enemies, player)
            },
            Projectile::Missile{base, target, trail} => {
                if *target == 0 {
                    let best = enemies.iter().fold((0, 1e5), |bestpair, enemy| {
                        let e = enemy.get_base();
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
                else if let Some(target_enemy) = enemies.iter().find(|e| e.get_id() == *target) {
                    let target_ent = target_enemy.get_base();
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
                Self::animate_player_bullet(base, enemies, player)
            }
        }
    }

    pub fn get_bb_base(base: &BulletBase) -> [f64; 4]{
        let e = &base.0;
        [e.pos[0] - BULLET_SIZE, e.pos[1] - BULLET_SIZE, e.pos[0] + BULLET_SIZE, e.pos[1] + BULLET_SIZE]
    }

    #[allow(dead_code)]
    pub fn get_bb(&self) -> [f64; 4]{
        let e = self.get_base();
        Self::get_bb_base(e)
    }

    pub fn draw(&self, c: &Context, g: &mut G2d, assets: &Assets){
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
        self.get_base().0.draw_tex(c, g, match self {
            Projectile::Bullet(_) => &assets.bullet_tex,
            Projectile::EnemyBullet(_) => &assets.ebullet_tex,
            Projectile::Missile{..} => &assets.missile_tex,
        });
    }
}


pub enum Item{
    PowerUp(Entity),
    PowerUp10(Entity)
}

impl Item{
    pub fn get_base(&self) -> &Entity{
        match self {
            Item::PowerUp(ent) | Item::PowerUp10(ent) => ent,
        }
    }

    pub fn draw(&self, c: &Context, g: &mut G2d, assets: &Assets){
        match self {
            Item::PowerUp(item) => item.draw_tex(c, g, &assets.power_tex),
            Item::PowerUp10(item) => item.draw_tex(c, g, &assets.power2_tex)
        }
    }

    pub fn power_value(&self) -> u32 {
        match self {
            Item::PowerUp(_) => 1,
            Item::PowerUp10(_) => 10,
        }
    }

    pub fn animate(&mut self, player: &mut Player) -> Option<DeathReason> {
        match self {
            Item::PowerUp(ent) | Item::PowerUp10(ent) => {
                if let Some(_) = ent.hits_player(&player.base) {
                    player.power += self.power_value();
                    return Some(DeathReason::Killed)
                }
                ent.animate()
            },
        }
    }
}



pub struct TempEntity<'a>{
    pub base: Entity,
    pub texture: &'a G2dTexture,
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
        let tex2 = self.texture;
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

