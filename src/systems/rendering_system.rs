use ggez::graphics::{self, Color, DrawParam, Image};
use ggez::mint as mi;
use ggez::Context;
use specs::{join::Join, Read, ReadStorage, System};
use std::time::Duration;

use crate::components::{Position, Renderable, RenderableKind};
use crate::constants::TILE_WIDTH;
use crate::resources::{Gameplay, Time};

pub struct RenderingSystem<'a> {
    pub context: &'a mut Context,
}

impl RenderingSystem<'_> {
    pub fn draw_text(&mut self, text_string: &str, x: f32, y: f32) {
        let text = graphics::Text::new(text_string);
        let destination = mi::Point2 { x, y };
        let color = Some(Color::new(0.0, 0.0, 0.0, 1.0));
        let dimensions = mi::Point2 { x: 0.0, y: 20.0 };

        graphics::queue_text(self.context, &text, dimensions, color);
        graphics::draw_queued_text(
            self.context,
            graphics::DrawParam::new().dest(destination),
            None,
            graphics::FilterMode::Linear,
        )
        .expect("expect drawing queued text");
    }

    pub fn get_image(&mut self, renderable: &Renderable, delta: Duration) -> Image {
        let path_index = match renderable.kind() {
            RenderableKind::Static => 0,
            RenderableKind::Animated => ((delta.as_millis() % 1000) / 250) as usize,
        };

        let image_path = renderable.path(path_index);
        Image::new(self.context, image_path).expect("expected image")
    }
}

impl<'a> System<'a> for RenderingSystem<'a> {
    type SystemData = (
        Read<'a, Gameplay>,
        Read<'a, Time>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        graphics::clear(self.context, Color::WHITE);
        let (gameplay, time, position, renderables) = data;

        // Should change that to FlaggedStorage to maintained a sorted Entity list
        // https://specs.amethyst.rs/docs/tutorials/12_tracked.html
        let mut rendering_data = (&position, &renderables).join().collect::<Vec<_>>();
        rendering_data.sort_by_key(|&k| k.0.z);

        for (position, renderable) in rendering_data.iter() {
            let image = self.get_image(renderable, time.delta);
            let x = position.x as f32 * TILE_WIDTH;
            let y = position.y as f32 * TILE_WIDTH;

            let draw_params = DrawParam::new().dest(mi::Point2 { x, y });
            graphics::draw(self.context, &image, draw_params).expect("expected render");
        }
        // Render any text
        self.draw_text(&gameplay.state.to_string(), 525.0, 80.0);
        self.draw_text(&gameplay.moves_count.to_string(), 525.0, 100.0);

        graphics::present(self.context).expect("expected to present");
    }
}
