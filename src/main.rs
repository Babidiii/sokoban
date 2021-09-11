use ggez;
use ggez::event::{self, EventHandler};
use ggez::graphics::DrawParam;
use ggez::graphics::Image;
use ggez::graphics::{self, Color};
use ggez::mint as mi;
use ggez::{conf, Context, ContextBuilder, GameError, GameResult};
use specs::{
    join::Join, Builder, Component, ReadStorage, RunNow, System, VecStorage, World, WorldExt,
};
use std::path;

const TILE_WIDTH: f32 = 32.0;

// Game hold all the game state
struct Game {
    world: World,
}

impl Game {
    pub fn new(world: World) -> Game {
        Game { world }
    }
}

impl EventHandler<GameError> for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::WHITE);

        {
            let mut rs = RenderingSystem { context: ctx };
            rs.run_now(&self.world);
        }
        graphics::present(ctx)
    }
}

fn main() {
    let mut world = World::new();
    register_components(&mut world);
    initialize_level(&mut world);

    // create a game context and event loop
    let context_builder = ContextBuilder::new("babidiii_sokoban", "sokoban")
        .window_setup(conf::WindowSetup::default().title("Sokoban!"))
        .window_mode(conf::WindowMode::default().dimensions(1000.0, 600.0))
        .add_resource_path(path::PathBuf::from("./resources"));

    let (mut ctx, event_loop) = context_builder.build().expect("Could not create ggez game");

    // Create game state
    let game = Game::new(world);
    // let game = &mut Game {};

    event::run(ctx, event_loop, game)
}

#[derive(Debug, Component, Clone, Copy)]
#[storage(VecStorage)]
pub struct Position {
    x: u8,
    y: u8,
    z: u8,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Renderable {
    path: String,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Wall {}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Player {}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Box {}

#[derive(Component)]
#[storage(VecStorage)]
pub struct BoxSpot {}

pub fn register_components(world: &mut World) {
    world.register::<Position>();
    world.register::<Renderable>();
    world.register::<Player>();
    world.register::<Wall>();
    world.register::<Box>();
    world.register::<BoxSpot>();
}

pub fn create_wall(world: &mut World, position: Position) {
    world
        .create_entity()
        .with(Position { z: 10, ..position })
        .with(Renderable {
            path: "/images/wall.png".to_string(),
        })
        .with(Wall {})
        .build();
}

pub fn create_floor(world: &mut World, position: Position) {
    world
        .create_entity()
        .with(Position { z: 5, ..position })
        .with(Renderable {
            path: "/images/floor.png".to_string(),
        })
        .build();
}

pub fn create_box(world: &mut World, position: Position) {
    world
        .create_entity()
        .with(Position { z: 10, ..position })
        .with(Renderable {
            path: "/images/box.png".to_string(),
        })
        .with(Box {})
        .build();
}

pub fn create_box_spot(world: &mut World, position: Position) {
    world
        .create_entity()
        .with(Position { z: 9, ..position })
        .with(Renderable {
            path: "/images/box_spot.png".to_string(),
        })
        .with(BoxSpot {})
        .build();
}

pub fn create_player(world: &mut World, position: Position) {
    world
        .create_entity()
        .with(Position { z: 10, ..position })
        .with(Renderable {
            path: "/images/player.png".to_string(),
        })
        .with(Player {})
        .build();
}

pub struct RenderingSystem<'a> {
    context: &'a mut Context,
}

impl<'a> System<'a> for RenderingSystem<'a> {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Renderable>);

    fn run(&mut self, data: Self::SystemData) {
        let (position, renderables) = data;

        let mut rendering_data = (&position, &renderables).join().collect::<Vec<_>>();
        rendering_data.sort_by_key(|&k| k.0.z);

        for (position, renderable) in rendering_data.iter() {
            let image = Image::new(self.context, renderable.path.clone()).expect("expected image");
            let x = position.x as f32 * TILE_WIDTH;
            let y = position.y as f32 * TILE_WIDTH;

            let draw_params = DrawParam::new().dest(mi::Point2 { x, y });
            graphics::draw(self.context, &image, draw_params).expect("expected render");
        }
    }
}

// // Initialize the level
// pub fn initialize_level(world: &mut World) {
//     create_player(
//         world,
//         Position {
//             x: 0,
//             y: 0,
//             z: 0, // we will get the z from the factory functions
//         },
//     );
//     create_wall(
//         world,
//         Position {
//             x: 1,
//             y: 0,
//             z: 0, // we will get the z from the factory functions
//         },
//     );
//     create_box(
//         world,
//         Position {
//             x: 2,
//             y: 0,
//             z: 0, // we will get the z from the factory functions
//         },
//     );
// }

pub fn initialize_level(world: &mut World) {
    const MAP: &str = "
    N N W W W W W W
    W W W . . . . W
    W . . . B . . W
    W . . . . . . W 
    W . P . . . . W
    W . . . . . . W
    W . . S . . . W
    W . . . . . . W
    W W W W W W W W
    ";

    load_map(world, MAP.to_string());
}

pub fn load_map(world: &mut World, map_string: String) {
    // read all lines
    let rows: Vec<&str> = map_string.trim().split('\n').map(|x| x.trim()).collect();

    for (y, row) in rows.iter().enumerate() {
        let columns: Vec<&str> = row.split(' ').collect();

        for (x, column) in columns.iter().enumerate() {
            // Create the position at which to create something on the map
            let position = Position {
                x: x as u8,
                y: y as u8,
                z: 0, // we will get the z from the factory functions
            };

            // Figure out what object we should create
            match *column {
                "." => create_floor(world, position),
                "W" => {
                    create_floor(world, position);
                    create_wall(world, position);
                }
                "P" => {
                    create_floor(world, position);
                    create_player(world, position);
                }
                "B" => {
                    create_floor(world, position);
                    create_box(world, position);
                }
                "S" => {
                    create_floor(world, position);
                    create_box_spot(world, position);
                }
                "N" => (),
                c => panic!("unrecognized map item {}", c),
            }
        }
    }
}
