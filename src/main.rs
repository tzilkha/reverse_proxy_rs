use std::env;
use actix_web::{web, App, HttpResponse, HttpServer, HttpRequest, Responder};
use awc::Client;
use awc::http::StatusCode;
use std::time::Duration;
use std::sync::{Arc, Mutex};

mod cache;
use cache::{Request, Cache};

// Get request handler
async fn handle_request(
        req: HttpRequest,
        data: web::Data<Arc<Mutex<Cache<Request, web::Bytes>>>>,
        origin: web::Data<String>
    ) -> impl Responder {

    // Create a new Request object with path and query information
    let current_request = Request::new(
        req.path().to_string(),
        req.query_string().to_string()
    );

    // Create the full path based on the origin
    let mut full_path = format!("https://{}{}", 
        origin.get_ref(),
        current_request.path
    );

    // If there was also a query add that to 
    if current_request.query_string != "" {
        full_path = format!("{}?{}",
        full_path,
        current_request.query_string 
        )
    }
    
    println!("Incoming GET request [{}]", full_path);

    // Check if this request is in cache
    if let Some(response_body) = data.lock().unwrap().get(&current_request) {
        println!("Got from cache.");

        // If so, return cached response
        return HttpResponse::Ok().body(response_body.clone());
    }

    // If not cached, make the request
    let client: Client = Client::default();

    let mut res = client
         .get(full_path) 
         .send()          
         .await
         .unwrap();

    // Lets not cache errors
    match res.status() {
        StatusCode::OK => {
            let body = res.body().await.unwrap();

            println!("Caching.");

            // Cache it
            data.lock().unwrap().insert(current_request, body.clone());

            // Return the response
            return HttpResponse::Ok().body(body.clone());
        }
        _ => {
            // If there is an error, return the error and body message
            println!("Error {}, not caching.", res.status());

            // Return the same error and body
            let body = res.body().await.unwrap();
            return HttpResponse::build(res.status()).body(body.clone());
        }
     }        
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Get the args
    let args: Vec<String> = env::args().collect();

    // FIX: Forsure a nicer way to do this
    if args.len() != 3 {
        panic!("Run as follows: cargo run [origin] [port].");
    }

    // Origin we wrap in Data as it will be accessed by the different threads
    let origin = web::Data::new(args[1].clone());
    let port = args[2].parse::<i32>().unwrap();

    // Console logging
    println!("Reverse proxy starting..");
    println!("Origin: {}", args[1]);
    println!("Port: {}", port);
    println!();

    // Create a Cache object with 30 second TTL
    // Wrap it in Mutex so that we can mutate it between threads
    // Wrap it in web Data object to be passed to Actix server
    let cache: web::Data<Arc<Mutex<Cache<Request, web::Bytes>>>> = web::Data::new(
        Arc::new(
            Mutex::new(
                Cache::new(
                    Duration::new(30,0)
                )
            )
        )
    );

    // Start server, route all GET requests to handle_request function
    // We pass it the shared data, the cache and origin
    HttpServer::new(move || {
        App::new()
            .app_data(cache.clone())
            .app_data(origin.clone())
            .route("/{path:.*}", web::get().to(handle_request))
    })
    .bind(("localhost", port as u16))?
    .run()
    .await
}