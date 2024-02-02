use deku::DekuWrite;
use eyre::{Result, bail, Context};
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, BufReader, AsyncSeekExt, AsyncSeek, AsyncWrite, AsyncRead}};
use tokio_stream::{StreamExt};
use futures::sink::SinkExt;
use bytes::Bytes;
use tokio_util::{codec::{LengthDelimitedCodec, Framed}, bytes::BufMut};

use mini_mqtt::message::*;

#[tokio::main]
async fn main() -> Result<()>{

    let mut stream= TcpStream::connect("127.0.0.1:1884").await?;
    let mut transport = Framed::new(stream, LengthDelimitedCodec::new());
    // load the frame into a Vec<u8>
    let message = "Hallo Welt".as_bytes().to_owned();
    let topic= "Hallo".as_bytes().to_owned();
    let msg= MessageRaw {
            message_type: MessageType::Publish,
            topic_length: topic.len() as u8,
            topic,
            message
    };
    let msg_b: Vec<u8> = msg.try_into().unwrap();
    dbg!(&msg_b);
    transport.send(Bytes::from(msg_b)).await?;
    tokio::spawn(async move {
        listen_for_messages(transport).await.unwrap();
    }).await.unwrap();
    transport.send(Bytes::from(msg_b)).await?;

    Ok(())
}


async fn listen_for_messages(mut transport: Framed<TcpStream, LengthDelimitedCodec>) -> Result<()> {
    while let Some(frame) = transport.next().await {
        let frame = frame.context("Frame from transport")?;
        let msg: Message = Message::try_from(MessageRaw::try_from(frame.as_ref()).context("Parsing Bytes to Msg")?)?;
        println!("{:?}", msg);
    };
    Ok(())
}
