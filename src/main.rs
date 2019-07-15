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
    Player,
    Enemy,
    ShieldedBoss,
    BulletBase,
    Projectile,
    TempEntity};




fn main() {
    use rand::Rng;
    let mut time = 0;
    let mut disptime = 0;
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
    let shield_tex = load_texture("shield.png");
    let ebullet_tex = load_texture("ebullet.png");
    let bullet_tex = load_texture("bullet.png");
    let missile_tex = load_texture("missile.png");
    let explode_tex = load_texture("explode.png");
    let explode2_tex = load_texture("explode2.png");
    let weapons_tex = load_texture("weapons.png");
    let sphere_tex = load_texture("sphere.png");
    let power_tex = load_texture("power.png");
    let power2_tex = load_texture("power2.png");

    let mut id_gen = 0;
    let mut player = Player::new(Entity::new(&mut id_gen, [240., 400.], [0., 0.], &player_tex));

    let mut enemies = Vec::<Enemy>::new();

    let mut items = Vec::<Entity>::new();

    let mut bullets = Vec::<Projectile>::new();

    let mut tent = Vec::<TempEntity>::new();

    let mut rng = thread_rng();

    let mut paused = false;
    let mut game_over = true;

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

    let [mut key_up, mut key_down, mut key_left, mut key_right, mut key_shoot,
        mut key_change, mut key_pause] = [false; 7];

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
                let (playback_rate, max_frames) = if is_bullet { (2, 8) } else { (4, 6) };
                ent = ent.health((max_frames * playback_rate) as i32);

                tent.push(TempEntity{base: ent,
                    max_frames,
                    width: if is_bullet { 16 } else { 32 },
                    playback_rate})
            };

            if !game_over && !paused {
                if key_up { player.move_up() }
                if key_down { player.move_down() }
                if key_left { player.move_left() }
                if key_right { player.move_right() }

                let shoot_period = if let Weapon::Bullet = weapon { 5 } else { 50 };

                if Weapon::Bullet == weapon || Weapon::Missile == weapon {
                    if key_shoot && time % shoot_period == 0 {
                        let level = player.power_level() as i32;
                        for i in -1-level..2+level {
                            let speed = if let Weapon::Bullet = weapon { BULLET_SPEED } else { MISSILE_SPEED };
                            let mut ent = Entity::new(&mut id_gen, player.base.pos, [i as f64, -speed], if let Weapon::Bullet = weapon { &bullet_tex } else { &missile_tex })
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
                            [player.base.pos[0] + i as f64, player.base.pos[1],
                            player.base.pos[0] + i as f64, 0.],
                            context.transform, graphics);
                    }
                    for enemy in enemies.iter_mut() {
                        if enemy.test_hit([player.base.pos[0] - LIGHT_WIDTH, 0., player.base.pos[0] + LIGHT_WIDTH, player.base.pos[1]]) {
                            add_tent(true, &enemy.get_base().pos, &mut id_gen, &mut rng);
                            enemy.damage(1 + player.power_level() as i32);
                        }
                    }
                }

                if 0 < player.invtime {
                    player.invtime -= 1;
                }
            }

            if !game_over {
                if player.invtime == 0 || disptime % 2 == 0 {
                    player.base.draw_tex(&context, graphics);
                }
            }

            if !paused {
                time += 1;
            }
            disptime += 1;

            let mut to_delete: Vec<usize> = Vec::new();

            for (i, e) in &mut ((&mut items).iter_mut().enumerate()) {
                if !paused {
                    if let Some(_) = e.animate() {
                        to_delete.push(i);
                        continue;
                    }
                    if let Some(_) = e.hits_player(&player.base) {
                        to_delete.push(i);
                        player.power += if e.texture == &power_tex { 1 } else { 10 };
                        continue;
                    }
                }
                e.draw_tex(&context, graphics);
            }

            for i in to_delete.iter().rev() {
                let dead = items.remove(*i);
                println!("Deleted Item id={}: {} / {}", dead.id, *i, items.len());
            }
            to_delete.clear();

            let wave_period = 1024;
            if !paused {
                let dice = 256;
                let wave = time % wave_period;
                if wave < wave_period * 3 / 4 {
                    let (enemy_count, boss_count, shielded_boss_count) = enemies.iter().fold((0, 0, 0),
                        |c, e| match e {
                            Enemy::Enemy1(_) => (c.0 + 1, c.1, c.2),
                            Enemy::Boss(_) => (c.0, c.1 + 1, c.2),
                            Enemy::ShieldedBoss(_) => (c.0, c.1, c.2 + 1)
                        });
                    let gen_amount = player.difficulty_level() * 4 + 8;
                    let mut i = rng.gen_range(0, dice);
                    while i < gen_amount {
                        let weights = [
                            if enemy_count < 128 { if player.score < 1024 { 64 } else { 16 } } else { 0 },
                            if boss_count < 32 { 4 } else { 0 },
                            if shielded_boss_count < 32 { std::cmp::min(4, player.difficulty_level()) } else { 0 }];
                        let allweights = weights.iter().fold(0, |sum, x| sum + x);
                        let accum = {
                            let mut accum = [0; 3];
                            let mut accumulator = 0;
                            for (i,e) in weights.iter().enumerate() {
                                accumulator += e;
                                accum[i] = accumulator;
                            }
                            accum
                        };

                        if 0 < allweights {
                            let dice = rng.gen_range(0, allweights);
                            let (pos, velo) = match rng.gen_range(0, 3) {
                                0 => { // top
                                    ([rng.gen_range(0., WIDTH as f64), 0.], [rng.gen::<f64>() - 0.5, rng.gen::<f64>() * 0.5])
                                },
                                1 => { // left
                                    ([0., rng.gen_range(0., WIDTH as f64)], [rng.gen::<f64>() * 0.5, rng.gen::<f64>() - 0.5])
                                },
                                2 => { // right
                                    ([WIDTH as f64, rng.gen_range(0., WIDTH as f64)], [-rng.gen::<f64>() * 0.5, rng.gen::<f64>() - 0.5])
                                }
                                _ => panic!("RNG returned out of range")
                            };
                            enemies.push(if dice < accum[0] {
                                Enemy::Enemy1(Entity::new(&mut id_gen, pos, velo, &enemy_tex)
                                .health(3))
                            }
                            else if dice < accum[1] {
                                Enemy::Boss(Entity::new(&mut id_gen, pos, velo, &boss_tex)
                                .health(64))
                            }
                            else {
                                Enemy::ShieldedBoss(ShieldedBoss::new(
                                    &mut id_gen,
                                    pos,
                                    velo,
                                    &boss_tex,
                                    &shield_tex))
                            });
                        }
                        i += rng.gen_range(0, dice);
                    }
                }
            }

            for (i, enemy) in &mut ((&mut enemies).iter_mut().enumerate()) {
                if !paused {
                    let killed = {
                        if let Some(death_reason) = enemy.animate(time) {
                            to_delete.push(i);
                            if let DeathReason::Killed = death_reason {true} else{false}
                        }
                        else {false}
                    };
                    if killed {
                        player.kills += 1;
                        player.score += if enemy.is_boss() { 10 } else { 1 };
                        if rng.gen_range(0, 100) < 20 {
                            items.push(Entity::new(&mut id_gen, enemy.get_base().pos, [0., 1.],
                                if enemy.is_boss() { &power2_tex } else { &power_tex }));
                        }
                        continue;
                    }
                }
                enemy.draw(&context, graphics);

                let x: i32 = rng.gen_range(0, if enemy.is_boss() { 16 } else { 64 });
                if x == 0 {
                    bullets.push(Projectile::Bullet(BulletBase(Entity::new(
                        &mut id_gen,
                        enemy.get_base().pos,
                        [rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5],
                        &ebullet_tex)
                    , false)))
                }
            }

            for i in to_delete.iter().rev() {
                let dead = enemies.remove(*i);
                println!("Deleted Enemy {} id={}: {} / {}", match dead {
                    Enemy::Enemy1(_) => "enemy",
                    Enemy::Boss(_) => "boss",
                    Enemy::ShieldedBoss(_) => "ShieldedBoss"
                }, dead.get_id(), *i, enemies.len());
            }

            to_delete.clear();

            for (i,b) in &mut bullets.iter_mut().enumerate() {
                if !paused {
                    if let Some(death_reason) = b.animate_bullet(&mut enemies, &mut player.base) {
                        to_delete.push(i);

                        let base = b.get_base();

                        match death_reason {
                            DeathReason::Killed | DeathReason::HitPlayer =>
                                add_tent(if let Projectile::Bullet(_) = b { true } else { false }, &base.0.pos, &mut id_gen, &mut rng),
                            _ => {}
                        }

                        if let DeathReason::HitPlayer = death_reason {
                            if player.invtime == 0 && !game_over && 0 < player.lives {
                                player.lives -= 1;
                                if player.lives == 0 {
                                    game_over = true;
                                }
                                else{
                                    player.invtime = PLAYER_INVINCIBLE_TIME;
                                }
                            }
                        }
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
                if !paused {
                    if let Some(_) = e.animate_temp() {
                        to_delete.push(i);
                        continue;
                    }
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

            rectangle([0., 0.5, 0.4, 1.], [WIDTH as f64, (3) as f64 * 12.0 + 4., player.power as f64, 8.], context.transform, graphics);

            let mut draw_text_pos = |s: &str, pos: [f64; 2], color: [f32; 4], size: u32| {
                text::Text::new_color(color, size).draw(
                    s,
                    &mut glyphs,
                    &context.draw_state,
                    context.transform.trans(pos[0], pos[1]),
                    graphics
                ).unwrap_or_default();
            };

            if paused {
                draw_text_pos("PAUSED", [(WIDTH / 2 - 80) as f64, (HEIGHT / 2) as f64], [1.0, 1.0, 0.0, 1.0], 20);
            }

            if game_over {
                let color = [1.0, 1.0, 1.0, 1.0];
                draw_text_pos("GAME OVER", [(WIDTH / 2 - 80) as f64, (HEIGHT * 3 / 4) as f64], color, 20);
                draw_text_pos("Press Space to Start", [(WIDTH / 2 - 110) as f64, (HEIGHT * 3 / 4 + 20) as f64], color, 20);
            }

            let mut draw_text = |s: &str, line: i32| draw_text_pos(s, [WIDTH as f64, (line + 1) as f64 * 12.0], [0.0, 1.0, 0.0, 1.0], 12);

            draw_text(&format!("Frame: {}", time), 0);
            draw_text(&format!("Score: {}", player.score), 1);
            draw_text(&format!("Kills: {}", player.kills), 2);
            draw_text(&format!("Power: {}, Level: {}", player.power, player.power_level()), 3);
            draw_text(&format!("Wave: {} Level: {}", time / wave_period, player.difficulty_level()), 4);
            draw_text(&format!("shots_bullet: {}", shots_bullet), 5);
            draw_text(&format!("shots_missile: {}", shots_missile), 6);

            let weapon_set = [(0, Weapon::Bullet, [1.,0.5,0.]), (2, Weapon::Light, [1.,1.,1.]), (3, Weapon::Missile, [0.,1.,0.])];

            draw_text_pos("Z", [
                ((WINDOW_WIDTH + WIDTH) / 2 - weapon_set.len() as u32 * 32 / 2 - 16) as f64,
                (WINDOW_HEIGHT * 3 / 4) as f64],
                [1.0, 1.0, 0.0, 1.0], 14);
            draw_text_pos("X", [
                ((WINDOW_WIDTH + WIDTH) / 2 + weapon_set.len() as u32 * 32 / 2 + 16) as f64,
                (WINDOW_HEIGHT * 3 / 4) as f64],
                [1.0, 1.0, 0.0, 1.0], 14);

            // Display weapon selection
            use piston_window::math::translate;
            let centerize = translate([-((sphere_tex.get_width() * weapon_set.len() as u32) as f64 / 2.), -(sphere_tex.get_height() as f64 / 2.)]);
            for (i,v) in weapon_set.iter().enumerate() {
                let sphere_image = if v.1 == weapon {
                    Image::new_color([v.2[0], v.2[1], v.2[2], 1.])
                }
                else {
                    Image::new_color([0.5 * v.2[0], 0.5 * v.2[1], 0.5 * v.2[2], 1.])
                };
                let transl = translate([((WINDOW_WIDTH + WIDTH) / 2 + i as u32 * 32) as f64, (WINDOW_HEIGHT * 3 / 4) as f64]);
                let transform = (Matrix(context.transform) * Matrix(transl) * Matrix(centerize)).0;
                sphere_image.draw(&sphere_tex, &context.draw_state, transform, graphics);
                let weapons_image = sphere_image.color(if v.1 == weapon { [1., 1., 1., 1.] } else { [0.5, 0.5, 0.5, 1.] })
                .src_rect([v.0 as f64 * 32., 0., 32., weapons_tex.get_height() as f64]);
                weapons_image.draw(&weapons_tex, &context.draw_state, transform, graphics);
            }

            // Display player lives
            for i in 0..player.lives {
                let width = player_tex.get_width();
                let height = player_tex.get_height();
                let transl = translate([(WINDOW_WIDTH - (i + 1) as u32 * width) as f64, (WINDOW_HEIGHT - height) as f64]);
                let transform = (Matrix(context.transform) * Matrix(transl)).0;
                image(&player_tex, transform, graphics);
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
                            if !key_change && tf && !game_over {
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
                        Key::P => {
                            if !key_pause && tf {
                                paused = !paused;
                            }
                            key_pause = tf;
                        },
                        Key::Space => if tf {
                            items.clear();
                            enemies.clear();
                            bullets.clear();
                            tent.clear();
                            time = 0;
                            id_gen = 0;
                            player.reset();
                            shots_bullet = 0;
                            shots_missile = 0;
                            paused = false;
                            game_over = false;
                        },
                        Key::G =>
                            if cfg!(debug_assertions) && tf {
                                player.score += 1000;
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