# Reverse Proxy RS
This is an implementation of a very simple and limited reverse proxy. It uses a cache to keep track of incoming requests and if an identical request has been received within a specific timeframe, the cached result will be returned, rather than querying the origin again.

## Request
A Request structure is created in order to keep track of the important information from a request. For this very basic demo, we only keep track of the path (from origin) and query (reason for keeping a query field is that it is actually split in incoming `actix` requests, so we maintain this information in case we need to query the origin - we can just append it to path). The structure implements equality and hashing so that it may be used as a key in the HashMap. Additional fields can easily be added such as "body" or "requester" information when this would be used for more than just GET requests.

## Cache
The cache is implemented using a simple HashMap in which values are Request objects and the values are 2-tuples of the body content (from the origin) and an Instant variable which defines the end of the entry's lifetime. The HashMap is conveniently wrapped in the Cache struct to conveniently handle setting the Instant variable based on the 30 second TTL and returning None when the HashMap is queried and the time is expired.

For this scope I did not handle cleaning up values which have expired. If I were to continue working on it, I was thinking of having a max size for the cache (a variable in the cache object), as things are added, if that size is hit, then we would call a method that would go through the map and remove all expired elements. Of course, it could be the case that we would have more valid elements than max size, so there are two options which come to mind - one could be to dynamically grow this max size (perhaps by increasing the size by a factor of two each time) or simply limiting new entries in order to make sure the cache doesn't exceed a specific size. Of course there are pros and cons to both of these which can be explored and a more involved scheme could be created for the dynamic approach, but I leave that for another time.

## Server
The server uses the `actix_web` library and reroutes all GET requests to a single handler funtion. We use `Arc` and `Mutex` to be able to share and mutate data across the spawned threads (specifically the cache and origin).

# Running

Simply run,

```
cargo run [origin] [port]
```

For example,

```
cargo run blockstream.info 8080
```

Tests can easily be done through Postman or any other method localy `https://[origin]:[port]/[path?]`.

# Improvements
Besides the improements made to cache optimization there are also areas where error handling could be better (invalid port, origin etc.). Additionally a way to dynamically decide the TTL on starting the program would be a better user experience - right now its fixed on 30 seconds. And many more..
