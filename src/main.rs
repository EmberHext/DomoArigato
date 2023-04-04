/*
 * Domo Arigato is a free tool to audit the Robots.txt file on websites, check if any of the Disallowed pages are
 * available to be visited, and check if they'd been indexed or archived in various places.
 * 
 * Author: Ember Hext
 * GitHub: https://github.com/EmberHext
 * Twitter: @EmberHext
 * 
 * It is released under the MIT License:
 * 
 * Copyright 2023 Ember Hext
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, 
 * including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished 
 * to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
 * 
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. 
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH 
 * THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 *
 * 
 * Note: This is intended to be used for your own servers or in contexts like pentesting and CTFs where you are authorised to be engaging with the server in this manner.
 * 
 */


use std::{
    collections::HashSet,
    process,
    sync::{
        Arc,
        Mutex
    },
    error::Error,
};
use reqwest::{
    self,
    StatusCode,
    Client,
    Error as ReqwestError,
};
use clap::{
    App, 
    Arg,
};
use futures::{
    stream::{
        self, 
        FuturesUnordered
    }, 
    StreamExt, 
    TryStreamExt
};
use tokio::{
    self,
    sync::RwLock,
};
use chrono::Local;

async fn check_responses(url: &str, only200: bool, client: &Client) -> Vec<String> {
    let pathlist = Arc::new(Mutex::new(HashSet::new()));
    let robots_txt_url = format!("http://{}/robots.txt", url);

    let text = match reqwest::get(&robots_txt_url).await {
        Ok(response) => response.text().await.unwrap(),
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

    lines.iter().for_each(|line| {
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

    let client = Arc::new(client.clone());

    let count = Arc::new(Mutex::new(0));
    let count_ok = Arc::new(Mutex::new(0));

    let pathlist_locked = pathlist.lock().unwrap();
    let pathlist_cloned: Vec<String> = pathlist_locked.iter().cloned().collect();

    let mut futures = pathlist_cloned
        .iter()
        .map(|p| {
            let disurl = format!("http://{}/{}", url, p);
            let client = Arc::clone(&client);
            let count = Arc::clone(&count);
            let count_ok = Arc::clone(&count_ok);

            async move {
                let res = client.get(&disurl).send().await.unwrap();
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
            }
        })
        .collect::<FuturesUnordered<_>>();

    while let Some(_) = futures.next().await {}
    
    let count = *count.lock().unwrap();
    let count_ok = *count_ok.lock().unwrap();
    
    if count_ok != 0 {
        println!("\n -- {} links have been analyzed and {} of them are available.", count, count_ok);
    } else {
        println!("\n\x1b[31m !! {} links have been analyzed, none are available.\x1b[0m", count);
    }

    pathlist_cloned
} 

async fn search_bing(url: &str, paths: Vec<String>, client: &Client) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n\nSearching the Disallow entries on bing.com...\n");
    let pathlist = paths.clone();
    let count = pathlist.len();
    let count_ok = Arc::new(tokio::sync::Mutex::new(0));
    let client = client.clone();

    let path_stream = stream::iter(pathlist.into_iter().map(Ok::<_, ReqwestError>));
    let concurrency_limit = 10; // Adjust this to control the maximum number of parallel requests

    path_stream
        .map(|path| {
            let client = &client;
            let count_ok = count_ok.clone();
            let url = url.to_string();
            async move {
                let disurl = format!("https://www.bing.com/search?q=site:{}/{}", url, path?);
                let res = client.get(&disurl).send().await?;
                let body = res.text().await?;

                if !body.contains("no results") {
                    println!("\x1b[32m{} found\x1b[0m", disurl);
                    let mut count_ok = count_ok.lock().await;
                    *count_ok += 1;
                }
                Ok(()) as Result<(), Box<dyn Error + Send + Sync>>
            }
        })
        .buffer_unordered(concurrency_limit)
        .try_collect::<()>()
        .await?;

    let count_ok = *count_ok.lock().await;

    if count_ok == 0 {
        println!("\n\x1b[31m !! No Disallows have been indexed on bing.com\x1b[0m\n");
    } else {
        println!("\n -- {} links have been analyzed and {} of them are available.", count, count_ok);
    }

    Ok(())
}

async fn search_archive_org(url: &str, paths: Vec<String>, client: &Client) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n\nSearching the Disallow entries on web.archive.org...\n");
    let pathlist = paths.clone();
    let count = pathlist.len();
    let count_ok = Arc::new(tokio::sync::Mutex::new(0));
    let client = client.clone();

    let path_stream = stream::iter(pathlist.into_iter().map(Ok::<_, ReqwestError>));
    let concurrency_limit = 10; // Adjust this to control the maximum number of parallel requests

    path_stream
        .map(|path| {
            let client = &client;
            let count_ok = count_ok.clone();
            let url = url.to_string();
            async move {
                let disurl = format!("https://web.archive.org/web/*/{}/{}", url, path?);
                let res = client.get(&disurl).send().await?;
                let body = res.text().await?;

                if body.contains("captures") {
                    println!("\x1b[32m{} found\x1b[0m", disurl);
                    let mut count_ok = count_ok.lock().await;
                    *count_ok += 1;
                }
                Ok(()) as Result<(), Box<dyn Error + Send + Sync>>
            }
        })
        .buffer_unordered(concurrency_limit)
        .try_collect::<()>()
        .await?;

    let count_ok = *count_ok.lock().await;

    if count_ok == 0 {
        println!("\n\x1b[31m !! No Disallows have been archived on web.archive.org\x1b[0m\n");
    } else {
        println!("\n -- {} links have been analyzed and {} of them are available.", count, count_ok);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    use std::time::Instant;
    let now = Instant::now();

    println!("{}", r"

    ________                             _____        .__              __          
    \______ \   ____   _____   ____     /  _  \_______|__| _________ _/  |_  ____  
     |    |  \ /  _ \ /     \ /  _ \   /  /_\  \_  __ \  |/ ___\__  \\   __\/  _ \ 
     |    `   (  <_> )  Y Y  (  <_> ) /    |    \  | \/  / /_/  > __ \|  | (  <_> )
    /_______  /\____/|__|_|  /\____/  \____|__  /__|  |__\___  (____  /__|  \____/ 
            \/             \/                 \/        /_____/     \/             
            
    ");
    
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
        .arg(
            Arg::with_name("searcharchive")
            .short('a')
            .long("archive")
            .help("Search the URLs on archive.org and return the results")
        )
        .get_matches();


        println!("Running Domo v1");
        let time_now = Local::now();
        let formatted_datetime = time_now.format("%Y-%m-%d %H:%M:%S");
        println!("Running @ {}, starting: {}\n\n", matches.value_of("url").unwrap(), formatted_datetime);

        let client = Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();

        let pathlist = check_responses(matches.value_of("url").unwrap(), matches.is_present("only200"), &client).await;
        let shared_pathlist = Arc::new(RwLock::new(pathlist));
    
        if matches.is_present("searchbing") {
            let pathlist = {
                let pathlist = shared_pathlist.clone();
                let pathlist = pathlist.read().await;
                pathlist.clone()
            };
            if let Err(e) = search_bing(matches.value_of("url").unwrap(), pathlist, &client).await {
                eprintln!("Error: {}", e);
            }
        }
        if matches.is_present("searcharchive") {
            let pathlist = {
                let pathlist = shared_pathlist.clone();
                let pathlist = pathlist.read().await;
                pathlist.clone()
            };
            if let Err(e) = search_archive_org(matches.value_of("url").unwrap(), pathlist, &client).await {
                eprintln!("Error: {}", e);
            }
        }

    let elapsed = now.elapsed();
    println!("Finished in {:.2?}", elapsed);

    Ok(())
}