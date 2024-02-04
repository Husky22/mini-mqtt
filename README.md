# Mini-Mqtt

After doing the HTTP-Server Codecrafters challenge I wanted to try to create my own MQTT-Broker in Rust.
As I didn't want to spend too much time to implement the full MQTT-Spec I limited myself to a broadcasting server with topic subscription functionality.

This was a nice exercise in using Tokio, especially its Message-Passing mechanisms and develop a custom codec using [Deku](https://docs.rs/deku/latest/deku/)

## Benchmarking

After replacing std::HashMap with FxHashMap and replacing the topics Mutex with a RWLock I could 10x the performance to 140000 msg/s.
The major improvement came by using RWLock though.


On my 2019 MBP i9-Version with 8-cores I got at first 12000 Messages/s with a message size of 15 Bytes.
After replacing std::HashMap with FxHashMap and replacing the `topics` Mutex with a RWLock I could 12x the performance to 140000 msg/s.
The major improvement came by using RWLock.

The message is as follows:
```
Message { message_type: Publish, topic: "Ping", message: "Hi from 6" }
```

The performance deteriorates pretty quickly as I add more clients to 30k msg/s for 3 clients. I will continue to inspect this issue. Because as soon as I disconnect all the clients, the 
performace rises to 250k msg/s.
So there should still be room for performance improvement.

## ToDo

 - [x] Given that the majority of operations involve reading from the hashmap, and only a minority (subscribe/unsubscribe requests) modify it, there may be more efficient concurrency solutions that could further optimize performance.
