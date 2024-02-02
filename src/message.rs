use deku::prelude::*;
use eyre::Result;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Copy)]
#[deku(type = "u8")]
pub enum MessageType {
    #[deku(id = "0x00")]
    Publish,
    #[deku(id = "0x01")]
    Subscribe,
    #[deku(id = "0x02")]
    Unsubscribe,
    #[deku(id = "0x03")]
    Connect,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
pub struct MessageRaw {
    pub message_type: MessageType,
    pub topic_length: u8,
    #[deku(count = "topic_length")]
    pub topic: Vec<u8>,
    #[deku(count = "deku::rest.len() / 8")]
    pub message: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub message_type: MessageType,
    pub topic: String,
    pub message: String,
}

impl TryFrom<MessageRaw> for Message {
    type Error = eyre::Report;
    fn try_from(value: MessageRaw) -> std::prelude::v1::Result<Self, Self::Error> {
        let topic = String::from_utf8(value.topic)?;
        let message = String::from_utf8(value.message)?;
        Ok(Message {
            message_type: value.message_type,
            topic,
            message,
        })
    }
}

impl From<Message> for MessageRaw {
    fn from(value: Message) -> Self {
        let topic = value.topic.as_bytes().to_owned();
        let message = value.message.as_bytes().to_owned();
        MessageRaw {
            message_type: value.message_type,
            topic_length: topic.len() as u8,
            topic,
            message,
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn serialize_deserialize() -> Result<()> {
        let topic = "Hallo".as_bytes().to_owned();
        let message = "Hallo".as_bytes().to_owned();
        let m = MessageRaw {
            message_type: MessageType::Publish,
            topic_length: topic.len() as u8,
            topic,
            message,
        };
        let b: Vec<u8> = m.clone().try_into()?;

        let res = MessageRaw::try_from(b.as_slice())?;
        assert_eq!(res, m);
        Ok(())
    }

    #[test]
    fn test_same_as_byte() -> Result<()> {
        let a = b"\0\x05HalloHallo Welt";
        let a_vec = a.to_vec();
        let topic = "Hallo".as_bytes().to_owned();
        let message = "Hallo Welt".as_bytes().to_owned();
        let m = MessageRaw {
            message_type: MessageType::Publish,
            topic_length: topic.len() as u8,
            topic,
            message,
        };
        let b: Vec<u8> = m.clone().try_into().unwrap();
        assert_eq!(a_vec, b);
        Ok(())
    }
}
