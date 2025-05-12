# Local Search Shortcuts

A local proxy service for redirecting to duckduckgo's [bangs](https://duckduckgo.com/bangs) feature in any search engine.

All pre-loaded shortcuts are generated from [`bang.js`](https://duckduckgo.com/bang.js).

## Usage Instructions:

After running, just set this as the search engine in your browser:
```
http://localhost:9321/?q=[TERMS]
```

Then use the many search engine shortcuts like so:
```
!wiki Hello World
```

This immediately redirects to the Wikipedia page or search results.

## Configuration File

```toml
# Located at <CONFIG DIRECTORY>/lss/config.toml or ./lss.toml

port = 9321 # host on this port
broadcast = false # make accessible to other devices on the network
default = "duckduckgo" # the default search engine (duckduckgo, google, bing, etc.)

[engines]
homemanager = "https://home-manager-options.extranix.com/?query={s}"
# if "{s}" is not present, it will always just redirect regardless of the search terms
# now you can search for "!homemanager vim"
```
