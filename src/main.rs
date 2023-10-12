use actix_web::{get, put, web, App, HttpResponse, HttpServer, Responder};
use serde_derive::Serialize;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

const KEY_SIZE: u32 = 6;
const CLUSTER_SIZE: u32 = 2u32.pow(KEY_SIZE);

/// Structure to represent a node in the cluster, with its id, ip and hashmap
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Node {
    id: u32,
    ip: String,
    hashmap: HashMap<String, String>,
    resp_keys: Vec<u32>,
}
/// Structure to represent the neighbors of a node
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Neighbors {
    prev: String,
    next: String,
}

/// Implementation of the Node structure
impl Node {
    fn new(id: u32, ip: String) -> Self {
        Node {
            id,
            ip,
            hashmap: HashMap::new(),
            resp_keys: Vec::new(),
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

/// Hashes the given IP address using SHA1 and maps it to a node ID.
///
/// ### Arguments
///
/// * `ip` - A string slice that holds the IP address to be hashed.
///
/// ### Returns
///
/// A `u32` value representing the node ID.
fn hash_function(ip: String) -> u32 {
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
fn populate_fingertable(node_id: u32, number_of_nodes: u32) -> Vec<(u32, Node)> {
    let mut nodes: HashSet<Node> = HashSet::new();
    let mut finger_table: Vec<(u32, Node)> = Vec::new();

    // read the ip.txt file to get the ip addresses of the nodes
    if let Ok(lines) = fs::read_to_string("ip.txt") {
        let lines_iter = lines.lines().take(number_of_nodes as usize);

        // parse each line to get the id and ip address of the node
        for line in lines_iter {
            // if the line is not empty
            if let Some((id, ip)) = parse_line(line) {
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
fn parse_line(line: &str) -> Option<(u32, &str)> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() == 3 {
        let ip = parts[2];
        let id = hash_function(ip.to_string());
        Some((id, ip))
    } else {
        None
    }
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
fn get_previous_node(node_id: u32, number_of_nodes: u32) -> Node {
    let mut nodes: HashSet<Node> = HashSet::new();

    // read the ip.txt file to get the ip addresses of the nodes
    if let Ok(lines) = fs::read_to_string("ip.txt") {
        let lines_iter = lines.lines().take(number_of_nodes as usize);

        for line in lines_iter {
            if let Some((id, ip)) = parse_line(line) {
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
fn fill_hashmap(node: &mut Node, previous_id: u32, current_id: u32) {
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

/// Returns the local IP address of the machine, or None if it cannot be determined.
///
/// ### Arguments
///
/// This function takes no arguments.
///
/// ### Returns
///
/// This function returns an `Option<IpAddr>`. If the local IP address of the machine can be determined,
/// it will be returned as an `IpAddr` wrapped in a `Some` variant. If it cannot be determined, `None`
/// will be returned.
fn get_local_ip() -> Option<IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok()?.ip().into()
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
async fn find_succesor(key: u32, finger_table: Vec<(u32, Node)>, current_id: u32) -> String {
    let mut succesor = String::new();

    let succesor_id = finger_table[0].1.id;

    if succesor_id < current_id {
        for i in current_id..=CLUSTER_SIZE {
            if i == key {
                succesor = finger_table[0].1.ip.clone();
                break;
            }
        }

        if succesor.is_empty() {
            for i in 0..=succesor_id {
                if i == key {
                    succesor = finger_table[0].1.ip.clone();
                    break;
                }
            }
        }
    } else {
        for i in current_id..succesor_id {
            if i == key {
                succesor = finger_table[0].1.ip.clone();
                break;
            }
        }
    }

    //sort the finger table by id
    let mut sorted_finger_table = finger_table.clone();
    sorted_finger_table.sort_by_key(|finger| finger.1.id);

    if succesor.is_empty() {
        for finger in sorted_finger_table.iter() {
            if finger.1.id <= key {
                // println!("Finger: {:?}", finger);
                succesor = finger.1.ip.clone();
            }
        }
    }

    if succesor.is_empty() {
        succesor = sorted_finger_table[sorted_finger_table.len() - 1]
            .1
            .ip
            .clone();
    }
    succesor
}

/// This function is the entry point of the Chord distributed hash table node.
/// It initializes the node by getting the local IP address, calculating the node ID,
/// populating the fingertable, getting the previous node, and formatting the IP addresses
/// and ports of the node, previous node, and nodes in the fingertable. It then creates a
/// new node with the node ID and IP address with port number, fills the hashmap, and starts
/// the HTTP server. The server listens on port 55000 and serves the index, item_get,
/// and item_put services.
///
/// ### Arguments
///
/// None
///
/// ### Returns
///
/// This function returns a `std::io::Result<()>` which indicates whether the operation was successful or not.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "todos=info");
    }
    pretty_env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Provde number of nodes as argument");
        std::process::exit(1);
    }

    let num_nodes: u32 = args[1].parse().unwrap();

    // get the local ip address of the node
    let local_ip: IpAddr = get_local_ip().unwrap();

    // get the node id of the node
    let node_id = hash_function(local_ip.to_string());

    // format the ip address and port of the node
    let ip_and_port = format!("{}:{}", local_ip.to_string(), 65000);

    // create a new node with the node id and ip address with port number
    let mut node = Node::new(node_id, ip_and_port);

    // populate the fingertable of the node
    let mut finger_table = populate_fingertable(node_id, num_nodes);

    // get the previous node of the current node in the cluster
    let mut previous_node = get_previous_node(node_id, num_nodes);

    // format the ip address and port of the previous node
    previous_node.ip = format!("{}:{}", previous_node.ip, 65000);

    // format the ip address and port of the nodes in the fingertable
    for finger in finger_table.iter_mut() {
        finger.1.ip = format!("{}:{}", finger.1.ip, 65000);
    }

    if num_nodes == 1 {
        node.resp_keys = (0..CLUSTER_SIZE).collect();
        previous_node = node.clone();
        finger_table = Vec::new();
        finger_table.push((node_id, node.clone()));
    }

    fill_hashmap(&mut node, previous_node.id, node_id);

    let node_data = web::Data::new(Arc::new(Mutex::new(node)));
    let previous_node_data = web::Data::new(Mutex::new(previous_node));
    let finger_table_data = web::Data::new(Mutex::new(finger_table));

    println!("Server starting at http:// {}", local_ip.to_string());

    HttpServer::new(move || {
        App::new()
            .app_data(node_data.clone())
            .app_data(previous_node_data.clone())
            .app_data(finger_table_data.clone())
            .service(index)
            .service(item_get)
            .service(item_put)
    })
    .bind("0.0.0.0:65000")?
    .run()
    .await?;

    Ok(())
}

/// Handles GET requests to retrieve the IP addresses of the current node's neighbors in the Chord ring.
///
/// ### Arguments
///
/// * `finger_table` - A web::Data<Mutex<Vec<(u32, Node)>>> representing the finger table of the current node.
/// * `prev_node` - A web::Data<Mutex<Node>> representing the previous node in the Chord ring.
///
/// ### Returns
///
/// Returns a JSON response containing the IP addresses of the current node's neighbors.
#[get("/storage/neighbors")]
async fn index(
    finger_table: web::Data<Mutex<Vec<(u32, Node)>>>,
    prev_node: web::Data<Mutex<Node>>,
) -> impl Responder {
    let finger_table = finger_table.lock().unwrap();
    let prev_node = prev_node.lock().unwrap();
    let neightbors = Neighbors {
        prev: prev_node.ip.to_string(),
        next: finger_table[0].1.ip.clone().to_string(),
    };
    let ip_addr = vec![neightbors.prev, neightbors.next];
    HttpResponse::Ok().json(ip_addr)
}

/// Handles PUT requests for a specific key in the storage.
/// If the key exists, updates the value. Otherwise, sends the request to the successor node.
///
/// ### Arguments
///
/// * `key` - A String representing the key to be updated or inserted.
/// * `data` - A String representing the value to be updated or inserted.
/// * `node_data` - A web::Data<Mutex<Node>> representing the current node's data.
/// * `finger_table` - A web::Data<Mutex<Vec<(u32, Node)>>> representing the current node's finger table.
///
/// ### Returns
///
/// An HttpResponse indicating whether the item was updated or inserted successfully.
#[put("/storage/{key}")]
async fn item_put(
    key: web::Path<String>,
    data: String,
    node_data: web::Data<Arc<Mutex<Node>>>,
    finger_table: web::Data<Mutex<Vec<(u32, Node)>>>,
) -> impl Responder {
    let hash_ref = &key;
    let hashed_key = hash_function(hash_ref.to_string());

    let node_ref = node_data.get_ref();

    let mut node = node_ref.lock().unwrap();

    if node.resp_keys.contains(&hashed_key) {
        node.hashmap.insert(key.to_string(), data);
        HttpResponse::Ok().body(format!("Item {:?} updated", key))
    } else {
        let succesor =
            find_succesor(hashed_key, finger_table.lock().unwrap().clone(), node.id).await;

        let url = format!("http://{}/storage/{}", succesor, key);
        let client = reqwest::Client::new();
        let _res = client.put(&url).body(data).send().await;

        HttpResponse::Ok().body(format!("Item {:?} inserted", key))
    }
}

/// Handler for GET requests to retrieve an item from the node's storage.
///
/// ### Arguments
///
/// * `key` - The key of the item to retrieve.
/// * `node_data` - The node's data, containing a mutex-protected hashmap of stored items.
/// * `finger_table` - The node's finger table, containing a mutex-protected vector of finger table entries.
///
/// ### Returns
///
/// Returns an HTTP response with the retrieved item if it exists, or a 404 Not Found response if it does not.
#[get("/storage/{key}")]
async fn item_get(
    key: web::Path<String>,
    node_data: web::Data<Arc<Mutex<Node>>>,
    finger_table: web::Data<Mutex<Vec<(u32, Node)>>>,
) -> impl Responder {
    let node_ref = node_data.get_ref();

    let node = node_ref.lock().unwrap();
    let hash_ref = &key;
    let hashed_key = hash_function(hash_ref.to_string());

    if node.resp_keys.contains(&hashed_key) {
        if let Some(value) = node.hashmap.get(&key.to_string()) {
            HttpResponse::Ok().body(value.to_string())
        } else {
            HttpResponse::NotFound().body("Key not found")
        }
    } else {
        let succesor =
            find_succesor(hashed_key, finger_table.lock().unwrap().clone(), node.id).await;

        let url = format!("http://{}/storage/{}", succesor, key);
        let client = reqwest::Client::new();

        let res = client.get(&url).send().await;

        if let Ok(response) = res {
            if response.status() == 404 {
                HttpResponse::NotFound().body("Key not found")
            } else {
                HttpResponse::Ok().body(response.text().await.unwrap())
            }
        } else {
            HttpResponse::InternalServerError().finish()
        }
    }
}
