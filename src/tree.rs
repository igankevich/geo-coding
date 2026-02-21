use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "std")]
mod io;

const EMPTY: u32 = 0;

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
struct Node<C, V> {
    location: [C; 2],
    value: V,
    lesser_index: u32,
    greater_index: u32,
}

/// Two-dimensional tree that maps a location given by `[i64; 2]` to a string.
pub type NamesTree = Tree2D<i64, String>;

/// Two-dimensional tree that maps a location given by `[C; 2]` to a value `V`.
#[derive(Debug, PartialEq, Eq)]
pub struct Tree2D<C, V> {
    nodes: Vec<Node<C, V>>,
}

impl<C: Ord + Copy + Default, V: Default> Tree2D<C, V> {
    /// Create a new tree from the given nodes.
    ///
    /// The nodes are recursively subdividded into two groups: one that is behind and one that is
    /// in front of the plane that goes through the median node.
    /// The plane alternates between _x = 0_ and _y = 0_ for each layer of the tree.
    ///
    /// The values are moved from the vector without copying.
    pub fn from_nodes(mut nodes: Vec<([C; 2], V)>) -> Self {
        assert!(nodes.len() < u32::MAX as usize);
        let mut output_nodes = Vec::with_capacity(nodes.len());
        for _ in 0..nodes.len() {
            output_nodes.push(Node {
                location: Default::default(),
                value: Default::default(),
                lesser_index: EMPTY,
                greater_index: EMPTY,
            });
        }
        let mut output_node_index: u32 = 1;
        let mut next_output_node_index = || {
            let i = output_node_index;
            output_node_index += 1;
            i
        };
        let mut queue = VecDeque::new();
        queue.push_back((0, next_output_node_index(), nodes.as_mut_slice()));
        while let Some((coord_index, i, nodes)) = queue.pop_front() {
            let next_coord_index = (coord_index + 1) % 2;
            let nodes_len = nodes.len();
            if nodes_len == 0 {
                break;
            }
            if nodes_len == 1 {
                output_nodes[(i - 1) as usize] = Node {
                    location: nodes[0].0,
                    value: core::mem::take(&mut nodes[0].1),
                    lesser_index: EMPTY,
                    greater_index: EMPTY,
                };
                continue;
            }
            let (lesser_nodes, median, greater_nodes) = nodes
                .select_nth_unstable_by(nodes_len / 2, |a, b| {
                    a.0[coord_index].cmp(&b.0[coord_index])
                });
            let lesser_index = if !lesser_nodes.is_empty() {
                let i = next_output_node_index();
                queue.push_back((next_coord_index, i, lesser_nodes));
                i
            } else {
                EMPTY
            };
            let greater_index = if !greater_nodes.is_empty() {
                let i = next_output_node_index();
                queue.push_back((next_coord_index, i, greater_nodes));
                i
            } else {
                EMPTY
            };
            output_nodes[(i - 1) as usize] = Node {
                location: median.0,
                value: core::mem::take(&mut median.1),
                lesser_index,
                greater_index,
            };
        }
        Self {
            nodes: output_nodes,
        }
    }

    /// Returns up to `max_neighbours` nodes within `max_distance` that are closest to the `location`.
    ///
    /// The distance between nodes is computed using `calc_distance`.
    pub fn find_nearest<D>(
        &self,
        location: &[C; 2],
        mut max_distance: D,
        max_neighbours: usize,
        mut calc_distance: impl FnMut(&[C; 2], &[C; 2]) -> D,
    ) -> Vec<(D, &[C; 2], &V)>
    where
        D: Ord + Copy + core::fmt::Display,
    {
        let mut neighbours = Vec::new();
        if max_neighbours == 0 {
            return neighbours;
        }
        // TODO optimize for max_neighbours == 1
        let Some(root) = self.nodes.first() else {
            return neighbours;
        };
        let mut queue = VecDeque::new();
        queue.push_back((0, root));
        while let Some((coord_index, node)) = queue.pop_front() {
            let d = calc_distance(&node.location, location);
            let mut lesser = false;
            let mut greater = false;
            if d.le(&max_distance) {
                match neighbours.binary_search_by(|(distance, ..)| distance.cmp(&d)) {
                    Err(i) if i == max_neighbours => {}
                    Ok(i) | Err(i) => {
                        if neighbours.len() == max_neighbours {
                            neighbours.pop();
                        }
                        neighbours.insert(i, (d, &node.location, &node.value));
                    }
                }
                if neighbours.len() == max_neighbours {
                    // We've already found enough neighbours; now we can limit our search to the
                    // ones that are closer than the closest one found so far.
                    max_distance = neighbours[0].0;
                }
                lesser = true;
                greater = true;
            } else if location[coord_index] < node.location[coord_index] {
                lesser = true;
            } else {
                greater = true;
            }
            let next_coord_index = (coord_index + 1) % 2;
            if lesser && node.lesser_index != EMPTY {
                queue.push_back((
                    next_coord_index,
                    &self.nodes[(node.lesser_index - 1) as usize],
                ));
            }
            if greater && node.greater_index != EMPTY {
                queue.push_back((
                    next_coord_index,
                    &self.nodes[(node.greater_index - 1) as usize],
                ));
            }
        }
        neighbours
    }

    /// Returns an iterator over nodes.
    pub fn iter(&self) -> impl Iterator<Item = (&[C; 2], &V)> {
        self.nodes.iter().map(|node| (&node.location, &node.value))
    }

    /// Returns the number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::euclidean_distance_squared;
    use alloc::vec;

    #[test]
    fn tree_works() {
        let tree = Tree2D::from_nodes(vec![
            ([0_i64, 0], ()), //
            ([-1, 0], ()),    //
            ([1, 0], ()),     //
            ([2, 0], ()),     //
            ([3, 0], ()),     //
        ]);
        let neighbours = tree.find_nearest(&[5, 0], 25_u64, 1, euclidean_distance_squared);
        assert_eq!(vec![(4, &[3, 0], &())], neighbours);
    }
}
