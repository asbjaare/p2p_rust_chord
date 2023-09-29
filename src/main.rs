use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use actix_web::{get, put, web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use serde_derive::Serialize;
use actix_web::rt::System;

const KEY_SIZE: u32 = 4;
const CLUSTER_SIZE: u32 = 2u32.pow(KEY_SIZE);

// Structure to represent a node in the cluster, with its id, ip and hashmap
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Node {
    id: u32,
    ip: String,
    hashmap: HashMap<u32, String>,
}
// Structure to represent the neighbors of a node
#[derive(Serialize,Debug, Clone, Eq, PartialEq)]
pub struct Neighbors {
    prev: String,
    next: String
}

// Implementation of the Node structure
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

// Implementation of the Display trait for the Node structure
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node {{ id: {}, ip: {} }}", self.id, self.ip)
    }
}

// consistant hash function to map the ip address to a node id with SHA1
fn hash_function(ip: String) -> u32 {
    let mut hasher = Sha1::new();
    hasher.update(ip);
    let result = hasher.finalize();
    let bytes = result.as_slice();
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes[0..4]);
    u32::from_be_bytes(buf) % CLUSTER_SIZE

  
}

// populate the fingertable of a node
fn populate_fingertable(node_id: u32, number_of_nodes: u32) -> Vec<(u32, Node)> {
    let mut nodes: HashSet<Node> = HashSet::new();
    let mut finger_table: Vec<(u32, Node)> = Vec::new();
  

    // read the ip.txt file to get the ip addresses of the nodes
    if let Ok(lines) = fs::read_to_string("ip.txt") {
        
        let lines_iter  = lines.lines().take(number_of_nodes as usize);

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

// parse a line of the ip.txt file to get the id and ip address of a node
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

// get the previous node of the current node in the cluster
fn get_previous_node(node_id: u32, number_of_nodes: u32) -> Node {
    let mut nodes: HashSet<Node> = HashSet::new();

    // read the ip.txt file to get the ip addresses of the nodes
    if let Ok(lines) = fs::read_to_string("ip.txt") {

        let lines_iter  = lines.lines().take(number_of_nodes as usize);

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

// fill the hashmap of a node with the keys and values
fn fill_hashmap(node: &mut Node, previous_id: u32, current_id: u32) {
   
   // if the previous id is less than the current id, fill the hashmap with the keys between the previous id and the current id. 
   // else fill the hashmap with the keys between the previous id and the cluster size, and then between 0 and the current id
    let (start_id, end_id) = if previous_id < current_id {
        (previous_id + 1, current_id)
    } else {
        (previous_id + 1, CLUSTER_SIZE)
    };
    for key_id in start_id..=end_id {
        node.hashmap.insert(key_id, format!("value_{}", key_id));
    }

    if previous_id > current_id {
        for key_id in 0..=current_id  {
            node.hashmap.insert(key_id, format!("value_{}", key_id));
        }
    }
}

// get the local ip address of the node
fn get_local_ip() ->Option<IpAddr>{
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok()?.ip().into()
}

//Function to find next succesor if the current node does not have the key. (Under construction)
async fn find_succesor(key: u32, finger_table: Vec<(u32, Node)> ) -> String{
    let mut succesor = String::new();

    // println!("Finger table: {:?}", finger_table);

    for finger in finger_table.iter() {
        if finger.0 == key {
            succesor = finger.1.ip.clone();
        }
    }

    
    if succesor.is_empty() {

        for finger in finger_table.iter(){
            if finger.0 > key {
                succesor = finger.1.ip.clone();
                break;
            }
            else {
                succesor = finger_table[0].1.ip.clone();
            }
       
        }
    }
    
    println!("Succesor: {}", succesor);


    succesor
}

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
    let ip_and_port = format!("{}:{}", local_ip.to_string(), 55000);

    // create a new node with the node id and ip address with port number
    let mut node = Node::new(node_id, ip_and_port);
    
    // populate the fingertable of the node
    let mut  finger_table = populate_fingertable(node_id, num_nodes);
    
    // get the previous node of the current node in the cluster
    let mut previous_node = get_previous_node(node_id, num_nodes);

    // format the ip address and port of the previous node
    previous_node.ip = format!("{}:{}", previous_node.ip, 55000);
    
    // format the ip address and port of the nodes in the fingertable
    for finger in finger_table.iter_mut() {
        finger.1.ip = format!("{}:{}", finger.1.ip, 55000);
    }
    
    // println!("Node: {}", node);
    // println!("Previous node: {}", previous_node);
    // println!("Finger table: {:?}", finger_table);
    
    
    
    fill_hashmap(&mut node, previous_node.id, node_id);
    // println!("hashmap: {:?}", node.hashmap);
    
    let node_data = web::Data::new(Mutex::new(node));
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
    .bind("0.0.0.0:55000")?
    .run()
    .await?;
  
    Ok(())

   

}



#[get("/storage/neighbors")]
async fn index(finger_table: web::Data<Mutex<Vec<(u32, Node)>>>, prev_node: web::Data<Mutex<Node>>  ) -> impl Responder {
    println!("Received GET request"); 
    let finger_table = finger_table.lock().unwrap();
    let prev_node = prev_node.lock().unwrap();
    let neightbors = Neighbors {
        prev: prev_node.ip.to_string(),
        next: finger_table[0].1.ip.clone().to_string(),
    };
    let ip_addr = vec![neightbors.prev, neightbors.next];
    HttpResponse::Ok().json(ip_addr)
}

#[put("/storage/{key}")]
async fn item_put(key: web::Path<String>, data:String  ,node_data: web::Data<Mutex<Node>>, finger_table: web::Data<Mutex<Vec<(u32, Node)>>>) -> impl Responder {

    let hash_ref = &key;
    let hashed_key = hash_function(hash_ref.to_string());
    println!("Received PUT request for key: {}", hashed_key);

    let mut node = node_data.lock().unwrap();
    if node.hashmap.contains_key(&hashed_key) {
        
        node.hashmap.insert(hashed_key, data);
        HttpResponse::Ok().body(format!("Item {:?} updated", node.hashmap))
        
    } 
    else {

     
        
        
        
        //Sends API call to succesor. (Under construction)
        let succesor = find_succesor(hashed_key, finger_table.lock().unwrap().clone()).await;
       

        let url = format!("http://{}/storage/{}", succesor, key);
        let client = reqwest::Client::new();
        let res = client.put(&url).body(data).send().await;
        println!("Response: {:?}", res);


        HttpResponse::Ok().body(format!("Item {:?} inserted", res ))
    }

}

#[get("/storage/{key}")]
async fn item_get(key: web::Path<u32>, node_data: web::Data<Mutex<Node>>, finger_table: web::Data<Mutex<Vec<(u32, Node)>>>) -> impl Responder {

let node = node_data.lock().unwrap();
let hash_ref = &key;
let hashed_key = hash_function(hash_ref.to_string());


if let Some(value) = node.hashmap.get(&hashed_key) {
    HttpResponse::Ok().body(format!("Item {:?} found", value))
} else {

    let succesor = find_succesor(hashed_key, finger_table.lock().unwrap().clone()).await;

    let url = format!("http://{}/storage/{}", succesor, key);
    let client = reqwest::Client::new();
    println!("Test");
    let res = client.get(&url).send().await;

    HttpResponse::Ok().body(format!("Item {:?} not found", res))

}


}
