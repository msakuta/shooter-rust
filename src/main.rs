extern crate piston_window;
extern crate find_folder;
//extern crate sprite;
extern crate vecmath;
extern crate glutin_window;
extern crate gfx_graphics;
extern crate gfx_device_gl;

use std::rc::Rc;
use glutin_window::GlutinWindow;
use piston_window::*;
//use gfx_graphics;
//use gfx_device_gl;
//use sprite::*;

/*impl piston_window::GenericEvent for piston_window::Event{

}*/

type GWindow = PistonWindow<GlutinWindow>;

struct Enemy{
    pos: [f64; 2],
    velo: [f64; 2],
    texture: Rc<G2dTexture>
}

impl Enemy{
    fn animate(&mut self){
        let pos = &mut self.pos;
        for i in 0..2 {
            pos[i] = pos[i] + self.velo[i];
        }
    }

    fn draw/*<G: Graphics>*/(&self, context: &Context
        /*, graphics: &mut gfx_graphics::back_end::GfxGraphics<'_, gfx_device_gl::Resources, gfx_device_gl::command::CommandBuffer>*/) -> math::Matrix2d
        {
        let (width, height) = (640, 480);
        let pos = &self.pos;
        let mut tran: math::Matrix2d = context.transform;
        tran[0][2] = (pos[0] as f64) / width as f64;
        tran[1][2] = (pos[1] as f64) / height as f64;
        //image(self.texture.as_ref(), tran, graphics);
        return tran;
    }
}

fn main() {
    let (width, height) = (640, 480);
    let mut time = 0;
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [width, height])
        .exit_on_esc(true).opengl(opengl).build().unwrap();

    //let mut scene = Scene::new();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let rust_logo = assets.join("boss.bmp");
    let bg = Texture::from_path(
            &mut window.factory,
            &assets.join("bg.bmp"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap();
    let playerTex = Rc::new(Texture::from_path(
            &mut window.factory,
            &assets.join("player.bmp"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap());
    let tex = Rc::new(Texture::from_path(
            &mut window.factory,
            &rust_logo,
            Flip::None,
            &TextureSettings::new()
        ).unwrap());
    let mut player = Enemy{pos: [0., 100.], velo: [0., 0.], texture: playerTex};

    let mut enemies = vec!{
        Enemy{pos: [135., 312.], velo: [0f64, 0f64], texture: tex.clone()},
        Enemy{pos: [564., 152.], velo: [1f64, 0f64], texture: tex.clone()},
        Enemy{pos: [64., 202.], velo: [1f64, 0f64], texture: tex.clone()},
        Enemy{pos: [314., 102.], velo: [1f64, 1f64], texture: tex.clone()}
    };

    //let mut sprite = Sprite::from_texture(tex.clone());
    //sprite.set_position(width as f64 / 2.0, height as f64 / 2.0);

    //let id = scene.add_child(sprite);

    while let Some(event) = window.next() {
        //let gevt: &GenericEvent = &event as &GenericEvent;
        //scene.event(&event as &GenericEvent);

        window.draw_2d(&event, |context, graphics| {
            clear([0.0, 0., 0., 1.], graphics);

            image(&bg, context.transform, graphics);

            //let im: &mut gfx_graphics::back_end::GfxGraphics<'_, gfx_device_gl::Resources, gfx_device_gl::command::CommandBuffer> = graphics;
            image(player.texture.as_ref(), player.draw(&context/*, &mut graphics*/), graphics);

            time = (time + 1) % 100;
            for e in &mut enemies {
                e.animate();
                let tran = e.draw(&context/*, &mut graphics*/);
                image(e.texture.as_ref(), tran, graphics);
            }
            //print!("time: {}, tran: {:?}\n", time, tran);
            //scene.draw(context.transform, graphics);
        });
    }
}