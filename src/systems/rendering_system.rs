use ggez::graphics::{self, Color, DrawParam, Image};
use ggez::mint as mi;
use ggez::Context;
use specs::{join::Join, Read, ReadStorage, System};

use crate::components::{Position, Renderable};
use crate::constants::TILE_WIDTH;
use crate::resources::Gameplay;

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
}

impl<'a> System<'a> for RenderingSystem<'a> {
    type SystemData = (
        Read<'a, Gameplay>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        graphics::clear(self.context, Color::WHITE);
        let (gameplay, position, renderables) = data;

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
        // Render any text
        self.draw_text(&gameplay.state.to_string(), 525.0, 80.0);
        self.draw_text(&gameplay.moves_count.to_string(), 525.0, 100.0);

        graphics::present(self.context).expect("expected to present");
    }
}
