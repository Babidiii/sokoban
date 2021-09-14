use specs::{Join, ReadStorage, System, Write};
use std::collections::HashMap;

use crate::{
    components::{Box, BoxSpot, Position},
    resources::{Gameplay, GameplayState},
};

pub struct GameplayStateSystem;

impl<'a> System<'a> for GameplayStateSystem {
    type SystemData = (
        Write<'a, Gameplay>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Box>,
        ReadStorage<'a, BoxSpot>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (mut gameplay_state, positions, boxes, box_spots) = data;

        let boxes_by_position: HashMap<(u8, u8), &Box> = (&positions, &boxes)
            .join()
            .map(|t| ((t.0.x, t.0.y), t.1))
            .collect::<HashMap<_, _>>();

        for (box_spot, position) in (&box_spots, &positions).join() {
            if let Some(the_box) = boxes_by_position.get(&(position.x, position.y)) {
                if the_box.color == box_spot.color {
                    // continue
                } else {
                    return;
                }
            } else {
                gameplay_state.state = GameplayState::Playing;
                return;
            }
        }

        gameplay_state.state = GameplayState::Won;
    }
}
