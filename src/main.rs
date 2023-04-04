use std::collections::HashSet;
use std::process;
use std::sync::{Arc, Mutex};
use std::error::Error;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use select::{document::Document, predicate::Name};
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use clap::{App, Arg};
use colored::Colorize;

fn check_responses(url: &str, only200: bool) -> Vec<String> {
    let pathlist = Arc::new(Mutex::new(HashSet::new()));
    let robots_txt_url = format!("http://{}/robots.txt", url);

    let text = match reqwest::blocking::get(&robots_txt_url) {
        Ok(response) => response.text().unwrap(),
        Err(error) => {
            match error.status() {
                Some(StatusCode::NOT_FOUND) => {
                    eprintln!("\x1b[31mNo robots.txt file has been found.\x1b[0m\n");
                }
                _ => {
                    eprintln!("\x1b[31mPlease, type a valid URL. This URL can't be resolved.\x1b[0m");
                    eprintln!("\x1b[31me.g: domo -u www.example.com -o -sb\x1b[0m\n");
                }
            }
            process::exit(1);
        }
    };
    let lines = text.lines().collect::<Vec<_>>();

    lines.par_iter().for_each(|line| {
        let line_str = line.to_string();
        let path: Vec<&str> = line_str.splitn(2, ": /").collect();
        if let Some(p) = path.get(1) {
            if line_str.starts_with("Disallow") {
                let sanitized_path = p.trim_start_matches('/').trim_end_matches('\r').to_string();
                let mut pathlist = pathlist.lock().unwrap();
                pathlist.insert(sanitized_path);
            }
        }
    });

    let client = Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();
    let client = Arc::new(client);

    let count = Arc::new(Mutex::new(0));
    let count_ok = Arc::new(Mutex::new(0));

    let pathlist = Arc::clone(&pathlist);
    let pathlist = pathlist.lock().unwrap().iter().cloned().collect::<Vec<String>>();
    pathlist.par_iter().for_each(|p| {
        let disurl = format!("http://{}/{}", url, p);
        let client = Arc::clone(&client);
        let res = client.get(&disurl).send().unwrap();
        let status = res.status();
    
        {
            let mut count = count.lock().unwrap();
            *count += 1;
        }
    
        if status == StatusCode::OK {
            println!("\x1b[32m{} {} {:?}\x1b[0m", disurl, status.as_u16(), status.canonical_reason().expect("Something went wrong fetching the http response"));
            let mut count_ok = count_ok.lock().unwrap();
            *count_ok += 1;
        } else if !only200 {
            println!("\x1b[31m{} {} {:?}\x1b[0m", disurl, status.as_u16(), status.canonical_reason().expect("Something went wrong fetching the http response"));
        }
    });
    
    let count = *count.lock().unwrap();
    let count_ok = *count_ok.lock().unwrap();
    
    if count_ok != 0 {
        println!("\n[+] {} links have been analyzed and {} of them are available.", count, count_ok);
    } else {
        println!("\n\x1b[31m[+] {} links have been analyzed, none are available.\x1b[0m", count);
    }

    pathlist.clone()
} 

fn search_bing(url: &str, only200: bool, pathlist: Vec<String>) -> Result<(), Box<dyn Error>> {
    println!("\nSearching the Disallows entries in Bing...\n");

    let client = Client::new();

    let mut count = 0;
    for p in pathlist {
        let disurl = format!("http://{}/{}", url, p);
        let url2 = format!("http://www.bing.com/search?q=site:{}", disurl);
        println!("{}", url2);

        let resp = match client.get(&url2).send() {
            Ok(r) => r,
            Err(_) => continue,
        };

        let body = match resp.text() {
            Ok(t) => t,
            Err(_) => continue,
        };

        let document = Document::from_read(std::io::Cursor::new(&*body))?;

        for cite in document.find(Name("cite")) {
            let text = cite.text();
            if text.contains(url) {
                count += 1;
                let resp2 = client.get(&text).send();

                match resp2 {
                    Ok(r2) if r2.status().is_success() => {
                        println!("\x1b[32m - {} {} {}\x1b[0m", text, r2.status().as_u16(), r2.status().canonical_reason().unwrap_or("Unknown"))
                    }
                    Ok(r2) if !only200 => {
                        println!("\x1b[31m - {} {} {}\x1b[0m", text, r2.status().as_u16(), r2.status().canonical_reason().unwrap_or("Unknown"))
                    }
                    _ => (),
                }
            }
        }
    }

    if count == 0 {
        println!("\n\x1b[31m[+] No Disallows have been indexed in Bing\x1b[0m");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", r"
    ________                             _____        .__              __          
    \______ \   ____   _____   ____     /  _  \_______|__| _________ _/  |_  ____  
     |    |  \ /  _ \ /     \ /  _ \   /  /_\  \_  __ \  |/ ___\__  \\   __\/  _ \ 
     |    `   (  <_> )  Y Y  (  <_> ) /    |    \  | \/  / /_/  > __ \|  | (  <_> )
    /_______  /\____/|__|_|  /\____/  \____|__  /__|  |__\___  (____  /__|  \____/ 
            \/             \/                 \/        /_____/     \/             
            
            ".purple());
    
    let matches = App::new("Domo Arigato")
        .version("1.0")
        .author("Ember Hext (https://github.com/EmberHext)")
        .about("Performs an audit of the robots.txt Disallow entries on a page")
        .arg(
            Arg::with_name("url")
                .short('u')
                .long("url")
                .value_name("URL")
                .help("URL to check the robots.txt")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("only200")
                .short('o')
                .long("only200")
                .help("Only show results with HTTP status 200"),
        )
        .arg(
            Arg::with_name("searchbing")
            .short('b')
            .long("bing")
            .help("Search the URLs on Bing and return the results")
        )
        .get_matches();

    let pathlist = check_responses(matches.value_of("url").unwrap(), matches.is_present("only200"));

    search_bing(matches.value_of("url").unwrap(), matches.is_present("only200"), pathlist)?;

    Ok(())
}