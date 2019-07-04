extern crate piston_window;
extern crate find_folder;
extern crate vecmath;
extern crate gfx_graphics;
extern crate gfx_device_gl;
extern crate rand;

use piston_window::*;
use rand::prelude::*;
use piston_window::draw_state::Blend;

mod consts;
mod entity;

use consts::*;
use crate::entity::{
    Entity,
    Enemy,
    BulletBase,
    Projectile,
    TempEntity,
    MAX_FRAMES, MAX_FRAMES2, PLAYBACK_RATE};

const PLAYER_SPEED: f64 = 2.;
const PLAYER_SIZE: f64 = 16.;



fn main() {
    use rand::Rng;
    let mut time = 0;
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("Shooter Rust", [WINDOW_WIDTH, WINDOW_HEIGHT])
        .exit_on_esc(true).opengl(opengl).build().unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    let bg = Texture::from_path(
            &mut window.factory,
            &assets.join("bg.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let player_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("player.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let boss_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("boss.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let enemy_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("enemy.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let ebullet_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("ebullet.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let bullet_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("bullet.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let missile_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("missile.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let explode_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("explode.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let explode2_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("explode2.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();

    let mut id_gen = 0;
    let mut player = Enemy::new(&mut id_gen, [240., 400.], [0., 0.], &player_tex);

    let mut enemies = Vec::<Enemy>::new();

    let mut bullets = Vec::<Projectile>::new();

    let mut tent = Vec::<TempEntity>::new();

    let mut rng = thread_rng();

    fn limit_viewport(viewport: &Viewport, ratio: f64, wwidth: u32, wheight: u32) -> Viewport{
        let vp_ratio = (viewport.rect[2] - viewport.rect[0]) as f64 /
            (viewport.rect[3] - viewport.rect[0]) as f64;
        let mut newvp = *viewport;
        newvp.window_size[0] = (wwidth as f64 * (vp_ratio / ratio).max(1.)) as u32;
        newvp.window_size[1] = (wheight as f64 * (ratio / vp_ratio).max(1.)) as u32;
        #[cfg(debug)]
        for (vp, name) in [(viewport, "Old"), (&newvp, "New")].iter() {
            println!("{} Context: ratio: {} vp.rect: {:?} vp.draw: {:?} vp.window: {:?}",
                name, ratio, vp.rect, vp.draw_size, vp.window_size);
        }
        newvp
    }

    let [mut key_up, mut key_down, mut key_left, mut key_right, mut key_shoot, mut key_change] = [false; 6];
    enum Weapon{
        Bullet,
        Missile
    }
    let mut weapon = Weapon::Bullet;

    while let Some(event) = window.next() {

        if let Some(_) = event.render_args() {
        window.draw_2d(&event, |mut context, graphics| {
            clear([0.0, 0., 0., 1.], graphics);

            if let Some(viewport) = context.viewport {
                let (fwidth, fheight) = (WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
                let ratio = fwidth / fheight;

                let wnd_context = Context::new_viewport(limit_viewport(&viewport, ratio, WINDOW_WIDTH, WINDOW_HEIGHT));

                wnd_context.trans(-1., -1.);

                image(&bg, wnd_context.transform, graphics);

                context = Context::new_viewport(limit_viewport(&viewport, ratio, WINDOW_WIDTH, WINDOW_HEIGHT));
            }

            if key_up && PLAYER_SIZE <= player.pos[1] - PLAYER_SPEED {
                player.pos[1] -= PLAYER_SPEED;
            }
            else if key_down && player.pos[1] + PLAYER_SPEED < HEIGHT as f64 - PLAYER_SIZE {
                player.pos[1] += PLAYER_SPEED;
            }
            if key_left && PLAYER_SIZE <= player.pos[0] - PLAYER_SPEED {
                player.pos[0] -= PLAYER_SPEED;
            }
            else if key_right && player.pos[0] + PLAYER_SPEED < WIDTH as f64 - PLAYER_SIZE {
                player.pos[0] += PLAYER_SPEED;
            }

            let shoot_period = if let Weapon::Bullet = weapon { 5 } else { 15 };

            if key_shoot && time % shoot_period == 0 {
                for i in -1..2 {
                    let speed = if let Weapon::Bullet = weapon { BULLET_SPEED } else { MISSILE_SPEED };
                    let mut ent = Entity::new(&mut id_gen, player.pos, [i as f64, -speed], if let Weapon::Bullet = weapon { &bullet_tex } else { &missile_tex })
                        .rotation((i as f32).atan2(speed as f32));
                    if let Weapon::Bullet = weapon {
                        ent = ent.blend(Blend::Add);
                        bullets.push(Projectile::Bullet(BulletBase(ent, true)))
                    }
                    else{
                        ent = ent.health(5);
                        bullets.push(Projectile::Missile(BulletBase(ent, true), 0))
                    }
                }
            }

            player.draw_tex(&context, graphics);

            time = (time + 1) % 100;

            if rng.gen_range(0, 100) < 1 {
                let boss = rng.gen_range(0, 100) < 20;
                enemies.push(Enemy::new(
                    &mut id_gen,
                    [rng.gen_range(0., WIDTH as f64), rng.gen_range(0., HEIGHT as f64)],
                    [rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5],
                    if boss { &boss_tex } else { &enemy_tex })
                    .health(if boss { 64 } else { 3 })
                )
            }

            let mut to_delete: Vec<usize> = Vec::new();
            for (i, e) in &mut ((&mut enemies).iter_mut().enumerate()) {
                if !e.animate() {
                    to_delete.push(i);
                    continue;
                }
                e.draw_tex(&context, graphics);

                let x: i32 = rng.gen_range(0, if e.texture == &boss_tex { 16 } else { 64 });
                if x == 0 {
                    bullets.push(Projectile::Bullet(BulletBase(Entity::new(
                        &mut id_gen,
                        e.pos,
                        [rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5],
                        &ebullet_tex)
                    , false)))
                }
            }

            for i in to_delete.iter().rev() {
                let dead = enemies.remove(*i);
                println!("Deleted Enemy {} id={}: {} / {}", if dead.texture == &boss_tex { "boss" } else {"enemy"}, dead.id, *i, enemies.len());
            }

            to_delete.clear();

            for (i,b) in &mut bullets.iter_mut().enumerate() {
                if !b.animate_bullet(&mut enemies){
                    to_delete.push(i);

                    let base = b.get_base();

                    let mut ent = Entity::new(
                        &mut id_gen,
                        [
                            base.0.pos[0] + 4. * (rng.gen::<f64>() - 0.5),
                            base.0.pos[1] + 4. * (rng.gen::<f64>() - 0.5)
                        ], [0., 0.], if let Projectile::Bullet(_) = b { &explode_tex } else { &explode2_tex })
                        .rotation(rng.gen::<f32>() * 2. * std::f32::consts::PI)
                        ;
                    if let Projectile::Bullet(_) = b {
                        ent = ent.health((MAX_FRAMES * PLAYBACK_RATE) as i32);
                    }
                    else{
                        ent = ent.health((MAX_FRAMES2 * PLAYBACK_RATE) as i32);
                    }

                    tent.push(TempEntity{base: ent,
                        max_frames: if let Projectile::Bullet(_) = b { MAX_FRAMES } else { MAX_FRAMES2 },
                        width: if let Projectile::Bullet(_) = b { 16 } else { 32 }})
                }

                b.get_base().0.draw_tex(&context, graphics);
            }

            for i in to_delete.iter().rev() {
                let b = bullets.remove(*i);
                println!("Deleted bullet id={}, {} / {}", b.get_base().0.id, *i, bullets.len());
            }

            to_delete.clear();

            for (i, e) in &mut ((&mut tent).iter_mut().enumerate()) {
                if !e.animate_temp() {
                    to_delete.push(i);
                    continue;
                }
                e.draw_temp(&context, graphics);
            }

            for i in to_delete.iter().rev() {
                tent.remove(*i);
                //println!("Deleted tent {} / {}", *i, bullets.len());
            }

            //print!("time: {}, tran: {:?}\n", time, tran);
            //scene.draw(context.transform, graphics);
        });
        }
        // else if let Some(pos) = event.mouse_cursor_args() {
        //     player.pos = pos;
        // }
        else{
            let mut toggle_key = |opt: Option<Button>, tf: bool| {
                if let Some(Button::Keyboard(key)) = opt {
                    match key {
                        Key::Up | Key::W => key_up = tf,
                        Key::Down | Key::S => key_down = tf,
                        Key::Left | Key::A => key_left = tf,
                        Key::Right | Key::D => key_right = tf,
                        Key::C => key_shoot = tf,
                        Key::Z => {
                            if !key_change && tf {
                                weapon = if let Weapon::Bullet = weapon { Weapon::Missile } else { Weapon::Bullet };
                                println!("Weapon switched: {}", if let Weapon::Bullet = weapon { "Bullet" } else { "Missile" });
                            }
                            key_change = tf;
                        },
                        _ => {}
                    }
                }
            };
            toggle_key(event.press_args(), true);
            toggle_key(event.release_args(), false);
        }

    }
}