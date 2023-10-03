use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use actix_web::{get, put, web, App, HttpResponse, HttpServer, Responder};
use std::sync::{Mutex, Arc};
use serde_derive::Serialize;

const KEY_SIZE: u32 = 6;
const CLUSTER_SIZE: u32 = 2u32.pow(KEY_SIZE);

// Structure to represent a node in the cluster, with its id, ip and hashmap
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Node {
    id: u32,
    ip: String,
    hashmap: HashMap<String, String>,
    resp_keys: Vec<u32>,
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
        Node { id, ip, hashmap: HashMap::new(), resp_keys: Vec::new() }
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
        node.resp_keys.push(key_id);
    }

    if previous_id > current_id {
        for key_id in 0..=current_id  {
            node.resp_keys.push(key_id);
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
async fn find_succesor(key: u32, finger_table: Vec<(u32, Node)>, current_id: u32 ) -> String{
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

    if  succesor.is_empty() {
        
        for finger in sorted_finger_table.iter() {
            if finger.1.id <= key {
                println!("Finger: {:?}", finger);
                succesor = finger.1.ip.clone();
            
            }
        }
    }

    println!("Succesor: {}", succesor.len());

    if succesor.is_empty() {

      
        

        succesor = sorted_finger_table[sorted_finger_table.len() - 1].1.ip.clone();
    }





    // println!("Finger table: {:?}", finger_table);

    // for finger in finger_table.iter() {
    //     if finger.1.id == key || finger.0 == key {
    //         succesor = finger.1.ip.clone();
    //         break;
    //     }
    // }

    
    // if succesor.is_empty() {

    //     for finger in finger_table.iter().rev(){
    //         println!("Finger: {:?}", finger);
    //         if finger.0 < key {
    //             succesor = finger.1.ip.clone();
                
                
    //         }
            
            
            
    //     }
    // }

    // if succesor.is_empty() {

    //     //sort the finger table by id
    //     let mut sorted_finger_table = finger_table.clone();
    //     sorted_finger_table.sort_by_key(|finger| finger.1.id);

    //     // set succesor as the first node in the sorted finger table
    //     succesor = sorted_finger_table[0].1.ip.clone();


        
    // }
    println!("Key: {}", key);
    println!("finger table: {:?}", finger_table);
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
    let ip_and_port = format!("{}:{}", local_ip.to_string(), 65000);

    // create a new node with the node id and ip address with port number
    let mut node = Node::new(node_id, ip_and_port);
    
    // populate the fingertable of the node
    let mut  finger_table = populate_fingertable(node_id, num_nodes);
    
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
async fn item_put(key: web::Path<String>, data:String  ,node_data: web::Data<Arc<Mutex<Node>>>, finger_table: web::Data<Mutex<Vec<(u32, Node)>>>) -> impl Responder {

    println!("Received PUT request for key: {:?}", key);
    let hash_ref = &key;
    let hashed_key = hash_function(hash_ref.to_string());


    let node_ref = node_data.get_ref();



    let mut node = node_ref.lock().unwrap();


    // println!("node id: {} is responsible for: {:?}", node.id, node.RespKeys);
   
    if node.resp_keys.contains(&hashed_key) {
        
        println!("Key {:?} is in the node {}", key, node.id);
        node.hashmap.insert(key.to_string(), data);
        HttpResponse::Ok().body(format!("Item {:?} updated", key))
        
    } 
    else {

     
        
        
        println!("node_id: {}", node.id);
        //Sends API call to succesor. (Under construction)
        let succesor = find_succesor(hashed_key, finger_table.lock().unwrap().clone(), node.id).await;
        

        let url = format!("http://{}/storage/{}", succesor, key);
        let client = reqwest::Client::new();
        let res = client.put(&url).body(data).send().await;

        // printlin!!()
        // if res.is_err() {
        //     println!("Error: {:?}", res);
        // }
        // else {
        //     println!("Success: {:?}", res);
            
        // }

        HttpResponse::Ok().body(format!("Item {:?} inserted", key ))
    }

}

#[get("/storage/{key}")]
async fn item_get(key: web::Path<String>, node_data: web::Data<Arc<Mutex<Node>>>, finger_table: web::Data<Mutex<Vec<(u32, Node)>>>) -> impl Responder {

let node_ref = node_data.get_ref();

let node = node_ref.lock().unwrap();
let hash_ref = &key;
let hashed_key = hash_function(hash_ref.to_string());



if node.resp_keys.contains(&hashed_key) {

    if let Some(value) = node.hashmap.get(&key.to_string()) {
        HttpResponse::Ok().body(value.to_string())
    } else
    {
        HttpResponse::NotFound().body("Key not found")
    }
    

} else {

    println!("node_id: {}", node.id);
    let succesor = find_succesor(hashed_key, finger_table.lock().unwrap().clone(), node.id).await;

    let url = format!("http://{}/storage/{}", succesor, key);
    let client = reqwest::Client::new();

    let res = client.get(&url).send().await;



    if let Ok(response) = res { 

        if response.status() == 404 {
            HttpResponse::NotFound().body("Key not found")
        }
        else {
            HttpResponse::Ok().body(response.text().await.unwrap())
        }
            
        


    } else {
       HttpResponse::InternalServerError().finish()
    }




}


}
