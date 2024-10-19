use ft_vox_prototype_0_main::run;
use ft_vox_prototype_0_terrain_worker_native::NativeTerrainWorker;
use futures::executor::block_on;

fn main() {
    block_on(run::<NativeTerrainWorker>());
}
