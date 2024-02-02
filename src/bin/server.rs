use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
};

use bytes::Bytes;
use eyre::{bail, Context, Result};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tokio::sync::mpsc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::UnboundedSender,
    sync::Mutex
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use std::time::Instant;


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
        connection_id: ConnectionId,
        topic: Topic,
    },
    Publish {
        msg: Message,
    },
}

struct Connection {
    writer: Writer,
    connected: bool,
    connection_id: ConnectionId
}

type ConnectionId = String;

type Topic = String;

type Writer = SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>;

type Reader = SplitStream<Framed<TcpStream, LengthDelimitedCodec>>;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1884").await?;
    let (tx, mut rx) = mpsc::unbounded_channel();

    let connection_manager = tokio::spawn(async move {
        let mut writers: HashMap<ConnectionId, Connection> = HashMap::new();
        let mut connecion_ids: Vec<ConnectionId> = Vec::new() ;
        let mut topics: Arc<Mutex<HashMap<Topic, HashSet<ConnectionId>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut disconnected: HashSet<ConnectionId> = HashSet::new();
        let mut i = 0;
        let start_time = Instant::now();
        while let Some(cmd) = rx.recv().await {
            i += 1;
            if i % 10000 == 0 {
                println!("{} cmds / s", i / start_time.elapsed().as_secs());
            }
            let topics = Arc::clone(&topics);
            match cmd {
                Command::Connect {
                    connection_id,
                    i
                } => {
                    connecion_ids.push(connection_id.clone());
                    if let Some(mut con) = writers.remove(&i.to_string()) {
                        con.connection_id = connection_id.clone();
                        con.connected = true;
                        writers.insert(connection_id.clone(), con);
                        dbg!("Connected {}", connection_id);
                    };
                },
                Command::NewWriter {
                    writer,
                    i
                } => {
                    // dbg!("New connection {}", i);
                    writers.insert(i.to_string(), Connection{ writer, connected: false, connection_id: i.to_string() });
                },
                Command::Publish { msg } => {
                    let tpcs = topics.lock().await;
                    if let Some(cons) = tpcs.get(&msg.topic) {
                        let msg_raw = MessageRaw::from(msg);
                        let msg_vec: Vec<u8> = msg_raw.try_into().unwrap();
                        let msg_b: Bytes = Bytes::from(msg_vec);
                        for con in cons {
                            if let Some(w) = writers.get_mut(con) {
                                w.writer.send(msg_b.clone()).await;
                            } else {
                                disconnected.insert(con.clone());
                            }
                        };


                    };
                },
                Command::Subscribe {
                    i,
                    topic,
                } => {
                    if let Some(connection_id) = connecion_ids.get(i) {
                        // dbg!("Subscribe {} to {}", connection_id, topic.clone());
                        match topics.lock().await.entry(topic) {
                            Entry::Vacant(v) => {
                                v.insert(HashSet::from([connection_id.clone()]));
                            }
                            Entry::Occupied(o) => {
                                o.into_mut().insert(connection_id.clone());
                            }
                        }
                    }
                },
                Command::Unsubscribe {
                    connection_id,
                    topic,
                } => match topics.lock().await.entry(topic) {
                    Entry::Vacant(v) => (),
                    Entry::Occupied(o) => {
                        o.into_mut().remove(&connection_id);
                    }
                },
            }
        }
    });
    let mut i = 0;

    loop {
        let (stream, _) = listener.accept().await?;
        let tx2 = tx.clone();
        let mut transport = Framed::new(stream, LengthDelimitedCodec::new());
        let (mut writer, mut reader) = transport.split();
        tx2.send(Command::NewWriter { i, writer})?;
        tokio::spawn(async move { process_reader(reader, tx2, i).await });
        i += 1;
    }
}
async fn process_reader(
    mut reader: Reader,
    tx: UnboundedSender<Command>,
    i: usize,
) -> Result<()> {
    println!("Started process reader");
    while let Some(frame) = reader.next().await {
        let frame = frame.context("Frame from transport")?;
        let msg: Message = Message::try_from(
            MessageRaw::try_from(frame.as_ref()).context("Parsing Bytes to Msg")?,
        )?;

        match msg.message_type {
            MessageType::Publish => tx.send(Command::Publish { msg: msg.clone() })?,
            MessageType::Subscribe => tx.send(Command::Subscribe { i, topic: msg.topic.clone() })?,
            MessageType::Unsubscribe => todo!(),
            MessageType::Connect=> tx.send(Command::Connect { connection_id: msg.message.clone(), i})?
        }
        // dbg!("{:?}", msg);
    }
    Ok(())
}

