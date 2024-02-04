use std::{
    collections::hash_map::Entry,
    sync::Arc,
};

use rustc_hash::{
    FxHashMap,
    FxHashSet,
};

use bytes::Bytes;
use eyre::{Context, Result};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::UnboundedSender,
    sync::RwLock,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use mini_mqtt::message::*;

#[derive(Debug)]
enum Command {
    NewWriter {
        i: usize,
        writer: Writer,
    },
    Connect {
        connection_id: ConnectionId,
        i: usize,
    },
    Subscribe {
        i: usize,
        topic: Topic,
    },
    Unsubscribe {
        i: usize,
        topic: Topic,
    },
    Publish {
        msg: Message,
    },
}

struct Connection {
    writer: Writer,
    connected: bool,
    connection_id: ConnectionId,
}

type ConnectionId = String;

type Topic = String;

type Writer = SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>;

type Reader = SplitStream<Framed<TcpStream, LengthDelimitedCodec>>;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1884").await?;
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Launch connection manager 
    tokio::spawn(async move {
        // Tcp Sinks
        let mut writers: FxHashMap<ConnectionId, Connection> = FxHashMap::default();
        let mut connecion_ids: Vec<ConnectionId> = Vec::new();

        // All topics and the subscribed connections
        let topics: Arc<RwLock<FxHashMap<Topic, FxHashSet<ConnectionId>>>> =
            Arc::new(RwLock::new(FxHashMap::default()));
        // Not used - was intended for handling disconnections of which the server wasnt notified
        // let mut disconnected: FxHashSet<ConnectionId> = FxHashSet::default();
        let mut msg_count = 0;
        let mut start_time = Instant::now();
        while let Some(cmd) = rx.recv().await {
            msg_count += 1;
            if msg_count % 1000000 == 0 {
                println!("{} cmds / s", msg_count as f64 / start_time.elapsed().as_secs_f64());
                msg_count = 0;
                start_time = Instant::now();
            }
            let topics = Arc::clone(&topics);
            match cmd {
                Command::Connect { connection_id, i } => {
                    connecion_ids.push(connection_id.clone());
                    if let Some(mut con) = writers.remove(&i.to_string()) {
                        con.connection_id = connection_id.clone();
                        con.connected = true;
                        writers.insert(connection_id.clone(), con);
                        println!("Connected {}", connection_id);
                    };
                }
                Command::NewWriter { writer, i } => {
                    // println!("New connection {}", i);
                    writers.insert(
                        i.to_string(),
                        Connection {
                            writer,
                            connected: false,
                            connection_id: i.to_string(),
                        },
                    );
                }
                Command::Publish { msg } => {
                    let tpcs = topics.read().await;
                    if let Some(cons) = tpcs.get(&msg.topic) {
                        let msg_raw = MessageRaw::from(msg);
                        let msg_vec: Vec<u8> = msg_raw.try_into().unwrap();
                        let msg_b: Bytes = Bytes::from(msg_vec);
                        for con in cons {
                            if let Some(w) = writers.get_mut(con) {
                                let _ = w.writer.send(msg_b.clone()).await;
                            } 
                        }
                    };
                }
                Command::Subscribe { i, topic } => {
                    if let Some(connection_id) = connecion_ids.get(i) {
                        //println!("Subscribe {} to {}", connection_id, topic.clone());
                        match topics.write().await.entry(topic) {
                            Entry::Vacant(v) => {
                                v.insert(FxHashSet::from_iter([connection_id.clone()]));
                            }
                            Entry::Occupied(o) => {
                                o.into_mut().insert(connection_id.clone());
                            }
                        }
                    }
                }
                Command::Unsubscribe { i, topic } => {
                    if let Some(connection_id) = connecion_ids.get(i) {
                        println!("Unsubscribe {} from {}", connection_id, topic.clone());
                        match topics.write().await.entry(topic) {
                            Entry::Vacant(_) => (),
                            Entry::Occupied(o) => {
                                o.into_mut().remove(connection_id);
                            }
                        }
                    }
                }
            }
        }
    });
    let mut i = 0;

    loop {
        let (stream, _) = listener.accept().await?;
        let tx2 = tx.clone();
        let transport = Framed::new(stream, LengthDelimitedCodec::new());
        let (writer, reader) = transport.split();
        tx2.send(Command::NewWriter { i, writer })?;
        tokio::spawn(async move { process_reader(reader, tx2, i).await });
        i += 1;
    }
}

async fn process_reader(mut reader: Reader, tx: UnboundedSender<Command>, i: usize) -> Result<()> {
    // Listen for messages
    println!("Started process reader");
    while let Some(frame) = reader.next().await {
        let frame = frame.context("Frame from transport")?;
        let msg: Message = Message::try_from(
            MessageRaw::try_from(frame.as_ref()).context("Parsing Bytes to Msg")?,
        )?;

        match msg.message_type {
            MessageType::Publish => tx.send(Command::Publish { msg: msg.clone() })?,
            MessageType::Subscribe => tx.send(Command::Subscribe {
                i,
                topic: msg.topic.clone(),
            })?,
            MessageType::Unsubscribe => tx.send(Command::Unsubscribe {
                i,
                topic: msg.topic.clone(),
            })?,
            MessageType::Connect => tx.send(Command::Connect {
                connection_id: msg.message.clone(),
                i,
            })?,
        }
        // dbg!(msg);
    }
    Ok(())
}
