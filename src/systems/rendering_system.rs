use ggez::graphics::{self, Color, DrawParam, Image};
use ggez::mint as mi;
use ggez::Context;
use specs::{join::Join, ReadStorage, System};

use crate::components::{Position, Renderable};
use crate::constants::TILE_WIDTH;

pub struct RenderingSystem<'a> {
    pub context: &'a mut Context,
}

impl<'a> System<'a> for RenderingSystem<'a> {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Renderable>);

    fn run(&mut self, data: Self::SystemData) {
        graphics::clear(self.context, Color::WHITE);
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
        graphics::present(self.context).expect("expected to present");
    }
}
