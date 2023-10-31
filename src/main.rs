use actix_web::{get, post, put, web, App, HttpResponse, HttpServer, Responder};
use actix_rt::spawn;
use actix_rt::time::interval;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use std::env;
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
// use no_deadlocks::RwLock;
use hyper::{Body, Client, Request, Uri};
mod node;

const KEY_SIZE: u32 = 20;
const CLUSTER_SIZE: u32 = 2u32.pow(KEY_SIZE);
use node::{Node, NodePrev};

/// Structure to represent the neighbors of a node
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Neighbors {
    prev: String,
    next: String,
}
/// Struct representing information about a node in the Chord protocol.
#[derive(Serialize, Debug, Clone, Eq, PartialEq, Deserialize)]
struct NodeInfo {
    node_hash: u32,
    successor: String,
    others: Vec<String>,
}

/// Struct representing a join query, containing the nprime field.
#[derive(Debug, Deserialize)]
struct JoinQuery {
    nprime: String,
}

/// Struct representing a leave query message.
#[derive(Debug, Serialize, Deserialize)]
struct LeaveQuery {
    predecessor: String,
    leaving_node: String,
}

/// Struct representing a successor query message.
#[derive(Debug, Serialize, Deserialize)]
struct SuccQuery {
    new_succesor: String,
    leaving_node: Vec<String>,
    succ_list_node: Vec<SuccListNode>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SuccListNode {
    id: u32,
    ip: String,
}


/// Sends a PUT request to the specified URL with the given data.
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL to send the request to.
/// * `data` - A string slice that holds the data to be sent in the request body.
///
/// # Returns
///
/// Returns a `Result` indicating whether the request was successful or not.
///
/// # Examples
///
/// ```
/// # use hyper::{Client, Uri, Request, Body};
/// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// # let url = "http://example.com".to_string();
/// # let data = "some data".to_string();
/// let res = send_put_request(url, data).await;
/// # Ok(())
/// # }
/// ```
async fn send_put_request(
    url: String,
    data: String,
) -> Result<() , Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    // println!("url {:?} and data {:?}", url, data);

    let uri = url.parse::<Uri>()?;

    let req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "text/plain")
        .body(Body::from(data))?;

    let _res = client.request(req).await?;

    Ok(())
}

async fn send_put_request_with_list(
    url: String,
    data: Vec<SuccListNode>,
) -> Result<Vec<SuccListNode> , Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    // println!("url {:?} and data {:?}", url, data);

    let uri = url.parse::<Uri>()?;

    let json_data = serde_json::to_string::<Vec<SuccListNode>>(&data)?;

    let req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json_data))?;

    let _res = client.request(req).await?;
    println!("res {:?}", _res);

    let body = hyper::body::to_bytes(_res.into_body()).await?;
    let body = String::from_utf8(body.to_vec())?;
    let nodes = serde_json::from_str::<Vec<SuccListNode>>(&body)?;
    // println!("body {:?}", nodes);

    Ok(nodes)
}

/// Sends a PUT request with JSON data to the specified URL for leaving the Chord network.
///
/// # Arguments
///
/// * `url` - A String representing the URL to send the request to.
/// * `data` - A LeaveQuery struct representing the data to be sent in the request body.
///
/// # Returns
///
/// Returns a Result indicating whether the request was successful or not.
///
/// # Examples
///
/// ```
/// use p2p_rust_chord::main::send_put_request_JSON_succ_leave;
///
/// #[tokio::main]
/// async fn main() {
///     let url = "http://localhost:8080/leave".to_string();
///     let data = LeaveQuery { id: 123 };
///     let result = send_put_request_JSON_succ_leave(url, data).await;
///     assert!(result.is_ok());
/// }
/// ```
async fn send_put_request_json_succ_leave(
    url: String,
    data: LeaveQuery,
) -> Result<Vec<SuccListNode>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    // println!("url {:?} and data {:?}", url, data);

    let uri = url.parse::<Uri>()?;

    let json_data = serde_json::to_string(&data)?;

    let req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json_data))?;

    let _res = client.request(req).await?;
    let body = hyper::body::to_bytes(_res.into_body()).await?;
    let body = String::from_utf8(body.to_vec())?;
    let nodes = serde_json::from_str::<Vec<SuccListNode>>(&body)?;

    // println!("res {:?}", _res);

    Ok(nodes)
}

/// Sends a PUT request with JSON data to the specified URL using the provided `SuccQuery` data.
///
/// # Arguments
///
/// * `url` - A `String` representing the URL to send the request to.
/// * `data` - A `SuccQuery` struct containing the data to send in the request body.
///
/// # Returns
///
/// Returns a `Result` with an empty `Ok` value if the request was successful, or a `Box` containing
/// a `dyn` error if an error occurred.
async fn send_put_request_json_pred_leave(
    url: String,
    data: SuccQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    // println!("url {:?} and data {:?}", url, data);

    let uri = url.parse::<Uri>()?;

    let json_data = serde_json::to_string(&data)?;

    let req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json_data))?;

    let _res = client.request(req).await?;

    // println!("res {:?}", _res);

 

    Ok(())
}

/// Sends a GET request to the specified URL and returns the response body as a String.
///
/// # Arguments
///
/// * `url` - A String that represents the URL to send the GET request to.
///
/// # Returns
///
/// * `Result<String, Box<dyn std::error::Error + Send + Sync>>` - A Result that contains the response body as a String if the request was successful, or an error if the request failed.
///
/// # Examples
///
/// ```
/// let response = send_get_request("https://example.com".to_string()).await;
/// match response {
///     Ok(body) => println!("Response body: {}", body),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
async fn send_get_request(url: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let uri = url.parse::<Uri>()?;

    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header("content-type", "text/plain")
        .body(Body::empty())?;

    let res = client.request(req).await?;

    
    if res.status() == 503 {

        Err("Node crashed".into())
        
    }
    else {
        
        let body = hyper::body::to_bytes(res.into_body()).await?;
    
        let body = String::from_utf8(body.to_vec())?;
        Ok(body)
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
    let crash_flag = AtomicBool::new(false);

    let _num_nodes: u32 = args[1].parse().unwrap();

    let port_num: u32 = args[2].parse().unwrap();

    // get the local ip address of the node
    let local_ip: IpAddr = get_local_ip().unwrap();

    // format the ip address and port of the node
    let ip_and_port = format!("{}:{}", local_ip.to_string(), port_num);

    let ip_and_port_clone = ip_and_port.clone();

    // get the node id of the node
    let node_id = Node::hash_function(ip_and_port.clone());

    // create a new node with the node id and ip address with port number
    let mut node = Node::new(node_id, ip_and_port);

    let current_num_nodes_in_cluster = 1;

    let prev_node = NodePrev::new(0, "".to_string());
    let mut finger_table: Vec<(u32, Node)> = Vec::with_capacity(KEY_SIZE as usize);
    for i in 0..KEY_SIZE {
        let finger_id = (node_id + 2u32.pow(i)) % CLUSTER_SIZE;

        finger_table.push((finger_id, node.clone()));
    }
    node.resp_keys = (0..CLUSTER_SIZE).collect();

    let succesor_list: Vec<SuccListNode> = Vec::new();
    // succesor_list.push(SuccListNode{id: node_id, ip: ip_and_port_clone.clone()});

    // println!("finger_table {:?} at node id {:?}", finger_table, node_id);

    let node_data = web::Data::new(Arc::new(RwLock::new(node)));
    let previous_node_data = web::Data::new(Arc::new(RwLock::new(prev_node)));
    let finger_table_data = web::Data::new(Arc::new(RwLock::new(finger_table)));
    let num_node_data = web::Data::new(Arc::new(RwLock::new(current_num_nodes_in_cluster)));
    let crash_flag = web::Data::new(Arc::new(RwLock::new(crash_flag)));
    let succesor_list_data = web::Data::new(Arc::new(RwLock::new(succesor_list)));
    let previous_node_data_clone = previous_node_data.clone();
    let crash_flag_clone = crash_flag.clone();

    let finger_table_data_clone = finger_table_data.clone();
    let succesor_list_data_clone = succesor_list_data.clone();
    println!(
        "Server starting at http:// {} with node_id {:?}",
        local_ip.to_string(),
        node_id
    );
    let server_addr = format!("{}:{}", "0.0.0.0", port_num);

    spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;

            let crash_flag_ref = crash_flag_clone.read().await;

            if crash_flag_ref.load(Ordering::Relaxed) {
                // println!("Node is simulating a crash");
                continue;
                
            }
            
            // let prev_node_ip = previous_node_data.read().await.ip.clone();
            let mut finger_table_ref = finger_table_data.write().await;
            let mut succ_list_ref = succesor_list_data_clone.write().await;
            let succ_node_ip = finger_table_ref[0].1.ip.clone();
            

            if succ_node_ip != ip_and_port_clone {
                let succ_node_url = format!("http://{}/check_alive", succ_node_ip);
                if let Err(_e) = send_get_request(succ_node_url).await {
                    let mut leaving_list:Vec<String> = Vec::new();
                    leaving_list.push(succ_node_ip.clone());
                    for succesor in succ_list_ref.clone() {
                        let succ_node_url = format!("http://{}/check_alive", succesor.ip);
                        if let Err(_e) = send_get_request(succ_node_url).await {
                            leaving_list.push(succesor.ip.clone());
                            
                        } else {
                            // println!("crashed succesors are {:?}", leaving_list);
                            // println!("Debug: Successor is alive and is {:?}", succesor.ip);
                            // let mut succ_list_ref = succesor_list_data_clone.write().await;
                            let prev_node_ref = previous_node_data.read().await;
                            // let mut finger_table_ref = finger_table_data.write().await;
                            let url = format!("http://{}/notify_succ/{}", succesor.ip, ip_and_port_clone);
                            let res = send_put_request_with_list(url, succ_list_ref.to_vec()).await;
                            let new_succ_node = res.unwrap();
                            println!("node_id {:?}", node_id);
                            println!("new succesor list {:?}", new_succ_node);
                            succ_list_ref.clear();
                            succ_list_ref.clone_from(&new_succ_node);


                        
                            if succ_list_ref.len() == KEY_SIZE as usize {
                                succ_list_ref.pop();
                                succ_list_ref.insert(0, SuccListNode{id: succesor.id, ip: succesor.ip.clone()});
                            }
                            else {
                                succ_list_ref.insert(0, SuccListNode{id: succesor.id, ip: succesor.ip.clone()});
                            }

                            println!("succ list {:?}", succ_list_ref);

                            let info_succ = SuccQuery {
                                new_succesor: succesor.ip.clone(),
                                leaving_node: leaving_list.clone(),
                                succ_list_node: succ_list_ref.to_vec(),
                            };

                            for (_i, finger) in finger_table_ref.iter_mut().enumerate() {

                                for leave in leaving_list.clone() {
                                    if finger.1.ip == leave {
                                        finger.1.id = succesor.id;
                                        finger.1.ip = succesor.ip.clone();
                                    }
                                }
                                
                                
                            }
                        
                            let url = format!("http://{}/notify_pred_leave", prev_node_ref.ip);
                            let _res = send_put_request_json_pred_leave(url, info_succ).await;
                            println!("Stablized");
                            break;
                        }
                    }
                } else {
                    // println!("Successor is alive");
                }
            }

            }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(node_data.clone())
            .app_data(finger_table_data_clone.clone())
            .app_data(previous_node_data_clone.clone())
            .app_data(num_node_data.clone())
            .app_data(crash_flag.clone())
            .app_data(succesor_list_data.clone())
            .service(index)
            .service(item_get)
            .service(item_put)
            .service(get_node_info)
            .service(post_join_ring)
            .service(put_notify)
            .service(get_zucc)
            .service(put_notify_succ)
            .service(get_reps_keys)
            .service(put_notify_succ_leave)
            .service(put_notify_pred_leave)
            .service(post_leave)
            .service(post_sim_crash)
            .service(post_sim_recover)
            .service(get_check_pred)
            .service(get_succ_list_node)
    })
    .workers(8)
    .bind(server_addr)?
    .run()
    .await?;


 


    Ok(())
}


#[get("/check_alive")]
async fn get_check_pred(crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }
    else {
        
        HttpResponse::Ok().body("check pred")
    }
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
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }

    let finger_table = finger_table.read().await;
    let prev_node = prev_node.read().await;
    let neightbors = Neighbors {
        prev: prev_node.ip.to_string(),
        next: finger_table[0].1.ip.clone().to_string(),
    };
    let ip_addr = vec![neightbors.prev, neightbors.next];
    HttpResponse::Ok().json(ip_addr)
}

/// Handler to store or update the value associated with a given key in the Chord ring's distributed storage.
///
/// This handler listens for PUT requests at the path `/storage/{key}`. When called, it determines
/// if the current node is responsible for the key. If so, it stores or updates the value for the key
/// in its local storage. If not, it forwards the request to the appropriate node using
/// the finger table's information.
///
/// # Arguments
///
/// * `key` - The key for which the value needs to be stored or updated, provided as a path parameter in the request URL.
/// * `data` - The value to be associated with the key, provided as the request body.
/// * `node_data` - Shared data about the current node, including its IP address, ID, the set of keys
///   it's responsible for (`resp_keys`), and its local storage (`hashmap`).
/// * `finger_table` - A shared reference to the finger table, a crucial data structure in the Chord protocol
///   that maintains references to other nodes in the system.
/// * `crash_flag` - A shared atomic boolean flag to check if the node is simulating a crash.
///
/// # Returns
///
/// If the `crash_flag` is set (indicating a simulated crash), the function returns a "Service Unavailable"
/// HTTP response.
///
/// If the node is responsible for the key, the value is stored or updated in its local storage, and an "OK"
/// HTTP response is returned indicating the status of the operation (either "updated" or "inserted").
///
/// If the node is not responsible for the key, the request is forwarded to the appropriate node, and
/// an "OK" HTTP response is returned indicating that the item was inserted.
#[put("/storage/{key}")]
async fn item_put(
    key: web::Path<String>,
    data: String,
    node_data: web::Data<Arc<RwLock<Node>>>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }

    let hashed_key = Node::hash_function(key.to_string());

    let mut node_ref = node_data.write().await;

    if node_ref.resp_keys.contains(&hashed_key) {
        node_ref.hashmap.insert(key.to_string(), data);
        HttpResponse::Ok().body(format!("Item {:?} updated", key))
    } else {
        drop(node_ref);
        let node_ref = node_data.read().await;
        let node = &*node_ref;

        match Node::find_successor_key(hashed_key, finger_table.read().await.clone(), node.id).await
        {
            Some(successor) => {
                let url = format!("http://{}/storage/{}", successor, key);
                match send_put_request(url, data).await {
                    Ok(_) => HttpResponse::Ok().body(format!("Item {:?} inserted", key)),
                    Err(_) => {
                        HttpResponse::InternalServerError().body("Failed to put data to successor")
                    }
                }
            }
            None => HttpResponse::ServiceUnavailable().body("Successor not found"),
        }
    }
}

/// Handler to retrieve the value associated with a given key in the Chord ring's distributed storage.
///
/// This handler listens for GET requests at the path `/storage/{key}`. When called, it determines
/// if the current node is responsible for the key. If so, it retrieves the value for the key
/// from its local storage. If not, it forwards the request to the appropriate node using
/// the finger table's information.
///
/// # Arguments
///
/// * `key` - The key for which the value needs to be retrieved, provided as a path parameter in the request URL.
/// * `node_data` - Shared data about the current node, including its IP address, ID, the set of keys
///   it's responsible for (`resp_keys`), and its local storage (`hashmap`).
/// * `finger_table` - A shared reference to the finger table, a crucial data structure in the Chord protocol
///   that maintains references to other nodes in the system.
/// * `crash_flag` - A shared atomic boolean flag to check if the node is simulating a crash.
///
/// # Returns
///
/// If the `crash_flag` is set (indicating a simulated crash), the function returns a "Service Unavailable"
/// HTTP response.
///
/// If the node is responsible for the key and the key exists in its local storage, the value associated
/// with the key is returned in an "OK" HTTP response.
///
/// If the node is not responsible for the key, the request is forwarded to the appropriate node.
/// If the key is not found in the system, a "Not Found" HTTP response is returned.
/// In case of any unexpected error, an "Internal Server Error" HTTP response is returned.
#[get("/storage/{key}")]
async fn item_get(
    key: web::Path<String>,
    node_data: web::Data<Arc<RwLock<Node>>>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }

    let node_ref = node_data.read().await;

    let node = &*node_ref;
    let hashed_key = Node::hash_function(key.to_string());

    println!("hashed key {:?}", hashed_key);
    // println!("resp keys {:?}", node.resp_keys);

    if node.resp_keys.contains(&hashed_key) {
        if let Some(value) = node.hashmap.get(&key.to_string()) {
            HttpResponse::Ok().body(value.to_string())
        } else {
            HttpResponse::NotFound().body("Key not found")
        }
    } else {
        match Node::find_successor_key(hashed_key, finger_table.read().await.clone(), node.id).await
        {
            Some(successor) => {
                let url = format!("http://{}/storage/{}", successor, key);
                match send_get_request(url).await {
                    Ok(response) => {
                        if response == "Key not found" {
                            HttpResponse::NotFound().body("Key not found")
                        } else {
                            HttpResponse::Ok().body(response)
                        }
                    }
                    Err(_) => HttpResponse::InternalServerError().finish(),
                }
            }
            None => HttpResponse::ServiceUnavailable().body("Successor not found"),
        }

        // let succesor =
        // Node::find_successor_key(hashed_key, finger_table.read().await.clone(), node.id).await;

        // let url = format!("http://{}/storage/{}", succesor.unwrap(), key);
        // println!("url {:?}", url);
        // let _res = send_get_request(url).await;
        // println!("res {:?}", _res);



        // if let Ok(response) = _res {
        // if response == "Key not found" {
        // HttpResponse::NotFound().body("Key not found")
        // } else {
        // HttpResponse::Ok().body(response)
        // }

        // } else {
        // HttpResponse::InternalServerError().finish()
        // }
    }
}

/// Handler to retrieve information about the current node in the Chord ring.
///
/// This handler listens for GET requests at the path `/node-info`. When called, it provides
/// key information about the current node, including its hash ID, its immediate successor in the Chord ring,
/// and references to other nodes maintained in its finger table.
///
/// # Arguments
///
/// * `node_data` - Shared data about the current node, including its IP address, ID, and the set of keys
///   it's responsible for (`resp_keys`).
/// * `finger_table` - A shared reference to the finger table, a crucial data structure in the Chord protocol
///   that maintains references to other nodes in the system.
/// * `crash_flag` - A shared atomic boolean flag to check if the node is simulating a crash.
///
/// # Returns
///
/// If the `crash_flag` is set (indicating a simulated crash), the function returns a "Service Unavailable"
/// HTTP response.
///
/// If the node is operational, it returns an "OK" HTTP response with a JSON object representing the
/// node's hash ID, its immediate successor, and other nodes from its finger table.
#[get("/node-info")]
async fn get_node_info(
    node_data: web::Data<Arc<RwLock<Node>>>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }
    let node_ref = node_data.read().await;
    let node = &*node_ref;
    let finger_table_ref = finger_table.read().await;
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

/// Handler for a node to join the Chord ring.
///
/// This handler listens for POST requests at the path `/join`. When called, it allows a node
/// to join an existing Chord ring by finding its appropriate successor and predecessor nodes.
/// The joining node updates its finger table, retrieves its set of responsible keys, and notifies
/// its successor and predecessor about its presence.
///
/// # Arguments
///
/// * `query` - Contains query parameters from the HTTP request, specifically the IP address and port
///   (`nprime`) of an existing node in the Chord ring that will be used as a reference point for the joining process.
/// * `node_data` - Shared data about the current node, including its IP address, ID, and the set of keys
///   it's responsible for (`resp_keys`).
/// * `finger_table` - A shared reference to the finger table, a crucial data structure in the Chord protocol
///   that maintains references to other nodes in the system.
/// * `previous_node_data` - Shared data about the current node's predecessor, including its IP address and ID.
/// * `num_node_data` - A shared counter that keeps track of the number of nodes currently in the Chord ring.
/// * `crash_flag` - A shared atomic boolean flag to check if the node is simulating a crash.
///
/// # Returns
///
/// If the `crash_flag` is set (indicating a simulated crash), the function returns a "Service Unavailable"
/// HTTP response.
///
/// If the joining process is successful, it returns an "OK" HTTP response with a message indicating
/// that the node has joined the Chord ring.
#[post("/join")]
async fn post_join_ring(
    query: web::Query<JoinQuery>,
    node_data: web::Data<Arc<RwLock<Node>>>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    previous_node_data: web::Data<Arc<RwLock<NodePrev>>>,
    num_node_data: web::Data<Arc<RwLock<i32>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
    succ_list: web::Data<Arc<RwLock<Vec<SuccListNode>>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }
    let _ip_and_port: Vec<&str> = query.nprime.split(":").collect();
    // println!("ip_and_port {:?}", ip_and_port);

    let mut node_ref = node_data.write().await;
    // println!("node_id {:?}", node_ref.id);

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

    // println!("succesor {:?}", successor);

    finger_table_ref[0].1.ip = successor.clone();
    finger_table_ref[0].1.id = succesor_id;

    for (i, finger) in finger_table_ref.iter_mut().enumerate() {
        if i == 0 {
            continue;
        }
        let best_succ =
            Node::get_best_successor(finger.0, node_ref.id, finger.1.clone(), suc_node.clone())
                .await;
        finger.1.ip = best_succ.ip;
        finger.1.id = best_succ.id;
    }

    // println!("node_ref {:?}", node_ref.clone());
    // // println!("succesor {:?}", suc_node);
    // println!("finger_table_ref {:?}", finger_table_ref);

    let url = format!("http://{}/neighbors", successor);
    let res = send_get_request(url).await;

    let neighbors: Vec<String> = serde_json::from_str(&res.unwrap()).unwrap();

    // println!("neighbors {:?}", neighbors);

    if neighbors[0] == "" {
        prev_node_ref.ip = neighbors[1].clone();
        prev_node_ref.id = Node::hash_function(neighbors[1].clone());
    } else {
        prev_node_ref.ip = neighbors[0].clone();
        prev_node_ref.id = Node::hash_function(neighbors[0].clone());
    }

    let node_ref_id = node_ref.id;

    Node::fill_hashmap(&mut node_ref, prev_node_ref.id, node_ref_id).await;

    // println!("joined node prev {:?} and succesor {:?}", prev_node_ref, suc_node);
    let mut  succ_list_ref = succ_list.write().await;

    let url = format!("http://{}/notify_succ/{}", successor, node_ref.ip);
    let res = send_put_request_with_list(url, succ_list_ref.to_vec()).await;


    succ_list_ref.clear();
    succ_list_ref.clone_from(&res.unwrap());

    if succ_list_ref.len() == KEY_SIZE as usize {
        succ_list_ref.pop();
        succ_list_ref.insert(0, SuccListNode{id: succesor_id, ip: successor.clone()});
    }
    else {
        succ_list_ref.insert(0, SuccListNode{id: succesor_id, ip: successor.clone()});
    }

    let url = format!("http://{}/notify_pred/{}", prev_node_ref.ip, node_ref.ip);
    let _res = send_put_request_with_list(url, succ_list_ref.to_vec()).await;

    HttpResponse::Ok().body("node joined")
}

/// Handles a POST request to leave the Chord network.
///
/// # Arguments
///
/// * `node_data` - A web::Data instance containing the current node's data.
/// * `finger_table` - A web::Data instance containing the current node's finger table.
/// * `previous_node_data` - A web::Data instance containing the previous node's data.
/// * `crash_flag` - A web::Data instance containing a flag indicating whether the node is simulating a crash.
///
/// # Returns
///
/// An HTTP response indicating whether the node successfully left the network.
#[post("/leave")]
async fn post_leave(
    node_data: web::Data<Arc<RwLock<Node>>>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    previous_node_data: web::Data<Arc<RwLock<NodePrev>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }
    let mut node_ref = node_data.write().await;
    let mut finger_table_ref = finger_table.write().await;
    let mut prev_node_ref = previous_node_data.write().await;

    let _node_ref_id = node_ref.id;
    let _prev_node_ref_id = prev_node_ref.id;

    node_ref.resp_keys.clear();
    node_ref.resp_keys = (0..CLUSTER_SIZE).collect();

    let url = format!("http://{}/notify_succ_leave", finger_table_ref[0].1.ip);

    let mut leaving_node_list = Vec::new();
    leaving_node_list.push(node_ref.ip.clone());

    let info: LeaveQuery = LeaveQuery {
        predecessor: prev_node_ref.ip.clone(),
        leaving_node: node_ref.ip.clone(),
    };

    let _res = send_put_request_json_succ_leave(url, info).await;

    let info_succ = SuccQuery {
        new_succesor: finger_table_ref[0].1.ip.clone(),
        leaving_node: leaving_node_list,
        succ_list_node: _res.unwrap(),
    };

    let url = format!("http://{}/notify_pred_leave", prev_node_ref.ip);
    let _res = send_put_request_json_pred_leave(url, info_succ).await;

    for finger in finger_table_ref.iter_mut() {
        finger.1.id = node_ref.id;
        finger.1.ip = node_ref.ip.clone();
    }

    prev_node_ref.ip = "".to_string();
    prev_node_ref.id = 0;

    HttpResponse::Ok().body("node left")
}

/// Handles a PUT request to notify that a successor node is leaving the network.
///
/// # Arguments
///
/// * `info` - JSON payload containing information about the leaving node and its predecessor.
/// * `node_data` - Arc-wrapped RwLock containing the current node's data.
/// * `finger_table` - Arc-wrapped RwLock containing the current node's finger table.
/// * `previous_node_data` - Arc-wrapped RwLock containing the previous node's data.
/// * `crash_flag` - Arc-wrapped RwLock containing a boolean flag indicating whether the node is simulating a crash.
///
/// # Returns
///
/// Returns an HTTP response indicating success or failure.
#[put("/notify_succ_leave")]
async fn put_notify_succ_leave(
    info: web::Json<LeaveQuery>,
    node_data: web::Data<Arc<RwLock<Node>>>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    previous_node_data: web::Data<Arc<RwLock<NodePrev>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
    succ_list: web::Data<Arc<RwLock<Vec<SuccListNode>>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }
    let mut node_ref = node_data.write().await;
    // let finger_table_ref = finger_table.write().await;
    let mut prev_node_ref = previous_node_data.write().await;
    let  succ_list_ref = succ_list.write().await;

    let node_ref_id = node_ref.id;
    // let leaving_node_id = Node::hash_function(info.leaving_node.clone());

    prev_node_ref.id = Node::hash_function(info.predecessor.clone());
    prev_node_ref.ip = info.predecessor.clone();

    node_ref.resp_keys.clear();
    Node::fill_hashmap(&mut node_ref, prev_node_ref.id, node_ref_id).await;

    // for finger in finger_table_ref.iter_mut() {
    //     if finger.1.id == leaving_node_id {
    //         finger.1.id = node_ref.id;
    //         finger.1.ip = node_ref.ip.clone();
    //     }
    // }



    HttpResponse::Ok().json(succ_list_ref.to_vec())
}

/// Handles PUT requests to notify the predecessor node of a leaving node in the Chord protocol.
///
/// # Arguments
///
/// * `info` - JSON payload containing information about the leaving node and its successor.
/// * `node_data` - Shared data of the current node.
/// * `finger_table` - Shared data of the finger table of the current node.
/// * `previous_node_data` - Shared data of the previous node of the current node.
/// * `crash_flag` - Shared data of the crash flag of the current node.
///
/// # Returns
///
/// Returns an HTTP response indicating the success or failure of the operation.
#[put("/notify_pred_leave")]
async fn put_notify_pred_leave(
    mut info: web::Json<SuccQuery>,
    node_data: web::Data<Arc<RwLock<Node>>>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    previous_node_data: web::Data<Arc<RwLock<NodePrev>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
    SuccListNode: web::Data<Arc<RwLock<Vec<SuccListNode>>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }
    let node_ref = node_data.write().await;
    let mut finger_table_ref = finger_table.write().await;
    let prev_node_ref = previous_node_data.write().await;

    let _node_ref_id = node_ref.id;

    // let mut leaving_node_ids = Vec::new();
    

    // for nodes in info.leaving_node.clone() {
    //    leaving_node_ids.push(Node::hash_function(nodes.clone()));
    // }

    let new_succ = Node::new(
        Node::hash_function(info.new_succesor.clone()),
        info.new_succesor.clone(),
    );

    let mut succ_list_ref = SuccListNode.write().await;
    let  new_list = info.succ_list_node.clone();

    succ_list_ref.clear();
    succ_list_ref.clone_from(&new_list);
    if succ_list_ref.len() == KEY_SIZE as usize {
        succ_list_ref.pop();
        succ_list_ref.insert(0, SuccListNode{id: new_succ.id, ip: new_succ.ip.clone()});
    }
    else {
        succ_list_ref.insert(0, SuccListNode{id: new_succ.id, ip: new_succ.ip.clone()});
    }

    info.succ_list_node.clear();
    info.succ_list_node.clone_from(&succ_list_ref);
    
    for (_i, finger) in finger_table_ref.iter_mut().enumerate() {

        for leave in info.leaving_node.clone() {
            if finger.1.ip == leave {
                finger.1.id = new_succ.id;
                finger.1.ip = new_succ.ip.clone();
            }
        }
        
        
    }


    if new_succ.id == node_ref.id {
        HttpResponse::Ok().body("ok")
    } else {
       
        let url = format!("http://{}/notify_pred_leave", prev_node_ref.ip);
        let _res = send_put_request_json_pred_leave(url, info.into_inner()).await;
        HttpResponse::Ok().body("ok")
    }
}

/// Retrieves the successor of a given node based on its IP address.
///
/// This handler listens for GET requests at the path `/succesor/{node_ip}`. It uses the Chord protocol's
/// algorithm to determine the successor of the provided node IP.
///
/// # Arguments
///
/// * `node_ip` - The IP address of the node for which the successor needs to be found.
///   This is extracted from the path parameter `{node_ip}`.
/// * `finger_table` - A shared reference to the finger table, which is a data st_res.into_body()tem.
/// * `node_data` - Shared data about the current node, including its IP address and ID.
/// * `crash_flag` - A shared atomic boolean flag to check if the node is simulating a crash.
///
/// # Returns
///
/// If the `crash_flag` is set (indicating a simulated crash), the function returns a "Service Unavailable"
/// HTTP response.
///
/// If the provided `node_ip` belongs to the current node or if its ID falls under the set of keys the current
/// node is responsible for (`resp_keys`), the handler returns the IP address of the current node.
///
/// Otherwise, it recursively finds the successor by making GET requests to other nodes' `/succesor/{node_ip}`
/// endpoints and returns the resultant IP address.
#[get("/succesor/{node_ip}")]
async fn get_zucc(
    node_ip: web::Path<String>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    node_data: web::Data<Arc<RwLock<Node>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }
    // Get the lock on the node data
    let node_ref = node_data.write().await;

    // Get the node id
    let node_ref_id = node_ref.id;

    // Get the lock on the finger table
    let _finger_table_ref = finger_table.read().await;

    // Get the node id of the node_ip
    let node_id = Node::hash_function(node_ip.to_string());

    // Check if the node_ip is the current node
    if node_ref.resp_keys.contains(&node_id) {
        // node is the current node
        HttpResponse::Ok().body(node_ref.ip.clone())
 
    } else {
        // node is not the current node and we need to find the successor
        match Node::find_successor_key(node_id, finger_table.read().await.clone(), node_ref_id)
            .await
        {
            Some(successor) => {
                let url = format!("http://{}/succesor/{}", successor, node_ip.to_string());
                match send_get_request(url).await {
                    Ok(res) => HttpResponse::Ok().body(res),
                    Err(_) => HttpResponse::ServiceUnavailable()
                        .body("Failed to get response from successor"),
                }
            }
            None => HttpResponse::ServiceUnavailable().body("Successor not found"),
        }
    }
}

/// Notifies the current node about its new successor.
///
/// This handler listens for PUT requests at the path `/notify_succ/{node_ip}`. It updates the current node's
/// predecessor information based on the provided IP address and recalculates the set of keys the current node
/// is responsible for (`resp_keys`).
///
/// # Arguments
///
/// * `node_ip` - The IP address of the node that is notifying the current node about being its successor.
///   This IP is extracted from the path parameter `{node_ip}`.
/// * `prev_node` - Shared data about the current node's predecessor, including its IP address and ID.
/// * `node_data` - Shared data about the current node, including its IP address, ID, and the set of keys
///   it's responsible for (`resp_keys`).
/// * `crash_flag` - A shared atomic boolean flag to check if the node is simulating a crash.
///
/// # Returns
///
/// If the `crash_flag` is set (indicating a simulated crash), the function returns a "Service Unavailable"
/// HTTP response.
///
/// After updating the predecessor data and recalculating the `resp_keys`, the handler returns an "OK" HTTP response.
#[put("/notify_succ/{node_ip}")]
async fn put_notify_succ(
    node_ip: web::Path<String>,
    prev_node: web::Data<Arc<RwLock<NodePrev>>>,
    node_data: web::Data<Arc<RwLock<Node>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
    succ_list: web::Data<Arc<RwLock<Vec<SuccListNode>>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }
    let mut prev_node_ref = prev_node.write().await;
    prev_node_ref.ip = node_ip.to_string();
    prev_node_ref.id = Node::hash_function(node_ip.to_string());

    let mut node_ref = node_data.write().await;

    node_ref.resp_keys.clear();

    let node_ref_id = node_ref.id;

    Node::fill_hashmap(&mut node_ref, prev_node_ref.id, node_ref_id).await;

    // println!("node_keys in notify {:?}", node_ref.resp_keys);

    // println!("prev_node_ref {:?}", prev_node_ref.clone());

    let succ_list_ref = succ_list.read().await;

    // println!("succ_list_ref {:?}", json!(succ_list_ref.clone()));
    
    HttpResponse::Ok().json(succ_list_ref.clone())
}

/// Notifies the current node about a potential better predecessor.
///
/// This handler listens for PUT requests at the path `/notify_pred/{node_ip}`. Upon receiving a notification,
/// the current node evaluates whether the notifying node is a better predecessor based on the Chord protocol's
/// criteria. The handler also updates the node's finger table entries.
///
/// # Arguments
///
/// * `node_ip` - The IP address of the node that is notifying the current node about being a potential better
///   predecessor. This IP is extracted from the path parameter `{node_ip}`.
/// * `finger_table` - A shared reference to the finger table, which is a data structure used in the Chord protocol
///   to maintain references to other nodes in the system.
/// * `prev_node` - Shared data about the current node's predecessor, including its IP address and ID.
/// * `node_data` - Shared data about the current node, including its IP address and ID.
/// * `crash_flag` - A shared atomic boolean flag to check if the node is simulating a crash.
///
/// # Returns
///
/// If the `crash_flag` is set (indicating a simulated crash), the function returns a "Service Unavailable"
/// HTTP response.
///
/// After evaluating the notifying node as a potential predecessor and updating the finger table, the handler
/// returns an "OK" HTTP response. If the notifying node isn't the direct predecessor but might be a better
/// predecessor for some other node in the network, this function can recursively notify other nodes about
/// the potential better predecessor.
#[put("/notify_pred/{node_ip}")]
async fn put_notify(
    node_ip: web::Path<String>,
    finger_table: web::Data<Arc<RwLock<Vec<(u32, Node)>>>>,
    prev_node: web::Data<Arc<RwLock<NodePrev>>>,
    node_data: web::Data<Arc<RwLock<Node>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
    succ_list: web::Data<Arc<RwLock<Vec<SuccListNode>>>>,
    json: web::Json<Vec<SuccListNode>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }
    let mut finger_table_ref = finger_table.write().await;
    let node_ref = node_data.read().await;

    // println!("node_ip {:?}", node_ip);

    let succesor_id = Node::hash_function(node_ip.to_string());
    let suc_node = Node::new(succesor_id, node_ip.to_string());

    for finger in finger_table_ref.iter_mut() {
        let best_succ =
            Node::get_best_successor(finger.0, node_ref.id, finger.1.clone(), suc_node.clone())
                .await;
        finger.1.ip = best_succ.ip;
        finger.1.id = best_succ.id;
    }


    
    let prev_node_ref = prev_node.write().await;
    
    let mut succ_list_ref = succ_list.write().await;
    
   
    
    succ_list_ref.clear();
    succ_list_ref.clone_from(&json.0);
    
    if succ_list_ref.len() == KEY_SIZE as usize{
        succ_list_ref.pop();
        succ_list_ref.insert(0, SuccListNode{id: finger_table_ref[0].1.id, ip: finger_table_ref[0].1.ip.clone()});
    }
    else {
        succ_list_ref.insert(0, SuccListNode{id: finger_table_ref[0].1.id, ip: finger_table_ref[0].1.ip.clone()});
    }
    
    // println!("node_ref {:?}", node_ref.clone());
    // println!("finger_table_ref {:?}", finger_table_ref);
    // println!("succ_list_ref {:?}", succ_list_ref);

    if prev_node_ref.id == succesor_id {
        HttpResponse::Ok().body("ok")
    } else {
        let url = format!("http://{}/notify_pred/{}", prev_node_ref.ip, node_ip);
        // println!("url {:?}", url);
        let _res = send_put_request_with_list(url, succ_list_ref.to_vec()).await;
        HttpResponse::Ok().body("ok")
    }
}

/// Retrieves the set of keys the current node is responsible for.
///
/// This handler listens for GET requests at the path `/reps_keys`. The function returns the set of keys (`resp_keys`)
/// that the current node is responsible for, as per the Chord protocol's criteria.
///
/// # Arguments
///
/// * `node_data` - Shared data about the current node, including its IP address, ID, and the set of keys
///   it's responsible for (`resp_keys`).
/// * `crash_flag` - A shared atomic boolean flag to check if the node is simulating a crash.
///
/// # Returns
///
/// If the `crash_flag` is set (indicating a simulated crash), the function returns a "Service Unavailable"
/// HTTP response.
///
/// Otherwise, it returns a JSON response containing the set of keys the current node is responsible for.
#[get("/reps_keys")]
async fn get_reps_keys(
    node_data: web::Data<Arc<RwLock<Node>>>,
    crash_flag: web::Data<Arc<RwLock<AtomicBool>>>,
) -> impl Responder {
    if crash_flag.read().await.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().body("Node is simulating a crash");
    }
    let node_ref = node_data.read().await;
    let node = &*node_ref;
    HttpResponse::Ok().json(node.resp_keys.clone())
}


#[get("/succ_list")]
async fn get_succ_list_node(
    SuccListNode: web::Data<Arc<RwLock<Vec<SuccListNode>>>>,
) -> impl Responder {
    let succ_list_ref = SuccListNode.read().await;

    HttpResponse::Ok().json(succ_list_ref.clone())
}

/// Handler for POST requests to simulate a node crash.
///
/// # Arguments
///
/// * `crash_flag` - A shared atomic boolean flag to indicate if a node should simulate a crash.
///
/// # Returns
///
/// Returns an HTTP response with a message indicating that the crash simulation has started.
#[post("/sim-crash")]
async fn post_sim_crash(crash_flag: web::Data<Arc<RwLock<AtomicBool>>>) -> impl Responder {
    let crash_flag = crash_flag.write().await;
    crash_flag.store(true, Ordering::SeqCst);
    HttpResponse::Ok().body("Crash simulation started")
}

/// Handler for the POST request to simulate node recovery after a crash.
///
/// # Arguments
///
/// * `crash_flag` - A shared atomic boolean flag to indicate if a crash has occurred.
///
/// # Returns
///
/// Returns an HTTP response with a message indicating that the crash simulation has stopped.
#[post("/sim-recover")]
async fn post_sim_recover(crash_flag: web::Data<Arc<RwLock<AtomicBool>>>) -> impl Responder {
    let crash_flag = crash_flag.write().await;
    crash_flag.store(false, Ordering::SeqCst);
    HttpResponse::Ok().body("Crash simulation stopped")
}
