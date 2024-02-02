use bytes::Bytes;
use deku::DekuWrite;
use eyre::{bail, Context, Result};
use futures::sink::SinkExt;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, BufReader},
    net::{TcpListener, TcpStream},
    time::{sleep, Duration},
};
use futures::{StreamExt, stream::SplitStream};
use tokio_util::{
    bytes::BufMut,
    codec::{Framed, LengthDelimitedCodec},
};
use rand::prelude::*;

use mini_mqtt::message::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:1884").await?;
    let mut transport = Framed::new(stream, LengthDelimitedCodec::new());
    let (mut writer, mut reader) = transport.split();
    // load the frame into a Vec<u8>
    let mut rng = rand::thread_rng();
    let n:usize = rng.gen_range(0..=100);
    let message = format!("Niklas {}", n).as_bytes().to_owned();
    let topic = "Ping".as_bytes().to_owned();
    let msg = MessageRaw {
        message_type: MessageType::Connect,
        topic_length: topic.len() as u8,
        topic,
        message,
    };
    let msg_b: Vec<u8> = msg.try_into().unwrap();
    writer.send(Bytes::from(msg_b)).await?;
    let topic = "Ping".as_bytes().to_owned();
    let msg = MessageRaw {
        message_type: MessageType::Subscribe,
        topic_length: topic.len() as u8,
        topic,
        message: "".as_bytes().to_owned(),
    };
    let msg_b: Vec<u8> = msg.try_into().unwrap();
    writer.send(Bytes::from(msg_b)).await?;
    tokio::spawn(async move {
        listen_for_messages(reader).await.unwrap();
    });
    loop {
        let topic = "Ping".as_bytes().to_owned();
        let msg = MessageRaw {
            message_type: MessageType::Publish,
            topic_length: topic.len() as u8,
            topic,
            message: format!("Hi {}", n).as_bytes().to_owned(),
        };
        let msg_b: Vec<u8> = msg.try_into().unwrap();
        println!("Send");
        writer.send(Bytes::from(msg_b)).await?;
    }

    Ok(())
}

async fn listen_for_messages(mut transport: SplitStream<Framed<TcpStream, LengthDelimitedCodec>>) -> Result<()> {
    while let Some(frame) = transport.next().await {
        let frame = frame.context("Frame from transport")?;
        let msg: Message = Message::try_from(
            MessageRaw::try_from(frame.as_ref()).context("Parsing Bytes to Msg")?,
        )?;
        println!("{:?}", msg);
    }
    Ok(())
}
