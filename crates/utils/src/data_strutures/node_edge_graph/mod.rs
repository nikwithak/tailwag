use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

type NodeId = usize;
type EdgeId = usize;

pub enum Error {
    NodeDoesNotExist,
}

pub struct NodeData<N, E> {
    id: NodeId,
    data: N,
    _edges: PhantomData<E>,
}

impl<N, E> NodeData<N, E> {
    // pub fn edges(&self) -> impl Iterator<Item = EdgeData<E>> {
    //     todo!()
    // }
    pub fn connect_to(
        &self,
        dest: impl Into<NodeId>,
    ) -> Result<(), Error> {
        todo!()
    }
}
pub struct EdgeData<E> {
    id: EdgeId,
    from: NodeId,
    to: NodeId,
    data: E,
    bidirectional: bool,
}

pub struct NodeEdgeGraph<N, E> {
    root: NodeId,
    node_data: Vec<NodeData<N, E>>,
    edge_data: Vec<EdgeData<E>>,
}

impl<N, E> NodeEdgeGraph<N, E> {
    pub fn new(root: N) -> Self {
        let root_id = 0;
        Self {
            root: root_id,
            node_data: vec![NodeData {
                id: root_id,
                data: root,
                _edges: PhantomData,
            }],
            edge_data: vec![],
        }
    }
}

impl<N, E> NodeEdgeGraph<N, E> {
    // All operations are atomic.
    fn insert(
        &mut self,
        data: N,
    ) -> NodeId {
        let id = self.node_data.len();
        let node = NodeData::<N, E> {
            id,
            data,
            _edges: PhantomData,
        };
        self.node_data.push(node);
        id
    }
    fn get(
        &self,
        id: NodeId,
    ) -> Option<&NodeData<N, E>> {
        self.node_data.get(id)
    }
    fn get_mut(
        &mut self,
        id: NodeId,
    ) -> Option<&mut NodeData<N, E>> {
        self.node_data.get_mut(id)
    }
    fn insert_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        data: E,
    ) -> EdgeId {
        let id = self.node_data.len();
        let edge = EdgeData {
            id,
            from,
            to,
            data,
            bidirectional: true,
        };
        self.edge_data.push(edge);
        id
    }
}

pub trait Visitor<N, E> {
    fn visit(
        &self,
        node: NodeData<N, E>,
    );
}

impl<N, E> NodeEdgeGraph<N, E> {
    pub fn traverse_bfs(visitor: impl Visitor<N, E>) {
        todo!()
    }
    pub fn traverse_dfs() {
        todo!()
    }
    pub fn find_predicate() {
        todo!()
    }
}
