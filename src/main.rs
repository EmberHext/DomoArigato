use std::collections::HashSet;
use std::io::BufRead;
use std::process;
use std::sync::{Arc, Mutex};

use reqwest::blocking::Client;
use reqwest::StatusCode;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

pub fn check_responses(url: &str, only200: bool) {
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
                    eprintln!("\x1b[31me.g: parsero -u www.behindthefirewalls.com -o -sb\x1b[0m\n");
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
            println!("\x1b[32m{} {} {:?}\x1b[0m", disurl, status.as_u16(), status.canonical_reason().expect("Something went wrong fetching the return"));
            let mut count_ok = count_ok.lock().unwrap();
            *count_ok += 1;
        } else if !only200 {
            println!("\x1b[31m{} {} {:?}\x1b[0m", disurl, status.as_u16(), status.canonical_reason().expect("Something went wrong fetching the return"));
        }
    });
    
    let count = *count.lock().unwrap();
    let count_ok = *count_ok.lock().unwrap();
    
    if count_ok != 0 {
        println!("\n[+] {} links have been analyzed and {} of them are available.", count, count_ok);
    } else if only200 {
        println!("\n\x1b[31m[+] {} links have been analyzed, none are available.\x1b[0m", count);
    } else {
        println!("\n\x1b[31m[+] {} links have been analyzed, none are available.\x1b[0m", count);
    }
} 

fn main() {
    check_responses("reddit.com", false);
}