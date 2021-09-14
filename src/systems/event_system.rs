use ggez::Context;
use specs::{Entities, Join, ReadStorage, System, Write};
use std::collections::HashMap;

use crate::{
    audio::AudioStore,
    components::*,
    events::{BoxPlacedOnSpot, EntityMoved, Event},
    resources::EventQueue,
};

pub struct EventSystem<'a> {
    pub context: &'a mut Context,
}

impl<'a> System<'a> for EventSystem<'_> {
    type SystemData = (
        Write<'a, EventQueue>,
        Write<'a, AudioStore>,
        Entities<'a>,
        ReadStorage<'a, Box>,
        ReadStorage<'a, BoxSpot>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut event_queue, mut audio_store, entities, boxes, box_spots, positions) = data;
        let mut new_events = Vec::new();

        for event in event_queue.events.drain(..) {
            println!("New event: {:?}", event);
            match event {
                Event::PlayerHitObstacle => {
                    audio_store.play_sound(&"error".to_string(), self.context);
                }

                Event::EntityMoved(EntityMoved { id }) => {
                    if let Some(the_box) = boxes.get(entities.entity(id)) {
                        let box_spots_with_positions: HashMap<(u8, u8), &BoxSpot> =
                            (&box_spots, &positions)
                                .join()
                                .map(|t| ((t.1.x, t.1.y), t.0))
                                .collect::<HashMap<_, _>>();

                        if let Some(box_position) = positions.get(entities.entity(id)) {
                            // Check if there is a spot on this position, and if there
                            // is if it's the correct or incorrect type
                            if let Some(box_spot) =
                                box_spots_with_positions.get(&(box_position.x, box_position.y))
                            {
                                new_events.push(Event::BoxPlacedOnSpot(BoxPlacedOnSpot {
                                    is_correct_spot: (box_spot.color == the_box.color),
                                }));
                            }
                        }
                    }
                }
                Event::BoxPlacedOnSpot(BoxPlacedOnSpot { is_correct_spot }) => {
                    // play sound here
                    let sound = if is_correct_spot {
                        "correct"
                    } else {
                        "incorrect"
                    };
                    audio_store.play_sound(&sound.to_string(), self.context);
                }
            }
        }
        event_queue.events.append(&mut new_events);
    }
}
