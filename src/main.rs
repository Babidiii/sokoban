use ggez;
use ggez::event::{self, EventHandler};
use ggez::event::{KeyCode, KeyMods};
use ggez::{conf, Context, ContextBuilder, GameError, GameResult};
use specs::{RunNow, World, WorldExt};
use std::path;

mod components;
mod constants;
mod entities;
mod map;
mod resources;
mod systems;

use crate::components::*;
use crate::map::*;
use crate::resources::*;
use crate::systems::*;

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
        {
            let mut rs = RenderingSystem { context: ctx };
            rs.run_now(&self.world);
        }
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _context: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        let mut input_queue = self.world.write_resource::<InputQueue>();
        input_queue.keys_pressed.push(keycode);
    }
}

pub fn initialize_level(world: &mut World) {
    const MAP: &str = "
    N N W W W W W W
    W W W . . . . W
    W . . . B . . W
    W . . . . . . W 
    W . P W B . . W
    W . . . . . . W
    W . . S . . . W
    W . . . . . . W
    W W W W W W W W
    ";

    load_map(world, MAP.to_string());
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
