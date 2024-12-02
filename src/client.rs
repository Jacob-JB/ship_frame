use crate::{
    graph::Graph,
    messages::{FrameUpdate, SerializedGraph},
};

pub struct ShipFrame<B> {
    graph: Graph<B>,
}

impl<B> ShipFrame<B> {
    pub fn new(serialized: SerializedGraph<B>) -> Self {
        ShipFrame {
            graph: serialized.into(),
        }
    }

    pub fn apply_update(&mut self, update: FrameUpdate<B>) {
        match update {
            FrameUpdate::AddBeam {
                vertex_a,
                position_a,
                vertex_b,
                position_b,
                beam_data,
            } => {
                self.graph
                    .add_beam(vertex_a, position_a, vertex_b, position_b, beam_data);
            }
            FrameUpdate::RemoveBeam { id } => {
                self.graph.remove_beam(id);
            }
        }
    }
}
