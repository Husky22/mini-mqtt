# Mini-Mqtt

After doing the HTTP-Server Codecrafters challenge I wanted to try to create my own MQTT-Broker in Rust.
As I didn't want to spend too much time to implement the full MQTT-Spec I limited myself to a broadcasting server with topic subscription functionality.

This was a nice exercise in using Tokio, especially its Message-Passing mechanisms and develop a custom codec using [Deku](https://docs.rs/deku/latest/deku/)

## Benchmarking
On my 2019 MBP i9-Version with 8-cores I get 12000 Messages/s with a message size of 15 Bytes.
The message is as follows:
```
Message { message_type: Publish, topic: "Ping", message: "Hi from 6" }
```
There is significant room for performance enhancement, potentially through the application of multithreading techniques and more efficient memory management.
Especially by avoiding ArcMutex on the topic HashMap.
Given that the majority of operations involve reading from the hashmap, and only a minority (subscribe/unsubscribe requests) modify it, there may be more efficient concurrency solutions that could further optimize performance.
