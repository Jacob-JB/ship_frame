use bevy::math::Vec3;
use indexmap::IndexMap;

use crate::{BeamDirection, BeamEnd, BeamId, VertexId};

/// The core data structure used by the server and client.
pub struct Graph<B> {
    pub(crate) vertices: IndexMap<VertexId, Vertex>,
    pub(crate) beams: IndexMap<BeamId, B>,
}

pub struct Vertex {
    pub(crate) position: Vec3,
    pub(crate) connections: Vec<BeamEnd>,
}

impl<B> Default for Graph<B> {
    fn default() -> Self {
        Graph {
            vertices: IndexMap::new(),
            beams: IndexMap::new(),
        }
    }
}

impl<B> Graph<B> {
    /// Inserts a beam between either existing or new vertices.
    ///
    /// If one end of the beam is connecting to an existing vertex, provide it's position as `None`
    /// Provide `Some` to insert a new vertex.
    ///
    /// Panics if inserting an existing vertex or if an exisiting vertex isn't in the graph.
    pub fn add_beam(
        &mut self,
        vertex_a: VertexId,
        position_a: Option<Vec3>,
        vertex_b: VertexId,
        position_b: Option<Vec3>,
        beam_data: B,
    ) {
        let (down_id, down_position, up_id, up_position) = match vertex_a.cmp(&vertex_b) {
            std::cmp::Ordering::Equal => {
                panic!("Tried to insert a beam between a vertex and itself.")
            }
            std::cmp::Ordering::Less => (vertex_a, position_a, vertex_b, position_b),
            std::cmp::Ordering::Greater => (vertex_b, position_b, vertex_a, position_a),
        };

        let beam_id = BeamId::from_vertices(down_id, up_id);

        for (id, position, beam_end) in [
            (down_id, down_position, BeamDirection::Down),
            (up_id, up_position, BeamDirection::Up),
        ] {
            if let Some(position) = position {
                let None = self.vertices.insert(
                    id,
                    Vertex {
                        position,
                        connections: vec![BeamEnd { beam_id, beam_end }],
                    },
                ) else {
                    panic!("Tried to insert a vertex twice.");
                };
            } else {
                let Some(vertex) = self.vertices.get_mut(&id) else {
                    panic!("Tried to connect a beam to a vertex that doesn't exist.");
                };

                vertex.connections.push(BeamEnd { beam_id, beam_end });
            }
        }

        let None = self.beams.insert(beam_id, beam_data) else {
            panic!("Tried to insert a beam twice.");
        };
    }

    /// Removes a beam, removing it's vertices from the graph
    /// if this beam was their last remaining connection.
    ///
    /// Panics if the beam is not in the graph
    pub fn remove_beam(&mut self, beam: BeamId) -> B {
        let Some(beam_data) = self.beams.swap_remove(&beam) else {
            panic!("Tried to remove a beam that doesn't exist.");
        };

        for id in [beam.down_vertex(), beam.up_vertex()] {
            let Some(vertex) = self.vertices.get_mut(&id) else {
                panic!("Vertex should exist if beam exists.");
            };

            let Some(index) = vertex
                .connections
                .iter()
                .position(|&BeamEnd { beam_id, .. }| beam_id == beam)
            else {
                panic!("Vertex should have a connection to the beam.");
            };

            vertex.connections.remove(index);

            if vertex.connections.is_empty() {
                self.vertices.swap_remove(&id);
            }
        }

        beam_data
    }

    pub fn get_vertex(&self, vertex_id: VertexId) -> Option<&Vertex> {
        self.vertices.get(&vertex_id)
    }

    pub fn get_beam(&self, beam_id: BeamId) -> Option<&B> {
        self.beams.get(&beam_id)
    }

    pub fn get_beam_mut(&mut self, beam_id: BeamId) -> Option<&mut B> {
        self.beams.get_mut(&beam_id)
    }

    pub fn iter_vertices(&self) -> impl Iterator<Item = (VertexId, &Vertex)> {
        self.vertices.iter().map(|(id, vertex)| (*id, vertex))
    }
}

impl Vertex {
    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn connections(&self) -> &[BeamEnd] {
        self.connections.as_slice()
    }
}

impl<B> std::fmt::Debug for Graph<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Graph {{ vertices: {}, beams: {} }}",
            self.vertices.len(),
            self.beams.len()
        )
    }
}
