use crate::renderer::Renderer;

mod renderer;

fn main() {
    let renderer = Renderer::new(1280, 720);
    renderer.run();
}
