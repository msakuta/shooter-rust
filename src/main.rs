extern crate piston_window;
extern crate find_folder;
extern crate vecmath;
extern crate glutin_window;
extern crate gfx_graphics;
extern crate gfx_device_gl;
extern crate rand;

use std::rc::Rc;
use piston_window::*;
use rand::prelude::*;

struct Enemy{
    pos: [f64; 2],
    velo: [f64; 2],
    texture: Rc<G2dTexture>
}

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

impl Enemy{
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
        let mut tran: math::Matrix2d = context.transform;
        tran[0][2] = (pos[0] as f64) / WIDTH as f64;
        tran[1][2] = (pos[1] as f64) / HEIGHT as f64;
        let tex2 = self.texture.as_ref();
        image(tex2, tran, g);
    }
}

fn main() {
    use rand::Rng;
    //use glutin_window::GlutinWindow;
    //let (WIDTH, HEIGHT) = (640, 480);
    let mut time = 0;
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow  =
        WindowSettings::new("Hello Piston!", [WIDTH, HEIGHT])
        .exit_on_esc(true).opengl(opengl).build().unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let rust_logo = assets.join("boss.bmp");
    let bg = Texture::from_path(
            &mut window.factory,
            &assets.join("bg.bmp"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let player_tex = Rc::new(Texture::from_path(
            &mut window.factory,
            &assets.join("player.bmp"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap());
    let enemy_tex = Rc::new(Texture::from_path(
            &mut window.factory,
            &rust_logo,
            Flip::None,
            &TextureSettings::new()
        ).unwrap());
    let bullet_tex = Rc::new(Texture::from_path(
            &mut window.factory,
            &assets.join("ebullet.bmp"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap());
    let player = Enemy{pos: [0., 100.], velo: [0., 0.], texture: player_tex.clone()};

    let mut enemies = vec!{
        Enemy{pos: [135., 312.], velo: [0f64, 0f64], texture: enemy_tex.clone()},
        Enemy{pos: [564., 152.], velo: [1f64, 0f64], texture: enemy_tex.clone()},
        Enemy{pos: [64., 202.], velo: [1f64, 0f64], texture: enemy_tex.clone()},
        Enemy{pos: [314., 102.], velo: [1f64, 1f64], texture: enemy_tex.clone()}
    };

    let mut bullets = Vec::<Enemy>::new();

    let mut rng = thread_rng();

    while let Some(event) = window.next() {

        window.draw_2d(&event, |context, graphics| {
            clear([0.0, 0., 0., 1.], graphics);

            image(&bg, context.transform, graphics);

            player.draw_tex(&context, graphics);

            time = (time + 1) % 100;

            if rng.gen_range(0, 100) < 1 {
                enemies.push(Enemy{
                    pos: [rng.gen_range(0., WIDTH as f64), rng.gen_range(0., HEIGHT as f64)],
                    velo: [rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5],
                    texture: enemy_tex.clone()
                })
            }

            let mut to_delete: Vec<usize> = Vec::new();
            for (i, e) in &mut ((&mut enemies).iter_mut().enumerate()) {
                if !e.animate() {
                    to_delete.push(i);
                    continue;
                }
                e.draw_tex(&context, graphics);

                let x: i32 = rng.gen_range(0, 100);
                if x < 2 {
                    bullets.push(Enemy{
                        pos: e.pos,
                        velo: [rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5],
                        texture: bullet_tex.clone()
                    })
                }
            }

            for i in to_delete.iter().rev() {
                enemies.remove(*i);
                println!("Deleted Enemy {} / {}", *i, enemies.len());
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