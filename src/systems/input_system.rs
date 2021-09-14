use ggez::event::KeyCode;
use specs::world::Index;
use specs::{join::Join, Entities, ReadStorage, System, Write, WriteStorage};
use std::collections::HashMap;

use crate::components::{Immovable, Movable, Player, Position};
use crate::constants::MAP_HEIGHT;
use crate::constants::MAP_WIDTH;
use crate::resources::InputQueue;

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
                                Some(_id) => {
                                    to_move.clear();
                                    break;
                                } // immovable so we can't move<F2>
                                None => break, // we can move because of a gap
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
