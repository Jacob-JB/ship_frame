use bevy::prelude::*;

use ship_frame::{client, server};

fn main() {
    // let mut app = App::new();

    // app.add_plugins(LogPlugin {
    //     level: Level::DEBUG,
    //     ..default()
    // });
    // app.add_plugins(MinimalPlugins);

    // app.run();

    let mut id_world = server::FrameIdWorld::default();

    let mut server_frame = server::ShipFrame::<()>::new_from_beam(
        &mut id_world,
        Vec3::new(0., 0., 0.),
        Vec3::new(5., 0., 0.),
        (),
    );

    let (vertex_b, _) = server_frame.iter_vertices().nth(1).unwrap();

    let new_beam_message = server_frame.serialize();
    let mut update_messages = Vec::new();

    update_messages.push(server_frame.add_beam_extend(
        &mut id_world,
        vertex_b,
        Vec3::new(5., 5., 0.),
        (),
    ))
}
