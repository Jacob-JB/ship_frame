use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

use crate::{graph::*, BeamDirection, BeamEnd, BeamId, VertexId};

#[derive(Serialize, Deserialize, Clone)]
pub struct SerializedGraph<B> {
    pub(crate) vertices: Vec<(VertexId, Vec3)>,
    pub(crate) beams: Vec<(BeamId, B)>,
}

impl<B> Default for SerializedGraph<B> {
    fn default() -> Self {
        SerializedGraph {
            vertices: Vec::new(),
            beams: Vec::new(),
        }
    }
}

impl<B> From<&Graph<B>> for SerializedGraph<B>
where
    B: Clone,
{
    fn from(graph: &Graph<B>) -> Self {
        let mut serialized = SerializedGraph::default();

        for (&id, vertex) in graph.vertices.iter() {
            serialized.vertices.push((id, vertex.position()));
        }

        for (&id, beam_data) in graph.beams.iter() {
            serialized.beams.push((id, beam_data.clone()));
        }

        serialized
    }
}

impl<B> From<SerializedGraph<B>> for Graph<B> {
    fn from(serialized: SerializedGraph<B>) -> Self {
        let mut graph = Graph::default();

        for (id, position) in serialized.vertices {
            graph.vertices.insert(
                id,
                Vertex {
                    position,
                    connections: Vec::new(),
                },
            );
        }

        for (id, beam_data) in serialized.beams {
            graph
                .vertices
                .get_mut(&id.down_vertex())
                .expect("Invalid serialized graph structure.")
                .connections
                .push(BeamEnd {
                    beam_id: id,
                    beam_end: BeamDirection::Down,
                });

            graph
                .vertices
                .get_mut(&id.up_vertex())
                .expect("Invalid serialized graph structure.")
                .connections
                .push(BeamEnd {
                    beam_id: id,
                    beam_end: BeamDirection::Up,
                });

            graph.beams.insert(id, beam_data);
        }

        graph
    }
}

#[derive(Serialize, Deserialize)]
pub enum FrameUpdate<B> {
    AddBeam {
        vertex_a: VertexId,
        position_a: Option<Vec3>,
        vertex_b: VertexId,
        position_b: Option<Vec3>,
        beam_data: B,
    },
    RemoveBeam {
        id: BeamId,
    },
}
