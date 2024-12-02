use bevy::{prelude::*, utils::HashMap};

use crate::{
    graph::*,
    messages::{FrameUpdate, SerializedGraph},
    BeamId, VertexId,
};

#[derive(Resource, Default)]
pub struct FrameIdWorld {
    next_id: u64,
}

impl FrameIdWorld {
    pub fn next(&mut self) -> VertexId {
        let id = VertexId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Maps a frame graph into this id id_world's scope,
    /// ensuring that ids within the graph stay consistent,
    /// but don't reference any existing ids.
    pub fn map_frame<B>(&mut self, mut graph: SerializedGraph<B>) -> ShipFrame<B> {
        let mut map = HashMap::new();

        for (id, _) in graph.vertices.iter_mut() {
            *id = *map.entry(*id).or_insert_with(|| self.next());
        }

        for (id, _) in graph.beams.iter_mut() {
            let (mut id_a, mut id_b) = id.vertices();

            id_a = *map.entry(id_a).or_insert_with(|| self.next());
            id_b = *map.entry(id_b).or_insert_with(|| self.next());

            *id = BeamId::from_vertices(id_a, id_b)
        }

        ShipFrame {
            graph: graph.into(),
        }
    }
}

#[derive(Component)]
pub struct ShipFrame<B> {
    graph: Graph<B>,
}

impl<B> ShipFrame<B> {
    pub fn new_from_beam(
        id_world: &mut FrameIdWorld,
        position_a: Vec3,
        position_b: Vec3,
        beam_data: B,
    ) -> Self {
        let mut graph = Graph::default();

        let vertex_a = id_world.next();
        let vertex_b = id_world.next();

        graph.add_beam(
            vertex_a,
            Some(position_a),
            vertex_b,
            Some(position_b),
            beam_data,
        );

        ShipFrame { graph }
    }

    pub fn add_beam_extend(
        &mut self,
        id_world: &mut FrameIdWorld,
        existing_vertex: VertexId,
        position: Vec3,
        beam_data: B,
    ) -> FrameUpdate<B>
    where
        B: Clone,
    {
        let new_vertex = id_world.next();

        self.graph.add_beam(
            existing_vertex,
            None,
            new_vertex,
            Some(position),
            beam_data.clone(),
        );

        FrameUpdate::AddBeam {
            vertex_a: existing_vertex,
            position_a: None,
            vertex_b: new_vertex,
            position_b: Some(position),
            beam_data,
        }
    }

    pub fn add_beam_join(
        &mut self,
        vertex_a: VertexId,
        vertex_b: VertexId,
        beam_data: B,
    ) -> FrameUpdate<B>
    where
        B: Clone,
    {
        self.graph
            .add_beam(vertex_a, None, vertex_b, None, beam_data.clone());

        FrameUpdate::AddBeam {
            vertex_a,
            position_a: None,
            vertex_b,
            position_b: None,
            beam_data,
        }
    }

    pub fn serialize(&self) -> SerializedGraph<B>
    where
        B: Clone,
    {
        SerializedGraph::from(&self.graph)
    }

    pub fn iter_vertices(&self) -> impl Iterator<Item = (VertexId, &Vertex)> {
        self.graph.iter_vertices()
    }
}
