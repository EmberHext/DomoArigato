# Domo Arigato #

A simple command line tool which quickly audits the Disallow entries of a site's robots.txt. Disallow entries can be used to stop search engines from indexing specific directories and files. Thus, sometimes it can be an easy way to find pages that it's worthwhile for the user to check out.

Domo will run through every Disallow entry, visit the page and return the HTTP Status Code. If any return 200, it means you'll be able to visit it.

You can also choose to search bing.com to see if the page has been indexed, and web.archive.org to see if it's been archived.

## Usage ##
```
USAGE:
    domo [OPTIONS] --url <URL>

OPTIONS:
    -a, --archive      Search the URLs on web.archive.org and return the results
    -b, --bing         Search the URLs on Bing and return the results
    -h, --help         Print help information
    -o, --only200      Only show results with HTTP status 200
    -u, --url <URL>    URL to check the robots.txt
    -V, --version      Print version information
```

## Example ##
```
root@kali:~/tools/domo# domo -abo -u targetsite.com

    ________                             _____        .__              __          
    \______ \   ____   _____   ____     /  _  \_______|__| _________ _/  |_  ____  
     |    |  \ /  _ \ /     \ /  _ \   /  /_\  \_  __ \  |/ ___\__  \\   __\/  _ \ 
     |    `   (  <_> )  Y Y  (  <_> ) /    |    \  | \/  / /_/  > __ \|  | (  <_> )
    /_______  /\____/|__|_|  /\____/  \____|__  /__|  |__\___  (____  /__|  \____/ 
            \/             \/                 \/        /_____/     \/             
       	 
    
http://targetsite.com/files/ 200 "OK"
http://targetsite.com/Unixhelp/ 200 "OK"
http://targetsite.com/icons/ 200 "OK"
http://targetsite.com/webnews/ 200 "OK"
http://targetsite.com/staff/ 200 "OK"
http://targetsite.com/stats/ 200 "OK"
http://targetsite.com/HTML_Dictionary/ 200 "OK"
http://targetsite.com/primer/ 200 "OK"

 -- 16 links have been analyzed and 8 of them are available.


Searching the Disallow entries on web.archive.org...

https://web.archive.org/targetsite.com/cgi-bin/ found
https://web.archive.org/targetsite.com/webnews/ found
https://web.archive.org/targetsite.com/~chris/directions.html found
https://web.archive.org/targetsite.com/counter.html found
https://web.archive.org/targetsite.com/%7Echris/directions.html found
https://web.archive.org/targetsite.com/~chris/weird/ found
https://web.archive.org/targetsite.com/staff/ found
https://web.archive.org/targetsite.com/primer/ found
https://web.archive.org/targetsite.com/HTML_Dictionary/ found
https://web.archive.org/targetsite.com/stats/ found
https://web.archive.org/targetsite.com/icons/ found
https://web.archive.org/targetsite.com/%7Echris/weird/ found
https://web.archive.org/targetsite.com/Unixhelp/ found
https://web.archive.org/targetsite.com/files/ found
https://web.archive.org/targetsite.com/%7Echris/omc/ found
https://web.archive.org/targetsite.com/~chris/omc/ found

 -- 16 links have been analyzed and 16 of them are available.
Finished in 1.52s](http://targetsite.com/Unixhelp/ 200 "OK"
http://targetsite.com/stats/ 200 "OK"
http://targetsite.com/files/ 200 "OK"
http://targetsite.com/primer/ 200 "OK"
http://targetsite.com/icons/ 200 "OK"
http://targetsite.com/HTML_Dictionary/ 200 "OK"
http://targetsite.com/staff/ 200 "OK"
http://targetsite.com/webnews/ 200 "OK"

 -- 16 links have been analyzed and 8 of them are available.


Searching the Disallow entries on bing.com...


 !! No Disallows have been indexed on bing.com



Searching the Disallow entries on web.archive.org...

https://web.archive.org/targetsite.com/~chris/directions.html found
https://web.archive.org/targetsite.com/cgi-bin/ found
https://web.archive.org/targetsite.com/%7Echris/weird/ found
https://web.archive.org/targetsite.com/stats/ found
https://web.archive.org/targetsite.com/%7Echris/directions.html found
https://web.archive.org/targetsite.com/%7Echris/omc/ found
https://web.archive.org/targetsite.com/~chris/omc/ found
https://web.archive.org/targetsite.com/~chris/weird/ found
https://web.archive.org/targetsite.com/primer/ found
https://web.archive.org/targetsite.com/HTML_Dictionary/ found
https://web.archive.org/targetsite.com/Unixhelp/ found
https://web.archive.org/targetsite.com/counter.html found
https://web.archive.org/targetsite.com/files/ found
https://web.archive.org/targetsite.com/staff/ found
https://web.archive.org/targetsite.com/webnews/ found
https://web.archive.org/targetsite.com/icons/ found

 -- 16 links have been analyzed and 16 of them are available.
Finished in 3.64s
```
