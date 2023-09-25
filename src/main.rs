use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use actix_web::{get, put, web, App, HttpResponse, HttpServer, Responder};
use std::sync::{Arc, Mutex};

const KEY_SIZE: u32 = 4;
const CLUSTER_SIZE: u32 = 2u32.pow(KEY_SIZE);

#[derive(Debug, Clone, Eq, PartialEq)]
struct Node {
    id: u32,
    ip: String,
    hashmap: HashMap<u32, String>,

}

impl Node {
    fn new(id: u32, ip: String) -> Self {
        Node { id, ip, hashmap: HashMap::new() }
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node {{ id: {}, ip: {} }}", self.id, self.ip)
    }
}

fn hash_function(ip: &IpAddr) -> u32 {
    let mut hasher = Sha1::new();
    hasher.update(ip.to_string());
    let result = hasher.finalize();
    let bytes = result.as_slice();
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes[0..4]);
    u32::from_be_bytes(buf) % CLUSTER_SIZE

  
}

fn populate_fingertable(node_id: u32) -> Vec<(u32, Node)> {
    let mut nodes: HashSet<Node> = HashSet::new();
    let mut finger_table: Vec<(u32, Node)> = Vec::new();

    if let Ok(lines) = fs::read_to_string("ip.txt") {
        
        for line in lines.lines() {
            if let Some((id, ip)) = parse_line(line) {
                let node = Node::new(id, ip.to_string());
                
                nodes.insert(node);
            }
        }
    
        
     
        let mut sorted_nodes: Vec<Node> = nodes.into_iter().collect();
        sorted_nodes.sort_by_key(|node| node.id);

        for i in 0..KEY_SIZE {
            let start = (node_id + 2u32.pow(i)) % CLUSTER_SIZE;

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

fn parse_line(line: &str) -> Option<(u32, &str)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
  
    if parts.len() == 3 {
        let ip = parts[2];
        let id = hash_function(&ip.parse::<IpAddr>().unwrap());
        Some((id, ip))
    } else {
        None
    }
}

fn get_previous_node(node_id: u32) -> Node {
    let mut nodes: HashSet<Node> = HashSet::new();

    if let Ok(lines) = fs::read_to_string("ip.txt") {
        for line in lines.lines() {
            if let Some((id, ip)) = parse_line(line) {
                let node = Node::new(id, ip.to_string());
                nodes.insert(node);
            }
        }
    }

    let mut sorted_nodes: Vec<Node> = nodes.into_iter().collect();
    sorted_nodes.sort_by_key(|node| node.id);

    let mut previous_node = sorted_nodes[0].clone();

    for node in sorted_nodes {
        if node.id < node_id {
            previous_node = node;
        }
    }

    previous_node
}


#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

// #[put("/item/<key>")]
// async fn item_put(key: u32, node: web::Data<Node>) -> impl Responder {

//     let mut node = &mut *node.lock().unwrap();
//     node.hashmap.insert(key, "test".to_string());
//     HttpResponse::Ok().body("node, {}", node)
    
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "todos=info");
    }
    pretty_env_logger::init();

  
    

    let local_ip: IpAddr = get_local_ip().unwrap();


    let node_id = hash_function(&local_ip);


    let mut node = Node::new(node_id, local_ip.to_string());
    println!("node: {}", node);

    let finger_table = populate_fingertable(node_id);
    println!("finger_table: {:?}", finger_table);

    let previous_node = get_previous_node(node_id);
    println!("previous_node: {}", previous_node);

    node.hashmap.insert(1, "test".to_string());
    println!("hashmap: {:?}", node.hashmap);

    HttpServer::new(|| {
        App::new()
        .service(index)
    })
    .bind("0.0.0.0:65000")?
    .run()
    .await
  

   

}

fn get_local_ip() ->Option<IpAddr>{
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok()?.ip().into()
}

