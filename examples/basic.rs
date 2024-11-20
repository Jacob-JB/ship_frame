use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};

use ship_frame::*;

fn main() {
    // let mut app = App::new();

    // app.add_plugins(LogPlugin {
    //     level: Level::DEBUG,
    //     ..default()
    // });
    // app.add_plugins(MinimalPlugins);

    // app.run();

    let mut frame = FrameGraph::new();

    let (_, v00, v50) = frame.create_beam(Vec3::new(0., 0., 0.), Vec3::new(5., 0., 0.));
    let (_, v50b, v20) = frame.create_beam(Vec3::new(5., 0., 0.), Vec3::new(2., 0., 0.));
    frame.merge_vertices(v50b, v50);
    let (_, v00b, v01) = frame.create_beam(Vec3::new(0., 0., 0.), Vec3::new(0., 1., 0.));
    frame.merge_vertices(v00b, v00);
    let (_, v70, v80) = frame.create_beam(Vec3::new(7., 0., 0.), Vec3::new(8., 0., 0.));

    frame.try_split(v00, v80);
}
