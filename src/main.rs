mod client_packet;
mod server_packet;
mod flags;
mod client;
mod server;

use std::{fs, io::{Read, Write}, net::{SocketAddr, TcpListener, TcpStream}, path::PathBuf, sync::{Arc, RwLock}};
use client::Client;
use iron_oxide::net::{HTTPRequest, RequestType, WebSocket, HTTPS};
use server::Engine;


pub const SAVE_LOCATION: &str = "../save";
//pub const WEBSITE_FOLDER: &str = "C:/Dev/home_storage_web_frontend";
pub const WEBSITE_FOLDER: &str = "../home_storage_web_frontend";

const HTMLSTE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\ncharset=UTF-8\r\n\r\n";

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1889").unwrap();
    let mut engine = Arc::new(RwLock::new(Engine::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Ok(ip) = stream.peer_addr() {
                   handle_client(stream, ip, &mut engine);
                }
            },
            Err(_) => continue,
        }
    }
}

fn handle_client(mut stream: TcpStream, ip: SocketAddr, engine: &mut Arc<RwLock<Engine>>) {

    let mut buffer: [u8; 1024] = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(_) => (),
        Err(_) => return
    }

    let request = match HTTPRequest::parse(&buffer) {
        Some(request) => request,
        None => return
    };

    match request.request {
        //Just get the html fronend
        RequestType::GET(address, query) => {

            if address == "/" {
                let mainsite_html = fs::read(WEBSITE_FOLDER.to_string() + "/index.html").unwrap();
                let mut mainsite = Vec::with_capacity(HTMLSTE.len() + mainsite_html.len());
                mainsite.extend_from_slice(HTMLSTE);
                mainsite.extend_from_slice(&mainsite_html);
                stream.write(&mainsite).unwrap();
                stream.flush().unwrap();
                return;
            }

            if let Some(query) = query {

                if query != "src=save" {
                    return;
                } else if engine.read().unwrap().ip_connections(ip.ip()) == 0 {
                    return;
                }

                println!("query: {}, imgage: {}", query, SAVE_LOCATION.to_string() + &address);

                match HTTPS::format_content((SAVE_LOCATION.to_string() + &address).into()) {
                    Some(file_content) => {
                        stream.write(&file_content).unwrap();
                    },
                    None => {
                        println!("not found");
                        let c_address: String = WEBSITE_FOLDER.to_string() + "/404.html";
                        stream.write(&HTTPS::format_content(PathBuf::from(c_address)).unwrap()).unwrap();
                    }
                }
            } else {
                println!("request: {}", WEBSITE_FOLDER.to_string() + &address);
                match HTTPS::format_content((WEBSITE_FOLDER.to_string() + &address).into()) {
                    Some(file_content) => {
                        stream.write(&file_content).unwrap();
                    },
                    None => {
                        let c_address: String = WEBSITE_FOLDER.to_string() + "/404.html";
                        stream.write(&HTTPS::format_content(PathBuf::from(c_address)).unwrap()).unwrap();
                    }
                }
            }
            
        },
        //Try to connect via websocket
        RequestType::UpgradeWs(key) => {

            {
                let mut engine = engine.write().unwrap();
                if engine.ip_connections(ip.ip()) > 3 {
                    engine.kick_ip(ip.ip());
                    println!("kick");
                    return;
                } else {
                    engine.kick_ip(ip.ip());
                }
            }

            if let Some(ws) = WebSocket::try_connect(stream, &key) {
                println!("Ws connected");

                let client = Arc::new(RwLock::new(Client::new(engine.clone(), ws)));
                let mut engine = engine.write().unwrap();
                if let None = engine.add_client(ip, client) {
                    println!("not succest");
                    return;
                }
            }

        },
        RequestType::POST => ()
    }

}