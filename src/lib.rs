use serde::{Deserialize, Serialize};

pub mod client;
pub mod graph;
pub mod messages;
pub mod server;

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeamDirection {
    Down,
    Up,
}

/// The direction on a beam.
///
/// Beams are undirected but the `Down`
/// direction always points to the older vertex id.
impl BeamDirection {
    pub fn opposite(self) -> Self {
        match self {
            BeamDirection::Down => BeamDirection::Up,
            BeamDirection::Up => BeamDirection::Down,
        }
    }
}

/// A vertex id unique to a [FrameIdAllocator].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VertexId(u64);

/// A beam id made up of two [VertexId]s.
///
/// The older vertex id is the "down" vertex.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeamId {
    down_id: u64,
    up_id: u64,
}

impl BeamId {
    /// Creates the id of a beam that could exist between two vertices.
    pub fn from_vertices(a: VertexId, b: VertexId) -> Self {
        if a < b {
            BeamId {
                down_id: a.0,
                up_id: b.0,
            }
        } else {
            BeamId {
                down_id: b.0,
                up_id: a.0,
            }
        }
    }

    pub fn vertices(self) -> (VertexId, VertexId) {
        (VertexId(self.down_id), VertexId(self.up_id))
    }

    pub fn down_vertex(self) -> VertexId {
        VertexId(self.down_id)
    }

    pub fn up_vertex(self) -> VertexId {
        VertexId(self.up_id)
    }

    pub fn vertex(self, direction: BeamDirection) -> VertexId {
        VertexId(match direction {
            BeamDirection::Down => self.down_id,
            BeamDirection::Up => self.up_id,
        })
    }
}

pub struct BeamEnd {
    pub beam_id: BeamId,
    pub beam_end: BeamDirection,
}

impl BeamEnd {
    pub fn opposite(&self) -> VertexId {
        match self.beam_end {
            BeamDirection::Down => self.beam_id.up_vertex(),
            BeamDirection::Up => self.beam_id.down_vertex(),
        }
    }
}
