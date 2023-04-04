# Domo Arigato #

A simple command line tool which quickly audits the Disallow entries of a site's robots.txt. Disallow entries can be used to stop search engines from indexing specific directories and files. Thus, sometimes it can be an easy way to find pages that it's worthwhile for the user to check out.

Domo will run through every Disallow entry, visit the page and return the HTTP Status Code. If any return 200, it means you'll be able to visit it.

You can also choose to search bing.com to see if the page has been indexed, and archive.is to see if it's been archived.

## Usage ##
```
USAGE:
    domo [OPTIONS] --url <URL>

OPTIONS:
    -a, --archive      Search the URLs on archive.is and return the results
    -b, --bing         Search the URLs on Bing and return the results
    -h, --help         Print help information
    -o, --only200      Only show results with HTTP status 200
    -u, --url <URL>    URL to check the robots.txt
    -V, --version      Print version information
```

## Example ##
```
root@kali:~/tools/domo# domo -bao -u mycoolsite.com

    ________                             _____        .__              __
    \______ \   ____   _____   ____     /  _  \_______|__| _________ _/  |_  ____  
     |    |  \ /  _ \ /     \ /  _ \   /  /_\  \_  __ \  |/ ___\__  \\   __\/  _ \ 
     |    `   (  <_> )  Y Y  (  <_> ) /    |    \  | \/  / /_/  > __ \|  | (  <_> )
    /_______  /\____/|__|_|  /\____/  \____|__  /__|  |__\___  (____  /__|  \____/ 
            \/             \/                 \/        /_____/     \/

    
http://mycoolsite.com/ImageArchive.aspx 200 "OK"

 -- 27 links have been analyzed and 1 of them are available.

Searching the Disallow entries on Bing...


!! No Disallows have been indexed on Bing

Searching the Disallow entries on archive.is...


!! No Disallows have been archived on archive.is
 
Finished in 1.13s
```
