use serde_derive::Serialize;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};


const KEY_SIZE: u32 = 20;
const CLUSTER_SIZE: u32 = 2u32.pow(KEY_SIZE);

/// Structure to represent a node in the cluster, with its id, ip and hashmap
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Node {
    pub id: u32,
    pub ip: String,
    pub hashmap: HashMap<String, String>,
    pub resp_keys: Vec<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct NodePrev {
    pub id: u32,
    pub ip: String,
}
#[derive(Debug, Serialize, Clone)]
pub struct Succ_list_node {
    id: u32,
    ip: String,
}


impl NodePrev {
    pub fn new(id: u32, ip: String) -> Self {
        NodePrev { id, ip }
    }
}

/// Parses a line of the ip.txt file to get the id and ip address of a node.
///
/// ### Arguments
///
/// * `line` - A string slice that holds the line to be parsed.
///
/// ### Returns
///
/// * `Some((id, ip))` - A tuple containing the id and ip address of the node if the line is correctly formatted.
/// * `None` - If the line is not correctly formatted.
fn _parse_line(line: &str) -> Option<(u32, &str)> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() == 3 {
        let ip = parts[2];
        let id = Node::hash_function(ip.to_string());
        Some((id, ip))
    } else {
        None
    }
}

/// Implementation of the Node structure
impl Node {
    pub fn new(id: u32, ip: String) -> Self {
        Node {
            id,
            ip,
            hashmap: HashMap::new(),
            resp_keys: Vec::new(),
        }
    }

    /// Hashes the given IP address using SHA1 and maps it to a node ID.
    ///
    /// ### Arguments
    ///
    /// * `ip` - A string slice that holds the IP address to be hashed.
    ///
    /// ### Returns
    ///
    /// A `u32` value representing the node ID.
    pub fn hash_function(ip: String) -> u32 {
        let mut hasher = Sha1::new();
        hasher.update(ip);
        let result = hasher.finalize();
        let bytes = result.as_slice();
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&bytes[0..4]);
        u32::from_be_bytes(buf) % CLUSTER_SIZE
    }

    /// Populates the fingertable of a node.
    ///
    /// ### Arguments
    ///
    /// * `node_id` - An unsigned 32-bit integer representing the id of the node.
    ///
    /// ### Returns
    ///
    /// A vector of tuples containing the start key and the successor node for each entry in the fingertable.
    ///
    /// ### Example
    ///
    /// ```
    /// let fingertable = populate_fingertable(42);
    /// ```
    pub fn _populate_fingertable(node_id: u32, number_of_nodes: u32) -> Vec<(u32, Node)> {
        let mut nodes: HashSet<Node> = HashSet::new();
        let mut finger_table: Vec<(u32, Node)> = Vec::new();

        // read the ip.txt file to get the ip addresses of the nodes
        if let Ok(lines) = fs::read_to_string("ip.txt") {
            let lines_iter = lines.lines().take(number_of_nodes as usize);

            // parse each line to get the id and ip address of the node
            for line in lines_iter {
                // if the line is not empty
                if let Some((id, ip)) = _parse_line(line) {
                    let node = Node::new(id, ip.to_string());

                    nodes.insert(node);
                }
            }

            // sort the nodes by id
            let mut sorted_nodes: Vec<Node> = nodes.into_iter().collect();
            sorted_nodes.sort_by_key(|node| node.id);

            // println!("Sorted nodes: {:?}", sorted_nodes);

            // populate the fingertable of the node
            for i in 0..KEY_SIZE {
                let start = (node_id + 2u32.pow(i)) % CLUSTER_SIZE;

                // find the successor of the node
                let successor = sorted_nodes
                    .iter()
                    .find(|&node| node.id >= start)
                    .cloned()
                    .unwrap_or_else(|| sorted_nodes[0].clone());

                finger_table.push((start, successor));
            }
        }

        finger_table
    }

    /// Given a node id, this function returns the previous node in the cluster.
    /// It reads the ip.txt file to get the IP addresses of the nodes, sorts the nodes by id,
    /// and iterates over the sorted nodes to find the previous node of the current node.
    /// If the node with the given id is the first node in the cluster, the last node in the cluster is returned.
    ///
    /// ### Arguments
    ///
    /// * `node_id` - An unsigned 32-bit integer representing the id of the current node.
    ///
    /// ### Returns
    ///
    /// A `Node` struct representing the previous node in the cluster.
    pub fn _get_previous_node(node_id: u32, number_of_nodes: u32) -> Node {
        let mut nodes: HashSet<Node> = HashSet::new();

        // read the ip.txt file to get the ip addresses of the nodes
        if let Ok(lines) = fs::read_to_string("ip.txt") {
            let lines_iter = lines.lines().take(number_of_nodes as usize);

            for line in lines_iter {
                if let Some((id, ip)) = _parse_line(line) {
                    let node = Node::new(id, ip.to_string());
                    nodes.insert(node);
                }
            }
        }

        // sort the nodes by id
        let mut sorted_nodes: Vec<Node> = nodes.into_iter().collect();
        sorted_nodes.sort_by_key(|node| node.id);

        // set the previous node as the last node in the cluster
        let mut previous_node = sorted_nodes[sorted_nodes.len() - 1].clone();

        // find the previous node of the current node by iterating over the sorted nodes
        for node in sorted_nodes {
            if node.id < node_id {
                previous_node = node;
            }
        }

        previous_node
    }

    /// Fills the hashmap of a given node with keys between two given ids.
    /// If the previous id is less than the current id, the hashmap is filled with the keys between the previous id and the current id.
    /// If the previous id is greater than the current id, the hashmap is filled with the keys between the previous id and the cluster size, and then between 0 and the current id.
    ///
    /// ### Arguments
    ///
    /// * `node` - A mutable reference to the node whose hashmap is to be filled.
    /// * `previous_id` - The id of the previous node in the ring.
    /// * `current_id` - The id of the current node in the ring.
    pub async fn fill_hashmap(node: &mut Node, previous_id: u32, current_id: u32) {
        // if the previous id is less than the current id, fill the hashmap with the keys between the previous id and the current id.
        // else fill the hashmap with the keys between the previous id and the cluster size, and then between 0 and the current id
        let (start_id, end_id) = if previous_id < current_id {
            (previous_id + 1, current_id)
        } else {
            (previous_id + 1, CLUSTER_SIZE)
        };
        for key_id in start_id..=end_id {
            node.resp_keys.push(key_id);
        }

        if previous_id > current_id {
            for key_id in 0..=current_id {
                node.resp_keys.push(key_id);
            }
        }
    }

    // pub async fn find_succesor_to_joining_node(node_id: u32, finger_table: Vec<String>) -> String {
    //     let mut successor = String::new();

    //     let mut sorted_finger_table = finger_table.clone();
    //     sorted_finger_table.sort_by_key(|finger| Node::hash_function(finger.to_string()));

    //     for finger in sorted_finger_table.iter() {
    //         if Node::hash_function(finger.to_string()) > node_id {
    //             println!("Finger: {:?}", finger);
    //             successor = finger.to_string();
    //         }
    //     }

    //     successor
    // }

    // pub async fn is_between(id: u32, start: u32, end: u32) -> bool {
    //     if start <= end {
    //         id > start && id <= end
    //     } else {
    //         id > start || id <= end
    //     }
    // }

    pub async fn get_best_successor(
        key_id: u32,
        node_id: u32,
        succ1_node: Node,
        succ2_node: Node,
    ) -> Node {
        let dist1 = if succ1_node.id >= node_id {
            succ1_node.id - key_id
        } else {
            (u32::MAX - key_id) + succ1_node.id + 1
        };

        let dist2 = if succ2_node.id >= node_id {
            succ2_node.id - key_id
        } else {
            (u32::MAX - key_id) + succ2_node.id + 1
        };

        if dist1 <= dist2 {
            succ1_node
        } else {
            succ2_node
        }
    }

    /// Finds the successor node for a given key in the finger table.
    ///
    /// ### Arguments
    ///
    /// * `key` - A `u32` representing the key to find the successor for.
    /// * `Finger_table` - A `Vec` of tuples containing the finger table entries.
    ///
    /// ### Returns
    ///
    /// A `String` representing the IP address of the successor node.
    pub async fn find_successor_key(
        key: u32,
        finger_table: Vec<(u32, Node)>,
        current_id: u32,
    ) -> Option<String> {
        let mut successor = String::new();

        let successor_id = finger_table[0].1.id;

        if successor_id < current_id {
            for i in current_id..=CLUSTER_SIZE {
                if i == key {
                    successor = finger_table[0].1.ip.clone();
                    break;
                }
            }

            if successor.is_empty() {
                for i in 0..=successor_id {
                    if i == key {
                        successor = finger_table[0].1.ip.clone();
                        break;
                    }
                }
            }
        } else {
            for i in current_id..successor_id {
                if i == key {
                    successor = finger_table[0].1.ip.clone();
                    break;
                }
            }
        }

        // Sort the finger table by id
        let mut sorted_finger_table = finger_table.clone();
        sorted_finger_table.sort_by_key(|finger| finger.1.id);

        if successor.is_empty() {
            for finger in sorted_finger_table.iter() {
                if finger.1.id <= key {
                    successor = finger.1.ip.clone();
                }
            }
        }

        if successor.is_empty() {
            successor = sorted_finger_table[sorted_finger_table.len() - 1]
                .1
                .ip
                .clone();
        }

        if successor.is_empty() {
            None
        } else {
            Some(successor)
        }
    }


}

/// Implements the `Hash` trait for the `Node` struct.
///
/// This method hashes the `id` field of the `Node` struct using the provided `Hasher` instance.
impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Implements the Display trait for the Node structure.
///
/// This implementation allows for the Node structure to be printed in a formatted way.
/// It displays the id and ip of the Node.
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node {{ id: {}, ip: {} }}", self.id, self.ip)
    }
}
