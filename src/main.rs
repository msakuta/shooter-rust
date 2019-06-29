extern crate piston_window;
extern crate find_folder;
extern crate vecmath;
extern crate glutin_window;
extern crate gfx_graphics;
extern crate gfx_device_gl;
extern crate rand;

use piston_window::*;
use rand::prelude::*;
use rand::Rng;

struct Resources{
    enemy_tex: G2dTexture,
    boss_tex: G2dTexture,
    bullet_tex: G2dTexture,
}

impl Resources{
    fn new(window: &mut PistonWindow, assets: &std::path::PathBuf) -> Resources{
        Resources{boss_tex: Texture::from_path(
                &mut window.factory,
                &assets.join("boss.bmp"),
                Flip::None,
                &TextureSettings::new()
            ).unwrap(),
        enemy_tex: Texture::from_path(
                &mut window.factory,
                &assets.join("enemy.bmp"),
                Flip::None,
                &TextureSettings::new()
            ).unwrap(),
        bullet_tex: Texture::from_path(
                &mut window.factory,
                &assets.join("ebullet.bmp"),
                Flip::None,
                &TextureSettings::new()
            ).unwrap()
        }
    }
}

struct Game{
    rng: ThreadRng,
    enemies: Vec<Box<Entity>>,
    bullets: Vec<Box<Entity>>,
    resources: Resources
}

impl Game{
    fn animate(&mut self){
            if self.rng.gen_range(0, 100) < 1 {
                if self.rng.gen_range(0, 100) < 20 {
                    self.enemies.push(Enemy::new(
                        [self.rng.gen_range(0., WIDTH as f64), self.rng.gen_range(0., HEIGHT as f64)],
                        [self.rng.gen::<f64>() - 0.5, self.rng.gen::<f64>() - 0.5],
                        &self.resources
                    ))
                }
                else {
                    self.enemies.push(Boss::new(
                        [self.rng.gen_range(0., WIDTH as f64), self.rng.gen_range(0., HEIGHT as f64)],
                        [self.rng.gen::<f64>() - 0.5, self.rng.gen::<f64>() - 0.5],
                        &self.resources
                    ))
                }
            }
    }
}

trait Entity{
    fn getpos(&self) -> [f64; 2];
    fn setpos(&mut self, pos: [f64; 2]);
    fn getvelo(&self) -> [f64; 2];
    fn animate(&mut self) -> bool{
        let mut pos = self.getpos();
        for i in 0..2 {
            pos[i] = pos[i] + self.getvelo()[i];
        }
        if pos[0] < 0. || (WIDTH as f64) < pos[0] || pos[1] < 0. || (HEIGHT as f64) < pos[1] {
            false
        }
        else{
            self.setpos(pos);
            true
        }
    }
    fn draw(&self, context: &Context, g: &mut G2d);
    fn name(&self) -> &str;
}

struct EntityBase<'a>{
    pos: [f64; 2],
    velo: [f64; 2],
    texture: &'a G2dTexture
}

impl<'a> Entity for EntityBase<'a>{
    fn getpos(&self) -> [f64; 2]{ self.pos }
    fn setpos(&mut self, pos: [f64; 2]){ self.pos = pos; }
    fn getvelo(&self) -> [f64; 2]{ self.velo }
    fn draw(&self, context: &Context, g: &mut G2d){
        let pos = &self.pos;
        let mut tran: math::Matrix2d = context.transform;
        tran[0][2] = (pos[0] as f64) / WIDTH as f64;
        tran[1][2] = (pos[1] as f64) / HEIGHT as f64;
        let tex2 = self.texture;
        image(tex2, tran, g);
    }
    fn name(&self) -> &str{ "EntityBase" }
}

struct EntityTemp<'a>(EntityBase<'a>);

impl<'a> Entity for EntityTemp<'a>{
    fn getpos(&self) -> [f64; 2]{ self.0.getpos() }
    fn setpos(&mut self, pos: [f64; 2]){ self.0.setpos(pos); }
    fn getvelo(&self) -> [f64; 2]{ self.0.getvelo() }
    fn draw(&self, context: &Context, g: &mut G2d){ self.0.draw(context, g); }
    fn name(&self) -> &str{ "EntityTemp" }
}

/*impl<'a> Entity for EntityTemp<'a>{
    fn new(pos: [f64; 2], velo: [f64; 2], res: &Resources) -> Box<Bullet>{
        Box::new( EntityTemp::<'a>( EntityBase{ pos: pos, velo: velo, texture: Tag(&res) }) )
    }
}*/

struct Bullet<'a>(EntityBase<'a>);

impl<'a> Bullet<'a>{
    fn new(pos: [f64; 2], velo: [f64; 2], res: &Resources) -> Box<Bullet>{
        Box::new( Bullet( EntityBase{ pos: pos, velo: velo, texture: &res.bullet_tex }) )
    }
}

impl<'a> Entity for Bullet<'a>{
    fn getpos(&self) -> [f64; 2]{ self.0.getpos() }
    fn setpos(&mut self, pos: [f64; 2]){ self.0.setpos(pos); }
    fn getvelo(&self) -> [f64; 2]{ self.0.getvelo() }
    fn animate(&mut self) -> bool{
        //let mut pos = self.getpos();
        /*let x: i32 = game.rng.gen_range(0, 100);
        if x < 2 {
            game.bullets.push(Bullet::new (
                self.getpos(),
                [game.rng.gen::<f64>() - 0.5, game.rng.gen::<f64>() - 0.5],
                &game.resources
            ))
        }*/
        self.0.animate()
    }
    fn draw(&self, context: &Context, g: &mut G2d){ self.0.draw(context, g); }
    fn name(&self) -> &str{ "Bullet" }
}

struct Enemy<'a>(EntityBase<'a>);

impl<'a> Enemy<'a>{
    fn new(pos: [f64; 2], velo: [f64; 2], res: &Resources) -> Box<Enemy>{
        Box::new( Enemy( EntityBase{ pos: pos, velo: velo, texture: &res.enemy_tex }) )
    }
}

impl<'a> Entity for Enemy<'a>{
    fn getpos(&self) -> [f64; 2]{ self.0.getpos() }
    fn setpos(&mut self, pos: [f64; 2]){ self.0.setpos(pos); }
    fn getvelo(&self) -> [f64; 2]{ self.0.getvelo() }
    fn draw(&self, context: &Context, g: &mut G2d){ self.0.draw(context, g); }
    fn name(&self) -> &str{ "Enemy" }
}

struct Boss<'a>(EntityBase<'a>);

impl<'a> Boss<'a>{
    fn new(pos: [f64; 2], velo: [f64; 2], res: &Resources) -> Box<Boss>{
        Box::new( Boss(EntityBase{ pos: pos, velo: velo, texture: &res.boss_tex }) )
    }
}

impl<'a> Entity for Boss<'a>{
    fn getpos(&self) -> [f64; 2]{ self.0.getpos() }
    fn setpos(&mut self, pos: [f64; 2]){ self.0.setpos(pos); }
    fn getvelo(&self) -> [f64; 2]{ self.0.getvelo() }
    fn draw(&self, context: &Context, g: &mut G2d){ self.0.draw(context, g); }
    fn name(&self) -> &str{ "Boss" }
}


const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() {
    //use glutin_window::GlutinWindow;
    //let (WIDTH, HEIGHT) = (640, 480);
    let mut time = 0;
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow  =
        WindowSettings::new("Hello Piston!", [WIDTH, HEIGHT])
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
    //let resources = Resources::new(&mut window, &assets);
    let player = Enemy( EntityBase{ pos: [0., 100.], velo: [0., 0.], texture: &player_tex} );

    let mut game = Game{
        rng: thread_rng(),
        bullets: Vec::<Box<Entity>>::new(),
        resources: Resources::new(&mut window, &assets)
    };

    let mut enemies: Vec<Box<Entity>> = vec!{
        //Enemy::new([135., 312.], [0f64, 0f64], &game.resources),
    };

    let mut _rng = &game.rng;

    while let Some(event) = window.next() {

        window.draw_2d(&event, |context, graphics| {
            clear([0.0, 0., 0., 1.], graphics);

            image(&bg, context.transform, graphics);

            player.draw(&context, graphics);

            time = (time + 1) % 100;

            game.animate();

            let mut to_delete: Vec<usize> = Vec::new();
            for (i, e) in &mut ((&mut enemies).iter_mut().enumerate()) {
                if !e.animate() {
                    to_delete.push(i);
                    continue;
                }
                e.draw(&context, graphics);

                let x: i32 = game.rng.gen_range(0, 100);
                if x < 2 {
                    game.bullets.push(Bullet::new (
                        (*e).getpos(),
                        [game.rng.gen::<f64>() - 0.5, game.rng.gen::<f64>() - 0.5],
                        &game.resources
                    ))
                }
            }

            for i in to_delete.iter().rev() {
                let dead = enemies.remove(*i);
                println!("Deleted Enemy {} {} / {}", dead.name(), *i, enemies.len());
            }

            to_delete.clear();

            for (i,b) in &mut game.bullets.iter_mut().enumerate() {
                if !b.animate() {
                    to_delete.push(i);
                }
                b.draw(&context, graphics);
            }

            for i in to_delete.iter().rev() {
                game.bullets.remove(*i);
                println!("Deleted bullet {} / {}", *i, game.bullets.len());
            }

            //print!("time: {}, tran: {:?}\n", time, tran);
            //scene.draw(context.transform, graphics);
        });
    }
}