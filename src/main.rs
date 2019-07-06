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
    Matrix,
    DeathReason,
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

    let mut load_texture = |name| {
        Texture::from_path(
            &mut window.factory,
            &assets.join(name),
            Flip::None,
            &TextureSettings::new()
        ).unwrap()
    };

    let bg = load_texture("bg.png");
    let player_tex = load_texture("player.png");
    let boss_tex = load_texture("boss.png");
    let enemy_tex = load_texture("enemy.png");
    let ebullet_tex = load_texture("ebullet.png");
    let bullet_tex = load_texture("bullet.png");
    let missile_tex = load_texture("missile.png");
    let explode_tex = load_texture("explode.png");
    let explode2_tex = load_texture("explode2.png");
    let weapons_tex = load_texture("weapons.png");
    let sphere_tex = load_texture("sphere.png");

    let mut id_gen = 0;
    let mut player = Enemy::new(&mut id_gen, [240., 400.], [0., 0.], &player_tex);

    let mut enemies = Vec::<Enemy>::new();

    let mut bullets = Vec::<Projectile>::new();

    let mut tent = Vec::<TempEntity>::new();

    let mut rng = thread_rng();

    let mut kills = 0;
    let [mut shots_bullet, mut shots_missile] = [0, 0];

    let ref font = assets.join("FiraSans-Regular.ttf");
    let factory = window.factory.clone();
    let mut glyphs = Glyphs::new(font, factory, TextureSettings::new()).unwrap();


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

    #[derive(PartialEq, Clone)]
    enum Weapon{
        Bullet,
        Light,
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

            let shoot_period = if let Weapon::Bullet = weapon { 5 } else { 25 };

            // id_gen and rng must be passed as arguments since they are mutable
            // borrows and needs to be released for each iteration.
            // These variables are used in between multiple invocation of this closure.
            let mut add_tent = |is_bullet, pos: &[f64; 2], id_gen: &mut u32, rng: &mut ThreadRng| {
                let mut ent = Entity::new(
                    id_gen,
                    [
                        pos[0] + 4. * (rng.gen::<f64>() - 0.5),
                        pos[1] + 4. * (rng.gen::<f64>() - 0.5)
                    ], [0., 0.], if is_bullet { &explode_tex } else { &explode2_tex })
                    .rotation(rng.gen::<f32>() * 2. * std::f32::consts::PI)
                    ;
                if is_bullet {
                    ent = ent.health((MAX_FRAMES * PLAYBACK_RATE) as i32);
                }
                else{
                    ent = ent.health((MAX_FRAMES2 * PLAYBACK_RATE) as i32);
                }

                tent.push(TempEntity{base: ent,
                    max_frames: if is_bullet { MAX_FRAMES } else { MAX_FRAMES2 },
                    width: if is_bullet { 16 } else { 32 }})
            };

            if Weapon::Bullet == weapon || Weapon::Missile == weapon {
                if key_shoot && time % shoot_period == 0 {
                    for i in -1..2 {
                        let speed = if let Weapon::Bullet = weapon { BULLET_SPEED } else { MISSILE_SPEED };
                        let mut ent = Entity::new(&mut id_gen, player.pos, [i as f64, -speed], if let Weapon::Bullet = weapon { &bullet_tex } else { &missile_tex })
                            .rotation((i as f32).atan2(speed as f32));
                        if let Weapon::Bullet = weapon {
                            shots_bullet += 1;
                            ent = ent.blend(Blend::Add);
                            bullets.push(Projectile::Bullet(BulletBase(ent, true)))
                        }
                        else{
                            shots_missile += 1;
                            ent = ent.health(5);
                            bullets.push(Projectile::Missile{base: BulletBase(ent, true), target: 0, trail: vec!()})
                        }
                    }
                }
            }
            else if Weapon::Light == weapon && key_shoot {
                // Apparently Piston doesn't allow vertex colored rectangle, we need to 
                // draw multiple lines in order to display gradual change in color.
                for i in -3..4 {
                    let f = (4. - (i as i32).abs() as f32) / 4.;
                    line([f / 3., 0.5 + f / 2., 1., f],
                        1.,
                        [player.pos[0] + i as f64, player.pos[1],
                         player.pos[0] + i as f64, 0.],
                        context.transform, graphics);
                }
                for e in &mut (&mut enemies).iter_mut() {
                    if player.pos[0] - LIGHT_WIDTH < e.pos[0] + ENEMY_SIZE && e.pos[0] - ENEMY_SIZE < player.pos[0] + LIGHT_WIDTH &&
                        /*player.pos[1] < e.pos[1] + ENEMY_SIZE &&*/ e.pos[1] - ENEMY_SIZE < player.pos[1] {
                        e.health -= 1;
                        add_tent(true, &e.pos, &mut id_gen, &mut rng);
                    }
                }
            }

            player.draw_tex(&context, graphics);

            time += 1;

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
                if let Some(death_reason) = e.animate() {
                    to_delete.push(i);
                    if let DeathReason::Killed = death_reason {
                        kills += 1;
                    }
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
                if let Some(death_reason) = b.animate_bullet(&mut enemies) {
                    to_delete.push(i);

                    let base = b.get_base();

                    if let DeathReason::Killed = death_reason {
                        add_tent(if let Projectile::Bullet(_) = b { true } else { false }, &base.0.pos, &mut id_gen, &mut rng);
                    }
                }

                b.draw(&context, graphics);
            }

            for i in to_delete.iter().rev() {
                let b = bullets.remove(*i);
                println!("Deleted {} id={}, {} / {}", b.get_type(), b.get_base().0.id, *i, bullets.len());
            }

            to_delete.clear();

            for (i, e) in &mut ((&mut tent).iter_mut().enumerate()) {
                if let Some(_) = e.animate_temp() {
                    to_delete.push(i);
                    continue;
                }
                e.draw_temp(&context, graphics);
            }

            for i in to_delete.iter().rev() {
                tent.remove(*i);
                //println!("Deleted tent {} / {}", *i, bullets.len());
            }

            // Right side bar
            rectangle([0.20, 0.20, 0.4, 1.],
                [WIDTH as f64, 0., (WINDOW_WIDTH - WIDTH) as f64, WINDOW_HEIGHT as f64],
                context.transform, graphics);

            let mut draw_text = |s: &str, line: i32| {
                text::Text::new_color([0.0, 1.0, 0.0, 1.0], 12).draw(
                    s,
                    &mut glyphs,
                    &context.draw_state,
                    context.transform.trans(WIDTH as f64, (line + 1) as f64 * 12.0),
                    graphics
                ).unwrap_or_default();
            };

            draw_text(&format!("Frame: {}", time), 0);
            draw_text(&format!("Kills: {}", kills), 1);
            draw_text(&format!("shots_bullet: {}", shots_bullet), 2);
            draw_text(&format!("shots_missile: {}", shots_missile), 3);

            // Display weapon selection
            use piston_window::math::translate;
            let weapon_set = [(0, Weapon::Bullet), (2, Weapon::Light), (3, Weapon::Missile)];
            let centerize = translate([-((sphere_tex.get_width() * weapon_set.len() as u32) as f64 / 2.), -(sphere_tex.get_height() as f64 / 2.)]);
            for (i,v) in weapon_set.iter().enumerate() {
                let sphere_image = if v.1 == weapon { Image::new() } else { Image::new_color([0.5, 0.5, 0.5, 1.]) };
                let transl = translate([((WINDOW_WIDTH + WIDTH) / 2 + i as u32 * 32) as f64, (WINDOW_HEIGHT * 3 / 4) as f64]);
                let transform = (Matrix(context.transform) * Matrix(transl) * Matrix(centerize)).0;
                sphere_image.draw(&sphere_tex, &context.draw_state, transform, graphics);
                let weapons_image = sphere_image.src_rect([v.0 as f64 * 32., 0., 32., weapons_tex.get_height() as f64]);
                weapons_image.draw(&weapons_tex, &context.draw_state, transform, graphics);
            }
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
                        Key::Z | Key::X => {
                            if !key_change && tf {
                                use Weapon::*;
                                let weapon_set = [("Bullet", Bullet), ("Light", Light), ("Missile", Missile)];
                                let (name, next_weapon) = match weapon {
                                    Bullet => if key == Key::X { &weapon_set[1] } else { &weapon_set[2] },
                                    Light => if key == Key::X { &weapon_set[2] } else { &weapon_set[0] },
                                    Missile => if key == Key::X { &weapon_set[0] } else { &weapon_set[1] },
                                };
                                weapon = next_weapon.clone();
                                println!("Weapon switched: {}", name);
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