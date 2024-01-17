use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Error};
use actix::prelude::{Actor, StreamHandler};
use actix_web_actors::ws;
use actix::AsyncContext;

use std::time::{Duration, Instant};

use sysinfo::{System};

struct MyWs {
    heartbeat: Instant
}

impl MyWs {
    pub fn new() -> Self {
        Self { heartbeat: Instant::now() }
    }

    fn heartbeat(&mut self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| {
            let mut sys = System::new_all();
            println!("Heartbeat starting");
            println!("Collecting system information");
            sys.refresh_all();
            ctx.text(format!("Total memory usage: {} gigabytes", sys.used_memory() as f32/1024.0/1024.0/1024.0));
        });
    }
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("Processing an incoming message");
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                println!("Processing text message, text received: {}", text);
                let processed_text = text.to_uppercase();

                ctx.text(processed_text);
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs::new(), &req, stream);
    println!("{:?}", resp);
    resp
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Test?")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/ws", web::get().to(index))
            .route("/aaaah", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8001))?
    .run()
    .await
}
