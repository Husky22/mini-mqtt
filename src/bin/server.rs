use std::{sync::{Arc, Mutex}, collections::{HashMap, hash_map::Entry, HashSet}};

use eyre::{Result, bail, Context};
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}, sync::mpsc::UnboundedSender};
use tokio_util::codec::{LengthDelimitedCodec, Framed};
use tokio::sync::mpsc;
use bytes::Bytes;
use futures::{sink::SinkExt, StreamExt, stream::{SplitStream, SplitSink}};

use mini_mqtt::message::*;

#[derive(Debug)]
enum Command {
    Connect {
        connection_id: ConnectionId,
        writer: Writer
    },
    Subscribe {
        connection_id: ConnectionId,
        topic: Topic,
    },
    Unsubscribe {
        connection_id: ConnectionId,
        topic: Topic,
    },
    Publish {
        msg: Message
    }
}

type ConnectionId = String;

type Topic = String;

type Writer = SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>;


#[tokio::main]
async fn main() -> Result<()>{

    let listener = TcpListener::bind("127.0.0.1:1884").await?;
    let (tx, mut rx) = mpsc::unbounded_channel();

    let connection_manager = tokio::spawn(async move {
        let mut writers: HashMap<ConnectionId, Writer> = HashMap::new();
        let mut topics: Arc<Mutex<HashMap<Topic, HashSet<ConnectionId>>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut disconnected: HashSet<&ConnectionId> = HashSet::new();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Connect { connection_id, writer } => {
                    writers.insert(connection_id, writer);
                },
                Command::Publish { msg} => {
                    match topics.lock().unwrap().get(&msg.topic) {
                        Some(c) => {c.into_iter().for_each(|c| {
                            let w = match writers.get_mut(c) {
                                Some(w) => w,
                                None => {disconnected.insert(c); return ()}
                            };
                            )},
                        None => return (),
                        };
                    
                    }
                },
                Command::Subscribe { connection_id, topic } => {
                    match topics.lock().unwrap().entry(topic) {
                        Entry::Vacant(v) => {v.insert(HashSet::from([connection_id]));},
                        Entry::Occupied(o) => {o.into_mut().insert(connection_id);}
                    }
                },
                Command::Unsubscribe { connection_id, topic } => {
                    match topics.lock().unwrap().entry(topic) {
                        Entry::Vacant(v) => {()},
                        Entry::Occupied(o) => {o.into_mut().remove(&connection_id);}
                    }
                }
            }
        };

    });


    loop {
        let (stream, _) = listener.accept().await?;
        let tx2 = tx.clone();
        let mut transport = Framed::new(stream, LengthDelimitedCodec::new());
        let (mut writer, mut reader) = transport.split();
        println!("New connection");
        let reader_process = tokio::spawn(async move {process_reader(reader, tx2)});
    }
}



async fn process_reader(mut reader: SplitStream<Framed<TcpStream, LengthDelimitedCodec>>, tx: UnboundedSender<Command>) -> Result<()> {
    while let Some(frame) = reader.next().await {
        let frame = frame.context("Frame from transport")?;
        let msg: Message = Message::try_from(MessageRaw::try_from(frame.as_ref()).context("Parsing Bytes to Msg")?)?;

        match msg.message_type {
            MessageType::Publish => todo!(),
            MessageType::Subscribe => todo!(),
            MessageType::Unsubscribe => todo!(),
        }
        println!("{:?}", msg);
    };
    Ok(())
}

async fn process_writer(mut writer: SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>) -> Result<()> {
    Ok(())
}
