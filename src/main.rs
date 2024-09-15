use std::collections::HashSet;
use std::env::args;
use reqwest;
use serde_json::Value;
use rayon::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let domain = args().nth(1).unwrap_or_else(|| {
        println!("Missing Domain as first parameter");
        println!("Usage:");
        println!("cargo run -- \"example.com\"");
        std::process::exit(1);
    });
    let url = format!("https://crt.sh/?q=%25.{}&output=json", domain);

    // Using blocking reqwest for simplicity since we're not using tokio anymore
    let response = reqwest::blocking::get(&url)?;
    let json: Vec<Value> = response.json()?;

    let subdomains: HashSet<String> = json
        .into_iter()
        .filter_map(|entry| entry["name_value"].as_str().map(String::from))
        .collect();

    println!("Found {} unique subdomains", subdomains.len());

    let popular_ports = vec![80, 443, 22, 21, 25, 3306, 8080, 8443];

    // Using Rayon for parallel processing
    subdomains.par_iter().for_each(|subdomain| {
        // Skip subdomains with wildcards or other invalid characters
        if subdomain.contains('*') || subdomain.contains('?') {
            eprintln!("Skipping invalid subdomain: {}", subdomain);
            return;
        }

        for port in &popular_ports {
            let address = format!("{}:{}", subdomain, port);
            match (subdomain.as_str(), *port).to_socket_addrs() {
                Ok(mut addrs) => {
                    if let Some(socket_addr) = addrs.next() {
                        match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(1)) {
                            Ok(_) => println!("Open port found: {}", address),
                            Err(_) => {}
                        }
                    }
                },
                Err(e) => eprintln!("Failed to resolve address {}: {}", address, e),
            }
        }
    });

    Ok(())
}
