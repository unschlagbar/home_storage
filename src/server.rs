use std::{collections::HashMap, net::{IpAddr, SocketAddr}, sync::{Arc, RwLock}, thread::spawn};

use iron_oxide::net::WebSocket;

use crate::client::Client;


pub struct Engine {
    pub clients: HashMap<SocketAddr, Arc<RwLock<Client>>>,
}

impl Engine {
    pub fn new() -> Self {
        Self { clients: HashMap::with_capacity(10) }
    }

    pub fn add_client(&mut self, ip: SocketAddr, client: Arc<RwLock<Client>>) -> Option<()> {
        match self.clients.insert(ip, client) {
            None => {
                
                let client = self.clients.get(&ip).unwrap().clone();
                spawn(move || WebSocket::run(client));
                Some(())
            }
            Some(_) => return None,
        }
    }

    pub fn remove_client(&mut self, ip: SocketAddr) {
        let _ = self.clients.remove(&ip);
    }

    pub fn ip_connections(&self, ip: IpAddr) -> usize {
        let mut count = 0;

        for entry in self.clients.keys() {
            if entry.ip() == ip {
                count += 1;
            }
        }

        count
    }

    pub fn kick_ip(&mut self, ip: IpAddr)  {
        self.clients.retain(|k, v| {
            let should_retain = k.ip() != ip;
            if !should_retain {
                v.write().unwrap().ws.close();
            }
            should_retain
        });
        
    }
}