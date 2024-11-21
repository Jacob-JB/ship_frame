use bevy::prelude::*;

use ship_frame::*;

fn main() {
    // let mut app = App::new();

    // app.add_plugins(LogPlugin {
    //     level: Level::DEBUG,
    //     ..default()
    // });
    // app.add_plugins(MinimalPlugins);

    // app.run();

    let mut allocator = FrameIdAllocator::new();

    let (mut frame, a, b) = FrameGraph::new(
        &mut allocator,
        Vec3::new(0., 0., 0.),
        Vec3::new(5., 0., 0.),
        (),
    );
    let c = frame.new_beam_extend(&mut allocator, b, Vec3::new(0., 5., 0.), ());
    let d = frame.new_beam_extend(&mut allocator, c, Vec3::new(5., 5., 0.), ());
    frame.new_beam_join(a, d, ());

    println!("original: {:?}", frame);

    if let Some(split) = frame.remove_beam(BeamId::from_vertices(b, c)) {
        println!("split into {:?} and {:?}", frame, split);
    } else {
        println!("no split");
    }
}
