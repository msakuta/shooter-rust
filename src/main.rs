extern crate piston_window;
extern crate find_folder;
extern crate vecmath;
extern crate glutin_window;
extern crate gfx_graphics;
extern crate gfx_device_gl;
extern crate rand;

use piston_window::*;
use rand::prelude::*;

struct Enemy<'a>{
    pos: [f64; 2],
    velo: [f64; 2],
    texture: &'a G2dTexture
}

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;
const WIDTH: u32 = WINDOW_WIDTH * 3 / 4;
const HEIGHT: u32 = WINDOW_HEIGHT;

impl<'a> Enemy<'a>{
    fn animate(&mut self) -> bool{
        let pos = &mut self.pos;
        for i in 0..2 {
            pos[i] = pos[i] + self.velo[i];
        }
        if pos[0] < 0. || (WIDTH as f64) < pos[0] || pos[1] < 0. || (HEIGHT as f64) < pos[1] {
            false
        }
        else{
            true
        }
    }

    fn draw_tex(&self, context: &Context, g: &mut G2d){
        let pos = &self.pos;
        let mut mytran = vecmath::mat2x3_id();
        mytran[0][2] = pos[0];
        mytran[1][2] = pos[1];
        let tex2 = self.texture;
        image(tex2, vecmath::row_mat2x3_mul(context.transform, mytran), g);
    }
}

fn main() {
    use rand::Rng;
    //use glutin_window::GlutinWindow;
    //let (WIDTH, HEIGHT) = (640, 480);
    let mut time = 0;
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow  =
        WindowSettings::new("Hello Piston!", [WINDOW_WIDTH, WINDOW_HEIGHT])
        .exit_on_esc(true).opengl(opengl).build().unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    let bg = Texture::from_path(
            &mut window.factory,
            &assets.join("bg.bmp"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let player_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("player.bmp"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let boss_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("boss.bmp"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let enemy_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("enemy.bmp"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let bullet_tex = Texture::from_path(
            &mut window.factory,
            &assets.join("ebullet.bmp"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let player = Enemy{pos: [10., 100.], velo: [0., 0.], texture: &player_tex};

    let mut enemies = vec!{
        Enemy{pos: [135., 312.], velo: [0f64, 0f64], texture: &enemy_tex},
        Enemy{pos: [564., 152.], velo: [1f64, 0f64], texture: &boss_tex},
        Enemy{pos: [64., 202.], velo: [1f64, 0f64], texture: &enemy_tex},
        Enemy{pos: [314., 102.], velo: [1f64, 1f64], texture: &enemy_tex}
    };

    let mut bullets = Vec::<Enemy>::new();

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

    while let Some(event) = window.next() {

        window.draw_2d(&event, |mut context, graphics| {
            clear([0.0, 0., 0., 1.], graphics);

            if let Some(viewport) = context.viewport {
                let (fwidth, fheight) = (WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
                let ratio = fwidth / fheight;

                //let [fwidth, fheight] = [WINDOW_WIDTH as f64, HEIGHT as f64];
                let wnd_context = Context::new_viewport(limit_viewport(&viewport, ratio, WINDOW_WIDTH, WINDOW_HEIGHT));

                wnd_context.trans(-1., -1.);

                image(&bg, wnd_context.transform, graphics);

                context = Context::new_viewport(limit_viewport(&viewport, ratio, WINDOW_WIDTH, WINDOW_HEIGHT));
            }

            player.draw_tex(&context, graphics);

            time = (time + 1) % 100;

            if rng.gen_range(0, 100) < 1 {
                enemies.push(Enemy{
                    pos: [rng.gen_range(0., WIDTH as f64), rng.gen_range(0., HEIGHT as f64)],
                    velo: [rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5],
                    texture: if rng.gen_range(0, 100) < 20 { &boss_tex } else { &enemy_tex }
                })
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
                    bullets.push(Enemy{
                        pos: e.pos,
                        velo: [rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5],
                        texture: &bullet_tex
                    })
                }
            }

            for i in to_delete.iter().rev() {
                let dead = enemies.remove(*i);
                println!("Deleted Enemy {} {} / {}", if dead.texture == &boss_tex { "boss" } else {"enemy"}, *i, enemies.len());
            }

            to_delete.clear();

            for (i,b) in &mut bullets.iter_mut().enumerate() {
                if !b.animate(){
                    to_delete.push(i);
                }
                b.draw_tex(&context, graphics);
            }

            for i in to_delete.iter().rev() {
                bullets.remove(*i);
                println!("Deleted bullet {} / {}", *i, bullets.len());
            }

            //print!("time: {}, tran: {:?}\n", time, tran);
            //scene.draw(context.transform, graphics);
        });
    }
}