use actix_web::{get, put,post, web, App, HttpResponse, HttpServer, Responder};
use serde_derive::{Serialize, Deserialize};
use serde_json::Value;
use std::{env, iter::Successors};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync:: RwLock;
// use no_deadlocks::RwLock;
use hyper::{Body, Client, Request, Uri};
mod node;

const KEY_SIZE: u32 = 8;
const CLUSTER_SIZE: u32 = 2u32.pow(KEY_SIZE);
use node::{Node, NodePrev};

/// Structure to represent the neighbors of a node
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Neighbors {
    prev: String,
    next: String,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq, Deserialize)]
struct NodeInfo {
    node_hash: u32,
    successor: String,
    others: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct JoinQuery {
    nprime: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LeaveQuery {
    predecessor: String,
    leaving_node: String,
}


#[derive(Debug, Serialize, Deserialize)]
struct SuccQuery {
    new_succesor: String,
    leaving_node: String,
}



async fn send_put_request(url: String, data: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    println!("url {:?} and data {:?}", url, data);

    let uri = url.parse::<Uri>()?;

    let req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "text/plain")
        .body(Body::from(data))?;

    let _res = client.request(req).await?;

    println!("res {:?}", _res);

    Ok(())
}

async fn send_put_request_JSON_succ_leave(url: String, data: LeaveQuery) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    println!("url {:?} and data {:?}", url, data);

    let uri = url.parse::<Uri>()?;

    let json_data = serde_json::to_string(&data)?;

    let req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json_data))?;

    let _res = client.request(req).await?;

    println!("res {:?}", _res);

    Ok(())
}

async fn send_put_request_JSON_pred_leave(url: String, data: SuccQuery) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    println!("url {:?} and data {:?}", url, data);

    let uri = url.parse::<Uri>()?;

    let json_data = serde_json::to_string(&data)?;

    let req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json_data))?;

    let _res = client.request(req).await?;

    println!("res {:?}", _res);

    Ok(())
}

async fn send_get_request(url: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let uri = url.parse::<Uri>()?;

    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header("content-type", "text/plain")
        .body(Body::empty())?;

    let res = client.request(req).await?;

    

    let body = hyper::body::to_bytes(res.into_body()).await?;

    let body = String::from_utf8(body.to_vec())?;

        
    

    
    
    Ok(body)
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
/// first argument is the number of nodes in the cluster.
/// second argument is the port number of the node
///
/// ### Returns
///
/// This function returns a `std::io::Result<()>` which indicates whether the operation was successful or not.
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     if env::var_os("RUST_LOG").is_none() {
//         // Set `RUST_LOG=todos=debug` to see debug logs,
//         // this only shows access logs.
//         env::set_var("RUST_LOG", "todos=info");
//     }
//     pretty_env_logger::init();

//     let args: Vec<String> = env::args().collect();

//     if args.len() < 2 {
//         println!("Provde number of nodes as argument");
//         std::process::exit(1);
//     }


//     if args.len() < 3 {
//         println!("Provde port number as argument");
//         std::process::exit(1);
//     }
        
    

//     let num_nodes: u32 = args[1].parse().unwrap();

//     let port_num: u32 = args[2].parse().unwrap();

//     // get the local ip address of the node
//     let local_ip: IpAddr = get_local_ip().unwrap();

//     // get the node id of the node
//     let node_id = Node::hash_function(local_ip.to_string());

//     // format the ip address and port of the node
//     let ip_and_port = format!("{}:{}", local_ip.to_string(), port_num);

//     // create a new node with the node id and ip address with port number
//     let mut node = Node::new(node_id, ip_and_port);

//     // populate the fingertable of the node
//     let mut finger_table = Node::populate_fingertable(node_id, num_nodes);

//     // get the previous node of the current node in the cluster
//     let mut previous_node = Node::get_previous_node(node_id, num_nodes);

//     // format the ip address and port of the previous node
//     previous_node.ip = format!("{}:{}", previous_node.ip, port_num);

//     // format the ip address and port of the nodes in the fingertable
//     for finger in finger_table.iter_mut() {
//         finger.1.ip = format!("{}:{}", finger.1.ip, port_num);
//     }

//     if num_nodes == 1 {
//         node.resp_keys = (0..CLUSTER_SIZE).collect();
//         previous_node = node.clone();
//         finger_table = Vec::new();
//         finger_table.push((node_id, node.clone()));
//     }

//     Node::fill_hashmap(&mut node, previous_node.id, node_id);

//     let node_data = web::Data::new(Arc::new(RwLock::new(node)));
//     let previous_node_data = web::Data::new(RwLock::new(previous_node));
//     let finger_table_data = web::Data::new(Arc::new(RwLock::new(finger_table)));

//     println!("Server starting at http:// {}", local_ip.to_string());

//     let server_addr = format!("{}:{}", "0.0.0.0", port_num);

//     HttpServer::new(move || {
//         App::new()
//             .app_data(node_data.clone())
//             .app_data(previous_node_data.clone())
//             .app_data(finger_table_data.clone())
//             .service(index)
//             .service(item_get)
//             .service(item_put)
//             .service(get_node_info)
//     })
//     .workers(4)
//     .bind(server_addr)?
//     .run()
//     .await?;

//     Ok(())
// }


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    if env::var_os("RUST_LOG").is_none() {
                // Set `RUST_LOG=todos=debug` to see debug logs,
                // this only shows access logs.
                env::set_var("RUST_LOG", "todos=info");
            }
            pretty_env_logger::init();
        
            let args: Vec<String> = env::args().collect();
        
            if args.len() < 2 {
                println!("Provde number of nodes as argument");
                std::process::exit(1);
            }
        
        
            if args.len() < 3 {
                println!("Provde port number as argument");
                std::process::exit(1);
            }
                
           
        
            let _num_nodes: u32 = args[1].parse().unwrap();
        
            let port_num: u32 = args[2].parse().unwrap();
        
            // get the local ip address of the node
            let local_ip: IpAddr = get_local_ip().unwrap();
            
            // format the ip address and port of the node
            let ip_and_port = format!("{}:{}", local_ip.to_string(), port_num);
            
                // get the node id of the node
                let node_id = Node::hash_function(ip_and_port.clone());
        
            // create a new node with the node id and ip address with port number
            let mut node = Node::new(node_id, ip_and_port);

            let  current_num_nodes_in_cluster = 1;


            let  prev_node = NodePrev::new(0, "".to_string());
            let mut finger_table:Vec<(u32, Node)> = Vec::with_capacity(KEY_SIZE as usize);
            for i in 0..KEY_SIZE {
                let finger_id  = (node_id + 2u32.pow(i)) % CLUSTER_SIZE;


                finger_table.push((finger_id, node.clone()));
            }
            node.resp_keys = (0..CLUSTER_SIZE).collect();

        // println!("finger_table {:?} at node id {:?}", finger_table, node_id);

        let node_data = web::Data::new(Arc::new(RwLock::new(node)));
        let previous_node_data = web::Data::new(Arc::new(RwLock::new(prev_node)));
        let finger_table_data = web::Data::new(Arc::new(RwLock::new(finger_table)));
        let num_node_data = web::Data::new(Arc::new(RwLock::new(current_num_nodes_in_cluster)));


        println!("Server starting at http:// {} with node_id {:?}", local_ip.to_string(), node_id);
        let server_addr = format!("{}:{}", "0.0.0.0", port_num);

        HttpServer::new(move || {
            App::new()
                .app_data(node_data.clone())
                .app_data(finger_table_data.clone())
                .app_data(previous_node_data.clone())
                .app_data(num_node_data.clone())
                .service(index)
                .service(item_get)
                .service(item_put)
                .service(get_node_info)
                .service(post_join_ring)
                .service(put_notify)
                .service(get_succ)
                .service(put_notify_succ)
                .service(get_reps_keys)
                .service(put_notify_succ_leave)
                .service(put_notify_pred_leave)
                .service(post_leave)
        })
        .workers(8)
        .bind(server_addr)?
        .run()
        .await?;

        Ok(())

            


}


/// Handles GET requests to retrieve the IP addresses of the current node's neighbors in the Chord ring.
///
/// ### Arguments
///
/// * `finger_table` - A web::Data<RwLock<Vec<(u32, Node)>>> representing the finger table of the current node.
/// * `prev_node` - A web::Data<RwLock<Node>> representing the previous node in the Chord ring.
///
/// ### Returns
///
/// Returns a JSON response containing the IP addresses of the current node's neighbors.
#[get("/neighbors")]
async fn index(
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    prev_node: web::Data<Arc<RwLock<NodePrev>>>,
) -> impl Responder {
    let finger_table = finger_table.read().await;
    let prev_node = prev_node.read().await;
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
/// * `node_data` - A web::Data<RwLock<Node>> representing the current node's data.
/// * `finger_table` - A web::Data<RwLock<Vec<(u32, Node)>>> representing the current node's finger table.
///
/// ### Returns
///
/// An HttpResponse indicating whether the item was updated or inserted successfully.
#[put("/storage/{key}")]
async fn item_put(
    key: web::Path<String>,
    data: String,
    node_data: web::Data<Arc<RwLock<Node>>>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
) -> impl Responder {
    let hash_ref = &key;
    let hashed_key = Node::hash_function(hash_ref.to_string());

    let mut node_ref = node_data.write().await;

    // let mut node = &node_ref;

    if node_ref.resp_keys.contains(&hashed_key) {
        node_ref.hashmap.insert(key.to_string(), data);
        HttpResponse::Ok().body(format!("Item {:?} updated", key))
    } else {

        drop(node_ref);
        let node_ref = node_data.read().await;
        let node = &*node_ref;

        let succesor =
            Node::find_succesor_key(hashed_key, finger_table.read().await.clone(), node.id).await;

        let url = format!("http://{}/storage/{}", succesor, key);
       
        let _res = send_put_request(url, data).await;

        HttpResponse::Ok().body(format!("Item {:?} inserted", key))
    }
}

/// Handler for GET requests to retrieve an item from the node's storage.
///
/// ### Arguments
///
/// * `key` - The key of the item to retrieve.
/// * `node_data` - The node's data, containing a RwLock-protected hashmap of stored items.
/// * `finger_table` - The node's finger table, containing a RwLock-protected vector of finger table entries.
///
/// ### Returns
///
/// Returns an HTTP response with the retrieved item if it exists, or a 404 Not Found response if it does not.
#[get("/storage/{key}")]
async fn item_get(
    key: web::Path<String>,
    node_data: web::Data<Arc<RwLock<Node>>>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
) -> impl Responder {
    let node_ref = node_data.read().await;

    let node = &*node_ref;
    let hash_ref = &key;
    let hashed_key = Node::hash_function(hash_ref.to_string());

    if node.resp_keys.contains(&hashed_key) {
        if let Some(value) = node.hashmap.get(&key.to_string()) {
            HttpResponse::Ok().body(value.to_string())
        } else {
            HttpResponse::NotFound().body("Key not found")
        }
    } else {
        let succesor =
            Node::find_succesor_key(hashed_key, finger_table.read().await.clone(), node.id).await;

      
        let url = format!("http://{}/storage/{}", succesor, key);
        let _res = send_get_request(url).await;
        

 

       if let Ok(response) = _res {
           if response == "Key not found" {
               HttpResponse::NotFound().body("Key not found")
           } else {
               HttpResponse::Ok().body(response)
           }
           
       } else {
           HttpResponse::InternalServerError().finish()
       }
           
       

    }
}

#[get("/node-info")]
async fn get_node_info(node_data: web::Data<Arc<RwLock<Node>>>, finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>) -> impl Responder {
    let node_ref = node_data.read().await;
    let node = &*node_ref;
    let mut finger_table_ref = finger_table.read().await;
    let node_hash = node.id;
    let mut successor: String = "".to_string();
    let mut other = Vec::new();

    if finger_table_ref.len() != 0 {

        successor = finger_table_ref[0].1.ip.clone();
        for finger in finger_table_ref.iter() {
            other.push(finger.1.ip.clone());
        }
        
    }


    let node_info = NodeInfo {
        node_hash,
        successor,
        others: other,
    };

    HttpResponse::Ok().json(node_info)
}



 #[post("/join")]
async fn post_join_ring(query: web::Query<JoinQuery>, node_data: web::Data<Arc<RwLock<Node>>>, finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>, previous_node_data: web::Data<Arc<RwLock<NodePrev>>>, num_node_data: web::Data<Arc<RwLock<i32>>>,) -> impl Responder 
 {

    let ip_and_port:Vec<&str> = query.nprime.split(":").collect();
    println!("ip_and_port {:?}", ip_and_port); 
    
    
    let mut node_ref = node_data.write().await;
    println!("node_id {:?}", node_ref.id);
    

    let mut finger_table_ref = finger_table.write().await;
    
    
    let mut num_node_data_ref = num_node_data.write().await;
    

    let mut prev_node_ref = previous_node_data.write().await; //DEADLOCK????
    

    


    *num_node_data_ref += 1;

    node_ref.resp_keys.clear();
    

    let url = format!("http://{}/succesor/{}", query.nprime, node_ref.ip);

    let res = send_get_request(url).await;

    let successor = res.unwrap();
    let succesor_id = Node::hash_function(successor.clone());
    let suc_node = Node::new(succesor_id, successor.clone());

  

    finger_table_ref[0].1.ip = successor.clone();
        finger_table_ref[0].1.id = succesor_id;
    
    
        for (i, finger) in finger_table_ref.iter_mut().enumerate() {
            if i == 0 {
                continue;
            }
            let best_succ = Node::get_best_successor(finger.0, node_ref.id, finger.1.clone(), suc_node.clone());
            finger.1.ip = best_succ.ip;
            finger.1.id = best_succ.id;
        }
    
            // println!("node_ref {:?}", node_ref.clone());
            // println!("succesor {:?}", suc_node);
            // println!("finger_table_ref {:?}", finger_table_ref);

    let url = format!("http://{}/neighbors", successor);
    let res = send_get_request(url).await;



    let neighbors: Vec<String> = serde_json::from_str(&res.unwrap()).unwrap();


    
    println!("neighbors {:?}", neighbors);

    if neighbors[0] == "" {
        prev_node_ref.ip = neighbors[1].clone();
        prev_node_ref.id = Node::hash_function(neighbors[1].clone());
    
    }
    else {
        
            prev_node_ref.ip = neighbors[0].clone();
            prev_node_ref.id = Node::hash_function(neighbors[0].clone());
    }

    let node_ref_id = node_ref.id;

    Node::_fill_hashmap(&mut node_ref, prev_node_ref.id, node_ref_id);


    println!("joined node prev {:?} and succesor {:?}", prev_node_ref, suc_node);

    let url = format!("http://{}/notify_succ/{}", successor, node_ref.ip);
    let res = send_put_request(url, node_ref.ip.clone()).await;


    let url = format!("http://{}/notify_pred/{}", prev_node_ref.ip, node_ref.ip);
    let res = send_put_request(url, node_ref.ip.clone()).await;




    HttpResponse::Ok().body("ok")

 }


 #[post("/leave")]
 async fn post_leave(node_data: web::Data<Arc<RwLock<Node>>>, finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>, previous_node_data: web::Data<Arc<RwLock<NodePrev>>>) -> impl Responder
 {  

    let mut node_ref = node_data.write().await;
    let mut finger_table_ref = finger_table.write().await;
    let mut prev_node_ref = previous_node_data.write().await;

    let mut node_ref_id = node_ref.id;
    let mut prev_node_ref_id = prev_node_ref.id;
    
    node_ref.resp_keys.clear();
    node_ref.resp_keys = (0..CLUSTER_SIZE).collect();

    let url = format!("http://{}/notify_succ_leave", finger_table_ref[0].1.ip);

    let info: LeaveQuery = LeaveQuery {
        predecessor: prev_node_ref.ip.clone(),
        leaving_node: node_ref.ip.clone(),
    };

    let res = send_put_request_JSON_succ_leave(url, info).await;

    let info_succ = SuccQuery {
        new_succesor: finger_table_ref[0].1.ip.clone(),
        leaving_node: node_ref.ip.clone(),
    };

    let url = format!("http://{}/notify_pred_leave", prev_node_ref.ip);
    let res = send_put_request_JSON_pred_leave(url, info_succ).await;

    let mut new_finger_table = Vec::with_capacity(KEY_SIZE as usize);

    for i in 0..KEY_SIZE {
        let finger_id  = (node_ref_id + 2u32.pow(i)) % CLUSTER_SIZE;
        let finger_node = node_ref.clone();
        new_finger_table.push((finger_id, finger_node));
    }




    prev_node_ref.ip = "".to_string();
    prev_node_ref.id = 0;



    HttpResponse::Ok().body("ok")
 }

 #[put("/notify_succ_leave")]
 async fn put_notify_succ_leave(info: web::Json<LeaveQuery>,node_data: web::Data<Arc<RwLock<Node>>>, finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>, previous_node_data: web::Data<Arc<RwLock<NodePrev>>>) -> impl Responder
 {


    let mut node_ref = node_data.write().await;
    let mut finger_table_ref = finger_table.write().await;
    let mut prev_node_ref = previous_node_data.write().await;


    let mut node_ref_id = node_ref.id;
    let  leaving_node_id = Node::hash_function(info.leaving_node.clone());


    prev_node_ref.id= Node::hash_function(info.predecessor.clone());
    prev_node_ref.ip = info.predecessor.clone();

    node_ref.resp_keys.clear();
    Node::_fill_hashmap(&mut node_ref, prev_node_ref.id, node_ref_id);


    for finger in finger_table_ref.iter_mut() {

        if finger.1.id == leaving_node_id {
            finger.1.id = node_ref.id;
            finger.1.ip = node_ref.ip.clone();
        }

    }

    println!("info {:?}", info);

    println!("succ {:?}", finger_table_ref[0].1.ip);
    println!("prev {:?}", prev_node_ref.ip);

    HttpResponse::Ok().body("ok")
 }
 

 #[put("/notify_pred_leave")]
 async fn put_notify_pred_leave(info: web::Json<SuccQuery>,node_data: web::Data<Arc<RwLock<Node>>>, finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>, previous_node_data: web::Data<Arc<RwLock<NodePrev>>>) -> impl Responder
 {

    let mut node_ref = node_data.write().await;
    let mut finger_table_ref = finger_table.write().await;
    let mut prev_node_ref = previous_node_data.write().await;

    let mut node_ref_id = node_ref.id;
    let  leaving_node_id = Node::hash_function(info.leaving_node.clone());
    let new_succ = Node::new(Node::hash_function(info.new_succesor.clone()), info.new_succesor.clone());

    
    if new_succ.id == node_ref.id{
        HttpResponse::Ok().body("ok")
        
    }
    else {
    
        finger_table_ref[0].1.id = new_succ.id;
        finger_table_ref[0].1.ip = new_succ.ip.clone();
    
        for (i, finger) in finger_table_ref.iter_mut().enumerate() {
            if i == 0 {
                continue;
            }
            
            if finger.1.id == leaving_node_id {
                finger.1.id = new_succ.id;
                finger.1.ip = new_succ.ip.clone();
            }
        }
        let url = format!("http://{}/notify_pred_leave", prev_node_ref.ip);
        let res = send_put_request(url, serde_json::to_string(&info).unwrap()).await;
        HttpResponse::Ok().body("ok")
    }



 }

 #[get("/succesor/{node_ip}")]
 async fn get_succ(    node_ip: web::Path<String>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>, node_data: web::Data<Arc<RwLock<Node>>>) -> impl Responder
 {

        
    let mut node_ref = node_data.write().await;
    let node_ref_id = node_ref.id;
    let finger_table_ref = finger_table.read().await;

   

    let node_id = Node::hash_function(node_ip.to_string());

    println!("node_id {:?}, node_ip {:?} and keys_resp  {:?} ", node_ref_id, node_ref.ip, node_ref.resp_keys);

    let mut resp = 0;

    for keys in node_ref.resp_keys.iter() {
   
        if *keys == node_id {
          
            resp = 1;
        }
    }

    if resp == 1 {


        HttpResponse::Ok().body(node_ref.ip.clone())
    } else {
        let mut sorted_finger_table = finger_table_ref.clone();
        sorted_finger_table.sort_by_key(|finger| Node::hash_function(finger.1.ip.to_string()));

        let mut succesor = String::new();

        let node_id_int = node_id;

        // println!("sorted_finger_table {:?}", sorted_finger_table);

        for finger in sorted_finger_table.iter() {
            if finger.1.id >= node_id_int {
                succesor = finger.1.ip.to_string();
                break;
            
            }
        }   

        // println!("succesor {:?}", succesor);

        if succesor.is_empty() {
            let url = format!("http://{}/succesor/{}", sorted_finger_table[0].1.ip, node_id);
            let res = send_get_request(url).await;
            HttpResponse::Ok().body(res.unwrap())
        } else {
            HttpResponse::Ok().body(succesor)
        }
    }


 }


 #[put("/notify_succ/{node_ip}")]
async fn put_notify_succ(  node_ip: web::Path<String>,
    prev_node: web::Data<Arc<RwLock<NodePrev>>>,
    node_data: web::Data<Arc<RwLock<Node>>>,
 ) -> impl Responder
 {

    let mut prev_node_ref = prev_node.write().await;
    prev_node_ref.ip = node_ip.to_string();
    prev_node_ref.id = Node::hash_function(node_ip.to_string());

    let mut node_ref = node_data.write().await;

    node_ref.resp_keys.clear();

    let node_ref_id = node_ref.id;

    Node::_fill_hashmap(&mut node_ref, prev_node_ref.id, node_ref_id);

    // println!("node_keys in notify {:?}", node_ref.resp_keys);

    

    HttpResponse::Ok().body("ok")



 
 }



#[put("/notify_pred/{node_ip}")]
async fn put_notify(  node_ip: web::Path<String>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    prev_node: web::Data<Arc<RwLock<NodePrev>>>,
    node_data: web::Data<Arc<RwLock<Node>>>, ) -> impl Responder
 {

    let mut finger_table_ref = finger_table.write().await;
    let node_ref = node_data.read().await;

    println!("node_ip {:?}", node_ip);

   

    let succesor_id = Node::hash_function(node_ip.to_string());
    let suc_node = Node::new(succesor_id, node_ip.to_string());

    for finger in finger_table_ref.iter_mut() {
        let best_succ = Node::get_best_successor(finger.0, node_ref.id, finger.1.clone(), suc_node.clone());
        finger.1.ip = best_succ.ip;
        finger.1.id = best_succ.id;

    }

    // println!("finger_table_ref 2 {:?}", finger_table_ref);

    let mut prev_node_ref = prev_node.write().await;
    

    // println!("node_ref {:?}", node_ref.clone());
    // println!("prev_node_ref {:?}", prev_node_ref.clone());
    // println!("succesor_id {:?}", succesor_id);

    if prev_node_ref.ip == ""{
        prev_node_ref.ip = node_ip.to_string();
        prev_node_ref.id = succesor_id;
        HttpResponse::Ok().body("ok")
    }
    else if prev_node_ref.id == succesor_id  {
        HttpResponse::Ok().body("ok")
    }
    else {
        let url = format!("http://{}/notify_pred/{}", prev_node_ref.ip, node_ip);
        println!("url {:?}", url);
        let res = send_put_request(url, node_ip.to_string()).await;
        HttpResponse::Ok().body("ok")
    }
    



 
 }

#[get("/reps_keys")]
async fn get_reps_keys(node_data: web::Data<Arc<RwLock<Node>>>) -> impl Responder {
    let node_ref = node_data.read().await;
    let node = &*node_ref;
    HttpResponse::Ok().json(node.resp_keys.clone())
}