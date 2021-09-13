use ggez;
use ggez::event::{self, EventHandler};
use ggez::event::{KeyCode, KeyMods};
use ggez::graphics::DrawParam;
use ggez::graphics::Image;
use ggez::graphics::{self, Color};
use ggez::mint as mi;
use ggez::{conf, Context, ContextBuilder, GameError, GameResult};
use specs::world::Index;
use specs::{
    join::Join, Builder, Component, Entities, NullStorage, ReadStorage, RunNow, System, VecStorage,
    World, WorldExt, Write, WriteStorage,
};
use std::collections::HashMap;
use std::path;

const TILE_WIDTH: f32 = 32.0;
const MAP_WIDTH: u8 = 8;
const MAP_HEIGHT: u8 = 9;

// Specs ECS components
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

// Markers
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Movable;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Immovable;

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

// Resource
#[derive(Default)]
pub struct InputQueue {
    pub keys_pressed: Vec<KeyCode>,
}

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
        {
            let mut is = InputSystem {};
            is.run_now(&self.world);
        }
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

    fn key_down_event(
        &mut self,
        _context: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        println!("Key pressed: {:?}", keycode);

        let mut input_queue = self.world.write_resource::<InputQueue>();
        input_queue.keys_pressed.push(keycode);
    }
}

// ECS Rendering System
pub struct RenderingSystem<'a> {
    context: &'a mut Context,
}

impl<'a> System<'a> for RenderingSystem<'a> {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Renderable>);

    fn run(&mut self, data: Self::SystemData) {
        let (position, renderables) = data;

        // Should change that to FlaggedStorage to maintained a sorted Entity list
        // https://specs.amethyst.rs/docs/tutorials/12_tracked.html
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

pub struct InputSystem;

impl<'a> System<'a> for InputSystem {
    type SystemData = (
        Write<'a, InputQueue>,
        Entities<'a>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Movable>,
        ReadStorage<'a, Immovable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut input_queue, entities, mut positions, players, movables, immovables) = data;
        let mut to_move = Vec::new();

        for (position, _player) in (&positions, &players).join() {
            if let Some(key) = input_queue.keys_pressed.pop() {
                // retrive all the movables position and entity id into an hashmap (x,y) -> entity.id
                let mov: HashMap<(u8, u8), Index> = (&entities, &movables, &positions)
                    .join()
                    .map(|t| ((t.2.x, t.2.y), t.0.id()))
                    .collect::<HashMap<_, _>>();

                // retrive all the immovables position and entity id into an hashmap (x,y) -> entity.id
                let immov: HashMap<(u8, u8), Index> = (&entities, &immovables, &positions)
                    .join()
                    .map(|t| ((t.2.x, t.2.y), t.0.id()))
                    .collect::<HashMap<_, _>>();

                // Now iterate through current position to the end of the map
                // on the correct axis and check what needs to move
                let (start, end, is_x) = match key {
                    KeyCode::Up => (position.y, 0, false),
                    KeyCode::Down => (position.y, MAP_HEIGHT, false),
                    KeyCode::Left => (position.x, 0, true),
                    KeyCode::Right => (position.x, MAP_WIDTH, true),
                    _ => continue,
                };

                let range = if start < end {
                    (start..=end).collect::<Vec<_>>()
                } else {
                    (end..=start).rev().collect::<Vec<_>>()
                };

                for x_or_y in range {
                    // set the position from the range and player fixed value
                    let pos = if is_x {
                        (x_or_y, position.y)
                    } else {
                        (position.x, x_or_y)
                    };

                    // get the movable key:value for the specific position within our movable hashmap array
                    match mov.get(&pos) {
                        Some(id) => to_move.push((key, id.clone())), // we add the enity in our to_move vect
                        None => {
                            // It's not a movable so we will check if it's an immovable
                            match immov.get(&pos) {
                                Some(_id) => to_move.clear(), // immovable so we can't move<F2>
                                None => break,                // we can move because of a gap
                            }
                        }
                    }
                }
            }
        }

        // we move the entities for whom the id was added to to_move vect during the check
        for (key, id) in to_move {
            let position = positions.get_mut(entities.entity(id)); // retrive the position of the right enitity for writting purpose
            if let Some(position) = position {
                match key {
                    KeyCode::Up => position.y -= 1,
                    KeyCode::Down => position.y += 1,
                    KeyCode::Left => position.x -= 1,
                    KeyCode::Right => position.x += 1,
                    _ => (),
                };
            }
        }
    }
}

fn main() {
    let mut world = World::new();
    register_components(&mut world);
    register_resources(&mut world);
    initialize_level(&mut world);

    // create a game context and event loop
    let context_builder = ContextBuilder::new("babidiii_sokoban", "sokoban")
        .window_setup(conf::WindowSetup::default().title("Sokoban!"))
        .window_mode(conf::WindowMode::default().dimensions(1000.0, 600.0))
        .add_resource_path(path::PathBuf::from("./resources"));

    let (ctx, event_loop) = context_builder.build().expect("Could not create ggez game");

    // Create game state
    let game = Game::new(world);
    // let game = &mut Game {};

    event::run(ctx, event_loop, game)
}

// register components
pub fn register_components(world: &mut World) {
    world.register::<Position>();
    world.register::<Renderable>();
    world.register::<Player>();
    world.register::<Wall>();
    world.register::<Box>();
    world.register::<BoxSpot>();
    world.register::<Movable>();
    world.register::<Immovable>();
}

// Registering resources
pub fn register_resources(world: &mut World) {
    world.insert(InputQueue::default())
}

pub fn create_wall(world: &mut World, position: Position) {
    world
        .create_entity()
        .with(Position { z: 10, ..position })
        .with(Renderable {
            path: "/images/wall.png".to_string(),
        })
        .with(Wall {})
        .with(Immovable)
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
        .with(Movable)
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
        .with(Movable)
        .build();
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
