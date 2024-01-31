use nom::Finish;
use nom::error::VerboseError;
use nom::{IResult, multi::length_data, number::complete::be_u8, error::ParseError, error::{ErrorKind, Error, convert_error}};
use eyre::{Result, bail, Context};


#[tokio::main]
async fn main() {
    println!("Hello, world!");
}

#[derive(Debug, PartialEq)]
pub enum CustomError<I> {
  MyError(String),
  Nom(I, ErrorKind),
}

impl<I> ParseError<I> for CustomError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    CustomError::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

enum MessageType {
    Publish,
    Subscribe
}

struct Message {
    message_type: MessageType,
    topic_name: String,
    message: Option<Vec<u8>>
}

fn parser2(s: &[u8]) -> IResult<&[u8], MessageType, VerboseError<&[u8]>> {
    let (s, mt) = be_u8(s)?;
    let message_type = match mt {
        0 => MessageType::Publish,
        1 => MessageType:: Subscribe,
        _ => Err(Error(CustomError::MyError("Test".to_owned())))
    };
    Ok((s, message_type))

}


fn parser(s: &[u8]) -> Result<Message> {
    let p = be_u8::<&[u8], ()>(s);
    let (s, mt) = match p {
        Ok(a ) => a,
        Err(_) => bail!("failure parsing first byte")
    };
    let (s, tn) = length_data(be_u8::<&[u8], (&[u8], ErrorKind)>)(s)?;
    let message = if s.is_empty() {None} else {Some(s.to_vec())};

    let topic_name = String::from_utf8(tn.to_owned()).context("Parsing topic name")?;

    Ok(Message {message_type, topic_name, message})
}
