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
use crate::entity::{Entity, Matrix};

const PLAYER_SPEED: f64 = 2.;
const PLAYER_SIZE: f64 = 16.;


type Enemy<'a> = Entity<'a>;

struct Bullet<'a>(Entity<'a>, bool);

const ENEMY_SIZE: f64 = 8.;
const BULLET_SIZE: f64 = 8.;

impl<'a> Bullet<'a>{
    fn animate_bullet(&mut self, enemies: &mut Vec<Enemy>) -> bool{
        let ent = &mut self.0;
        if self.1 {
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
}

type TempEntity<'a> = Entity<'a>;

const MAX_FRAMES: u32 = 8;
const PLAYBACK_RATE: u32 = 3;

impl<'a> TempEntity<'a>{
    fn animate_temp(&mut self) -> bool{
        self.health -= 1;
        self.animate()
    }

    fn draw_temp(&self, context: &Context, g: &mut G2d){
        let pos = &self.pos;
        let tex2 = self.texture;
        let mut centerize = vecmath::mat2x3_id();
        centerize[0][2] = -(16. / 2.);
        centerize[1][2] = -(tex2.get_height() as f64 / 2.);
        let mut mytran = vecmath::mat2x3_id();
        mytran[0][2] = pos[0];
        mytran[1][2] = pos[1];
        let frame = MAX_FRAMES - (self.health as u32 / PLAYBACK_RATE) as u32;
        let draw_state = if let Some(blend_mode) = self.blend { context.draw_state.blend(blend_mode) } else { context.draw_state };
        let image   = Image::new().rect([0f64, 0f64, 16., tex2.get_height() as f64])
            .src_rect([frame as f64 * 16., 0., 16., tex2.get_height() as f64]);
        image.draw(tex2, &draw_state, (Matrix(context.transform) * Matrix(mytran) * Matrix(centerize)).0, g);
    }
}

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
    let explode_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("explode.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let mut player = Enemy::new([240., 400.], [0., 0.], &player_tex);

    let mut enemies = Vec::<Enemy>::new();

    let mut bullets = Vec::<Bullet>::new();

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

    let [mut key_up, mut key_down, mut key_left, mut key_right, mut key_shoot] = [false; 5];

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

            if key_shoot && time % 3 == 0 {
                for i in -1..2 {
                    bullets.push(Bullet(
                        Entity::new(player.pos, [i as f64, -5.], &bullet_tex)
                        .blend(Blend::Add)
                        .rotation((i as f32).atan2(5.)),
                         true))
                }
            }

            player.draw_tex(&context, graphics);

            time = (time + 1) % 100;

            if rng.gen_range(0, 100) < 1 {
                let boss = rng.gen_range(0, 100) < 20;
                enemies.push(Enemy::new(
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
                    bullets.push(Bullet(Entity::new(
                        e.pos,
                        [rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5],
                        &ebullet_tex)
                    , false))
                }
            }

            for i in to_delete.iter().rev() {
                let dead = enemies.remove(*i);
                println!("Deleted Enemy {} {} / {}", if dead.texture == &boss_tex { "boss" } else {"enemy"}, *i, enemies.len());
            }

            to_delete.clear();

            for (i,b) in &mut bullets.iter_mut().enumerate() {
                if !b.animate_bullet(&mut enemies){
                    to_delete.push(i);
                    tent.push(
                        Entity::new(b.0.pos, [0., 0.], &explode_tex)
                        .health((MAX_FRAMES * PLAYBACK_RATE) as i32)
                    )
                }
                b.0.draw_tex(&context, graphics);
            }

            for i in to_delete.iter().rev() {
                bullets.remove(*i);
                //println!("Deleted bullet {} / {}", *i, bullets.len());
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
                println!("Deleted tent {} / {}", *i, bullets.len());
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
                        _ => {}
                    }
                }
            };
            toggle_key(event.press_args(), true);
            toggle_key(event.release_args(), false);
        }

    }
}