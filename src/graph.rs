use bevy::math::Vec3;
use indexmap::{IndexMap, IndexSet};

use crate::{BeamEnd, BeamId, VertexId};

pub struct Vertex {
    position: Vec3,
    connections: Vec<BeamEnd>,
}

/// The core data structure used by the server and client
pub(crate) struct Graph {
    verticies: IndexMap<VertexId, Vertex>,
    beams: IndexSet<BeamId>,
}

impl Default for Graph {
    fn default() -> Self {
        Graph {
            verticies: IndexMap::new(),
            beams: IndexSet::new(),
        }
    }
}

impl Graph {}
