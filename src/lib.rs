use std::collections::BinaryHeap;

use bevy::{
    math::prelude::*,
    utils::{HashMap, HashSet},
};
use slotmap::{new_key_type, HopSlotMap};

new_key_type! {
    pub struct BeamId;
    pub struct VertexId;
}

/// Enumerated direction on a beam.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum BeamDirection {
    Up,
    Down,
}

/// A specific end on a beam.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct BeamEnd {
    pub beam: BeamId,
    pub end: BeamDirection,
}

/// A vertex.
///
/// Stores connected `BeamEnd`s and the position of the vertex.
pub struct Vertex {
    connections: Vec<BeamEnd>,
    pub position: Vec3,
}

/// A beam.
///
/// Stores which vertex is connected in each direction.
///
/// The position of each end is stored in the vertices.
pub struct Beam {
    up: VertexId,
    down: VertexId,
}

/// A frame graph.
///
/// Stores a set of beams and vertices.
/// You modify the graph by creating and removing beams,
/// vertices will be created and removed as needed.
#[derive(Default)]
pub struct FrameGraph {
    vertices: HopSlotMap<VertexId, Vertex>,
    beams: HopSlotMap<BeamId, Beam>,
}

impl FrameGraph {
    /// Creates a new empty frame graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new beam and two vertices between two points
    ///
    /// Returns the [BeamId] and the up and down [VertexId] in that order
    pub fn create_beam(
        &mut self,
        up_position: Vec3,
        down_position: Vec3,
    ) -> (BeamId, VertexId, VertexId) {
        let up = self.vertices.insert(Vertex {
            connections: Vec::new(),
            position: up_position,
        });

        let down = self.vertices.insert(Vertex {
            connections: Vec::new(),
            position: down_position,
        });

        let beam_id = self.beams.insert(Beam { up, down });

        self.get_vertex_mut(up).unwrap().connections.push(BeamEnd {
            beam: beam_id,
            end: BeamDirection::Up,
        });

        self.get_vertex_mut(down)
            .unwrap()
            .connections
            .push(BeamEnd {
                beam: beam_id,
                end: BeamDirection::Down,
            });

        (beam_id, up, down)
    }

    /// Removes a beam, removing its verticies if this beam was the last remaining connection.
    ///
    /// Returns `true` if the beam existed.
    pub fn remove_beam(&mut self, beam_id: BeamId) -> bool {
        let Some(beam) = self.beams.remove(beam_id) else {
            return false;
        };

        let up_vertex = self
            .get_vertex_mut(beam.up)
            .expect("Beam should point to a valid vertex");

        if up_vertex
            .remove_connection(BeamEnd {
                beam: beam_id,
                end: BeamDirection::Up,
            })
            .expect("Vertex should contain a connection to the beam")
        {
            self.vertices.remove(beam.up);
        }

        let down_vertex = self
            .get_vertex_mut(beam.down)
            .expect("Beam should point to a valid vertex");

        if down_vertex
            .remove_connection(BeamEnd {
                beam: beam_id,
                end: BeamDirection::Down,
            })
            .expect("Vertex should contain a connection to the beam")
        {
            self.vertices.remove(beam.down);
        }

        true
    }

    /// Merges all connections on one vertex onto another vertex, removing the first vertex.
    ///
    /// Returns the vertex containing all the connections.
    pub fn merge_vertices(
        &mut self,
        from_vertex_id: VertexId,
        into_vertex_id: VertexId,
    ) -> Option<&mut Vertex> {
        self.get_vertex(into_vertex_id)?;

        let Vertex {
            mut connections, ..
        } = self.vertices.remove(from_vertex_id)?;

        for connection in connections.iter() {
            let beam = self.get_beam_mut(connection.beam).unwrap();
            *beam.get_vertex_mut(connection.end) = into_vertex_id;
        }

        let into = self.get_vertex_mut(into_vertex_id).unwrap();

        into.connections.append(&mut connections);

        Some(into)
    }

    /// Splits a vertex into two vertices.
    /// The new vertex will have any connections that
    /// `predicate` returns `true` for.
    ///
    /// Returns `None` if the `VertexId` is invalid.
    ///
    /// Returns `Some(None)` if `predicate` returned
    /// `true` for zero connections.
    ///
    /// Returns `Some(Some(VertexId))` for the new vertex.
    ///
    /// Will do nothing and return the given `VertexId`
    /// if `predicate` returns `true` for every connection.
    pub fn split_vertex(
        &mut self,
        vertex_id: VertexId,
        mut predicate: impl FnMut(&BeamEnd) -> bool,
    ) -> Option<Option<VertexId>> {
        let vertex = self.get_vertex_mut(vertex_id)?;

        let mut connections = Vec::new();
        let position = vertex.position;

        vertex.connections.retain(|connection| {
            if predicate(connection) {
                connections.push(*connection);
                false
            } else {
                true
            }
        });

        if connections.is_empty() {
            return Some(None);
        }

        if vertex.connections.is_empty() {
            vertex.connections = connections;
            return Some(Some(vertex_id));
        }

        Some(Some(self.vertices.insert(Vertex {
            connections,
            position,
        })))
    }

    /// Gets a reference to a vertex if it exists.
    pub fn get_vertex(&self, vertex_id: VertexId) -> Option<&Vertex> {
        self.vertices.get(vertex_id)
    }

    /// Gets a mutable reference to a vertex if it exists.
    pub fn get_vertex_mut(&mut self, vertex_id: VertexId) -> Option<&mut Vertex> {
        self.vertices.get_mut(vertex_id)
    }

    /// Gets a reference to a beam if it exists.
    pub fn get_beam(&self, beam_id: BeamId) -> Option<&Beam> {
        self.beams.get(beam_id)
    }

    /// Gets a mutable reference to a beam if it exists.
    fn get_beam_mut(&mut self, beam_id: BeamId) -> Option<&mut Beam> {
        self.beams.get_mut(beam_id)
    }

    /// Returns an iterator for all vertices.
    pub fn iter_verticies(&self) -> impl Iterator<Item = (VertexId, &Vertex)> {
        self.vertices.iter()
    }

    /// Returns an iterator for all vertices mutably.
    pub fn iter_verticies_mut(&mut self) -> impl Iterator<Item = (VertexId, &mut Vertex)> {
        self.vertices.iter_mut()
    }

    /// Returns an iterator for all beams.
    pub fn iter_beams(&self) -> impl Iterator<Item = (BeamId, &Beam)> {
        self.beams.iter()
    }

    /// Returns an iterator for all beams along with the
    /// position of its up and down vertices respectively.
    pub fn iter_beam_positions(&self) -> impl Iterator<Item = (BeamId, &Beam, Vec3, Vec3)> {
        self.iter_beams().map(|(beam_id, beam)| {
            (
                beam_id,
                beam,
                self.get_vertex(beam.up)
                    .expect("Beam should contain a valid vertex id.")
                    .position,
                self.get_vertex(beam.down)
                    .expect("Beam should contain a valid vertex id.")
                    .position,
            )
        })
    }

    /// Checks if two vertices are in two separate sub graphs.
    ///
    /// If they are, will split all the verticies reachable by `start_vertex_id` into a separate `FrameGraph`.
    ///
    /// Panics if either `VertexId` isn't valid.
    ///
    /// Returns `None` if the verticies can reach each other.
    pub fn try_split(
        &mut self,
        start_vertex_id: VertexId,
        end_vertex_id: VertexId,
    ) -> Option<Self> {
        let start_position = self.get_vertex(start_vertex_id)?.position;
        let end_position = self.get_vertex(end_vertex_id)?.position;

        // Stores which verticies are reachable from `start_position`.
        let mut visited = HashSet::new();

        // Stores the best path cost from `start_position`.
        //
        // When a new best cost is found the path is re added to the queue.
        //
        // If there is no entry then the cost is infinity.
        let mut best_costs = HashMap::new();

        #[derive(Clone, Copy)]
        struct QueueItem {
            /// The path priority
            f: f32,
            cost: f32,
            vertex_id: VertexId,
        }

        // A min heap used to choose which path to explore next.
        let mut heap = vec![QueueItem {
            f: start_position.distance(end_position),
            cost: 0.,
            vertex_id: start_vertex_id,
        }];

        best_costs.insert(start_vertex_id, 0.);

        loop {
            print!("queue:");
            for i in heap.iter() {
                let position = self.get_vertex(i.vertex_id).unwrap().position;
                print!(" {}", position);
            }
            println!();

            // take the first item
            let Some(&QueueItem {
                vertex_id, cost, ..
            }) = heap.first()
            else {
                // No more paths to explore, exit loop and split sub graph.
                break;
            };

            // if there are remaing items restore the heap
            let last = heap.pop().unwrap();
            if let Some(front) = heap.first_mut() {
                *front = last;

                let mut parent_index = 0;

                loop {
                    let parent = heap.get(parent_index).unwrap();

                    let left_child_index = parent_index * 2 + 1;
                    let right_child_index = parent_index * 2 + 2;

                    if let Some(left_child) = heap.get(left_child_index) {
                        if parent.f < left_child.f {
                            heap.swap(parent_index, left_child_index);
                            parent_index = left_child_index;
                            continue;
                        }
                    }

                    if let Some(right_child) = heap.get(right_child_index) {
                        if parent.f < right_child.f {
                            heap.swap(parent_index, right_child_index);
                            parent_index = right_child_index;
                            continue;
                        }
                    }

                    break;
                }
            }

            if vertex_id == end_vertex_id {
                // End vertex is reachable.
                println!("path found");
                return None;
            }

            visited.insert(vertex_id);

            let vertex = self.get_vertex(vertex_id).unwrap();

            let &path_cost = best_costs
                .get(&vertex_id)
                .expect("Vertex should not be in queue if it is in map.");

            if path_cost < cost {
                // already explored a path on this vertex with a lower cost
                continue;
            }

            println!("visited {}", vertex.position);

            for connection in vertex.connections() {
                let opposite_vertex_id = self
                    .get_beam(connection.beam)
                    .unwrap()
                    .get_vertex(connection.end.opposite());

                let opposite_vertex = self.get_vertex(opposite_vertex_id).unwrap();

                let beam_length = vertex.position.distance(opposite_vertex.position);
                let possible_cost = path_cost + beam_length;

                let current_cost = best_costs.entry(opposite_vertex_id).or_insert(f32::MAX);

                if possible_cost < *current_cost {
                    *current_cost = possible_cost;

                    let mut child_index = heap.len();
                    heap.push(QueueItem {
                        f: possible_cost + opposite_vertex.position.distance(end_position),
                        cost: possible_cost,
                        vertex_id: opposite_vertex_id,
                    });

                    while child_index > 0 {
                        let parent_index = (child_index - 1) / 2;

                        let child = heap.get(child_index).unwrap();
                        let parent = heap.get(parent_index).unwrap();

                        if child.f < parent.f {
                            heap.swap(child_index, parent_index);
                            child_index = parent_index;
                            continue;
                        }

                        break;
                    }
                }
            }
        }

        let mut new_graph = FrameGraph::new();

        // first copy all the vertices over
        let mut vertex_id_map = HashMap::new();

        let mut visited_beams = HashSet::new();

        for &vertex_id in visited.iter() {
            let vertex = self.get_vertex(vertex_id).unwrap();
            let mapped_vertex_id = new_graph.vertices.insert(Vertex {
                position: vertex.position,
                connections: Vec::new(),
            });
            vertex_id_map.insert(vertex_id, mapped_vertex_id);

            for connection in vertex.connections() {
                visited_beams.insert(connection.beam);
            }
        }

        let mut beam_id_map = HashMap::new();

        for &beam_id in visited_beams.iter() {
            let beam = self.get_beam(beam_id).unwrap();
            let &up = vertex_id_map.get(&beam.up).unwrap();
            let &down = vertex_id_map.get(&beam.down).unwrap();
            let mapped_beam_id = new_graph.beams.insert(Beam { up, down });
            beam_id_map.insert(beam_id, mapped_beam_id);

            new_graph
                .get_vertex_mut(up)
                .unwrap()
                .connections
                .push(BeamEnd {
                    beam: mapped_beam_id,
                    end: BeamDirection::Up,
                });

            new_graph
                .get_vertex_mut(down)
                .unwrap()
                .connections
                .push(BeamEnd {
                    beam: mapped_beam_id,
                    end: BeamDirection::Down,
                });
        }

        todo!()
    }
}

impl BeamDirection {
    pub fn opposite(self) -> Self {
        match self {
            BeamDirection::Up => BeamDirection::Down,
            BeamDirection::Down => BeamDirection::Up,
        }
    }
}

impl Vertex {
    /// Attempts to remove a connected beam,
    /// returns `None` if the connection was not found.
    ///
    /// Will return `Some(true)` if it was the last remaining connection
    /// and the vertex should be removed
    fn remove_connection(&mut self, connection: BeamEnd) -> Option<bool> {
        let index = self.connections.iter().position(|c| *c == connection)?;

        self.connections.remove(index);

        Some(self.connections.is_empty())
    }

    /// Returns a slice of all the beams connected to this vertex.
    pub fn connections(&self) -> &[BeamEnd] {
        self.connections.as_slice()
    }
}

impl Beam {
    /// Gets the `VertexId` connected in the up direction.
    pub fn up_vertex(&self) -> VertexId {
        self.up
    }

    /// Gets the `VertexId` connected in the down direction.
    pub fn down_vertex(&self) -> VertexId {
        self.down
    }

    /// Gets the `VertexId` connected in a direction.
    pub fn get_vertex(&self, direction: BeamDirection) -> VertexId {
        match direction {
            BeamDirection::Up => self.up,
            BeamDirection::Down => self.down,
        }
    }

    /// Gets a mutable reference to the `VertexId` connected in a direction.
    fn get_vertex_mut(&mut self, direction: BeamDirection) -> &mut VertexId {
        match direction {
            BeamDirection::Up => &mut self.up,
            BeamDirection::Down => &mut self.down,
        }
    }
}
