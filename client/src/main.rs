use futures::executor::block_on;

mod context;
mod input;
mod run;
mod surface_wrapper;
mod wgpu_context;

fn main() {
    block_on(run::run());
}
