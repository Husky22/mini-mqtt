use bytes::Bytes;
use eyre::{Context, Result};
use futures::sink::SinkExt;
use futures::{stream::SplitStream, StreamExt};
use rand::prelude::*;
use tokio::{
    net::TcpStream,
    time::{sleep, Duration},
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use mini_mqtt::message::*;

#[tokio::main]
async fn main() -> Result<()> {

    let stream = TcpStream::connect("127.0.0.1:1884").await?;
    let transport = Framed::new(stream, LengthDelimitedCodec::new());
    let (mut writer, reader) = transport.split();

    let mut rng = rand::thread_rng();
    let n: usize = rng.gen_range(0..=100);

    let message = format!("Client {}", n).as_bytes().to_owned();
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
            message: format!("Hi from {}", n).as_bytes().to_owned(),
        };
        let msg_b: Vec<u8> = msg.try_into().unwrap();
        // println!("{}", msg_b.len());
        // println!("Send");
        writer.send(Bytes::from(msg_b)).await?;
        // sleep(Duration::from_secs(1)).await;
    }
    // let topic = "Ping".as_bytes().to_owned();
    // let msg = MessageRaw {
    //     message_type: MessageType::Unsubscribe,
    //     topic_length: topic.len() as u8,
    //     topic,
    //     message: "".as_bytes().to_owned(),
    // };
    // let msg_b: Vec<u8> = msg.try_into().unwrap();
    // writer.send(Bytes::from(msg_b)).await?;
    // Ok(())
}

async fn listen_for_messages(
    mut transport: SplitStream<Framed<TcpStream, LengthDelimitedCodec>>,
) -> Result<()> {
    while let Some(frame) = transport.next().await {
        let frame = frame.context("Frame from transport")?;
        let msg: Message = Message::try_from(
            MessageRaw::try_from(frame.as_ref()).context("Parsing Bytes to Msg")?,
        )?;
        // println!("{:?}", msg);
    }
    Ok(())
}
