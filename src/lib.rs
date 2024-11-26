use bevy::{
    math::prelude::*,
    utils::{HashMap, HashSet},
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub mod graph;

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

#[derive(Default)]
pub struct FrameIdAllocator {
    next_id: u64,
}

impl FrameIdAllocator {
    pub fn new() -> Self {
        FrameIdAllocator::default()
    }

    fn next(&mut self) -> VertexId {
        let id = self.next_id;
        self.next_id += 1;
        VertexId(id)
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
    ///
    /// Panics in debug mode if the vertices are the same.
    pub fn from_vertices(a: VertexId, b: VertexId) -> Self {
        debug_assert_ne!(
            a, b,
            "A beam should never exist between a vertex and itself"
        );

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

pub struct Vertex {
    position: Vec3,
    connections: Vec<BeamEnd>,
}

impl Vertex {
    pub fn connections(&self) -> &[BeamEnd] {
        self.connections.as_slice()
    }

    /// Removes a connection to a beam.
    ///
    /// Does nothing if there is no connection to the beam.
    fn remove_connection(&mut self, beam: BeamId) {
        let Some(index) = self
            .connections
            .iter()
            .position(|connection| connection.beam_id == beam)
        else {
            return;
        };

        self.connections.remove(index);
    }
}

pub struct Beam<B> {
    pub beam_data: B,
}

pub struct FrameGraph<B> {
    vertices: IndexMap<VertexId, Vertex>,
    beams: IndexMap<BeamId, Beam<B>>,
}

pub enum BeamRemoveResult<B> {
    None,
    SplitFrame(FrameGraph<B>),
    LastBeam,
}

impl<B> FrameGraph<B> {
    fn empty() -> Self {
        FrameGraph {
            vertices: IndexMap::new(),
            beams: IndexMap::new(),
        }
    }

    /// Creates a new frame graph.
    ///
    /// Requres creating a starting beam to build the frame from.
    ///
    /// Returns the frame and the vertex ids of the two vertices created.
    pub fn new(
        allocator: &mut FrameIdAllocator,
        down_position: Vec3,
        up_position: Vec3,
        beam_data: B,
    ) -> (Self, VertexId, VertexId) {
        let mut frame = FrameGraph::empty();

        let down_id = allocator.next();
        let up_id = allocator.next();

        // `down_id` is always the down vertex
        let beam_id = BeamId::from_vertices(down_id, up_id);

        frame.vertices.insert(
            down_id,
            Vertex {
                position: down_position,
                connections: vec![BeamEnd {
                    beam_id,
                    beam_end: BeamDirection::Down,
                }],
            },
        );

        frame.vertices.insert(
            up_id,
            Vertex {
                position: up_position,
                connections: vec![BeamEnd {
                    beam_id,
                    beam_end: BeamDirection::Up,
                }],
            },
        );

        frame.beams.insert(beam_id, Beam { beam_data });

        (frame, down_id, up_id)
    }

    /// Creates a new beam between an existing vertex and a position.
    ///
    /// The up vertex of the returned beam id is always the new vertex.
    ///
    /// Panics if the vertex id is invalid.
    pub fn new_beam_extend(
        &mut self,
        allocator: &mut FrameIdAllocator,
        vertex_id: VertexId,
        position: Vec3,
        beam_data: B,
    ) -> VertexId {
        let new_vertex_id = allocator.next();

        // `vertex_id` is always the down vertex
        let beam_id = BeamId::from_vertices(vertex_id, new_vertex_id);

        let vertex = self
            .get_vertex_mut(vertex_id)
            .expect("Invalid vertex id given.");

        vertex.connections.push(BeamEnd {
            beam_id,
            beam_end: BeamDirection::Down,
        });

        let None = self.vertices.insert(
            new_vertex_id,
            Vertex {
                position,
                connections: vec![BeamEnd {
                    beam_id,
                    beam_end: BeamDirection::Up,
                }],
            },
        ) else {
            panic!("Duplicate id created with allocator");
        };

        let None = self.beams.insert(beam_id, Beam { beam_data }) else {
            panic!("Duplicate id created with allocator");
        };

        new_vertex_id
    }

    /// Creates a new beam by joining two vertices.
    ///
    /// Panics if either vertex id is invalid or if
    /// a beam already exists between the vertices.
    pub fn new_beam_join(&mut self, vertex_a: VertexId, vertex_b: VertexId, beam_data: B) {
        let beam_id = BeamId::from_vertices(vertex_a, vertex_b);

        let (down_id, up_id) = if vertex_a < vertex_b {
            (vertex_a, vertex_b)
        } else {
            (vertex_b, vertex_a)
        };

        self.get_vertex_mut(down_id)
            .expect("Invalid vertex id given.")
            .connections
            .push(BeamEnd {
                beam_id,
                beam_end: BeamDirection::Down,
            });

        self.get_vertex_mut(up_id)
            .expect("Invalid vertex id given.")
            .connections
            .push(BeamEnd {
                beam_id,
                beam_end: BeamDirection::Up,
            });

        let None = self.beams.insert(beam_id, Beam { beam_data }) else {
            panic!("Beam already existed");
        };
    }

    /// Removes a beam, possibly creating a new frame.
    ///
    /// Panics if the given beam id is invalid, or if it is the last beam in the graph.
    pub fn remove_beam(&mut self, beam: BeamId) -> Option<Self> {
        self.beams
            .swap_remove(&beam)
            .expect("Invalid beam id given");

        if self.beams.is_empty() {
            panic!("Last beam removed from frame, empty frame should not exist.");
        }

        let mut empty_vertex_found = false;

        let start_vertex = self.get_vertex_mut(beam.down_vertex()).unwrap();
        start_vertex.remove_connection(beam);
        let start_position = start_vertex.position;

        if start_vertex.connections().is_empty() {
            self.vertices.swap_remove(&beam.down_vertex());
            empty_vertex_found = true;
        }

        let end_vertex = self.get_vertex_mut(beam.up_vertex()).unwrap();
        end_vertex.remove_connection(beam);
        let end_position = end_vertex.position;

        if end_vertex.connections().is_empty() {
            self.vertices.swap_remove(&beam.up_vertex());
            empty_vertex_found = true;
        }

        if empty_vertex_found {
            return None;
        }

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
            vertex_id: beam.down_vertex(),
        }];

        best_costs.insert(beam.down_vertex(), 0.);

        loop {
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

            // Check if end is reached.
            if vertex_id == beam.up_vertex() {
                return None;
            }

            // Mark vertex as visited.
            visited.insert(vertex_id);

            let vertex = self.get_vertex(vertex_id).unwrap();

            let &path_cost = best_costs
                .get(&vertex_id)
                .expect("Vertex should not be in queue if it is in map.");

            // Check if this path was already explored with a lower cost
            if path_cost < cost {
                continue;
            }

            for connection in vertex.connections() {
                let opposite_vertex_id = connection.opposite();
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

        let mut new_frame = FrameGraph::empty();

        for vertex_id in visited {
            let vertex = self.vertices.swap_remove(&vertex_id).unwrap();

            // Move over all beams once by copying them when seeing
            for &BeamEnd { beam_id, beam_end } in vertex.connections() {
                let BeamDirection::Down = beam_end else {
                    continue;
                };

                new_frame
                    .beams
                    .insert(beam_id, self.beams.swap_remove(&beam_id).unwrap());
            }

            new_frame.vertices.insert(vertex_id, vertex);
        }

        Some(new_frame)
    }

    pub fn get_vertex(&self, vertex_id: VertexId) -> Option<&Vertex> {
        self.vertices.get(&vertex_id)
    }

    pub fn get_vertex_mut(&mut self, vertex_id: VertexId) -> Option<&mut Vertex> {
        self.vertices.get_mut(&vertex_id)
    }
}

impl<B> std::fmt::Debug for FrameGraph<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FrameGraph {{ vertices: {}, beams: {} }}",
            self.vertices.len(),
            self.beams.len()
        )
    }
}
