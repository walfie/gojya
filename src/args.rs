use error::*;
use olifants;

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(short = "i", long = "instance",
                help = "Instance URL (HTTPS is assumed if protocol is unspecified)")]
    pub instance: String,

    #[structopt(short = "t", long = "token", help = "Access token")]
    pub access_token: String,

    #[structopt(help = "Stream type")]
    pub stream_type: StreamType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StreamType(pub olifants::timeline::Endpoint);

impl ::std::str::FromStr for StreamType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        use olifants::timeline::Endpoint::*;

        let result = match s {
            "user" => User,
            "notification" => Notification,
            "notifications" => Notification,
            "federated" => Federated,
            "local" => Local,
            other => Other(other.to_string()), // TODO: Error
        };

        Ok(StreamType(result))
    }
}
