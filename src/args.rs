use error::*;
use olifants::timeline::Endpoint;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "i", long = "instance",
                help = "Instance URL (if protocol is absent, `https` is assumed)")]
    pub instance: String,

    #[structopt(short = "t", long = "token",
                help = "Access token. If unspecified, uses the \
                        `MASTODON_ACCESS_TOKEN` environment variable.")]
    pub token: Option<String>,

    #[structopt(long = "timeline", help = "Timeline type", default_value = "local",
                possible_value = "local", possible_value = "federated", possible_value = "user")]
    pub timeline: String,
}

pub struct Args {
    pub instance_url: String,
    pub access_token: String,
    pub endpoint: Endpoint,
}

impl Args {
    pub fn init() -> Result<Self> {
        let args = Opt::from_args();

        // If no protocol specified, assume `https://`
        let instance_url =
            if args.instance.starts_with("http://") || args.instance.starts_with("https://") {
                args.instance
            } else {
                format!("https://{}", args.instance)
            };

        let access_token = args.token
            .or_else(|| ::std::env::var("MASTODON_ACCESS_TOKEN").ok())
            .ok_or_else(|| {
                let msg = "please specify an access token with `--token`, or by \
                       setting the `MASTODON_ACCESS_TOKEN` environment variable";
                Error::from_kind(ErrorKind::Msg(msg.into()))
            })?;

        use olifants::timeline::Endpoint::*;

        let endpoint = match args.timeline.as_ref() {
            "user" => User,
            "notification" => Notification,
            "notifications" => Notification,
            "federated" => Federated,
            "local" => Local,
            other => Other(other.to_string()), // TODO: Error?
        };

        Ok(Args {
            instance_url,
            access_token,
            endpoint,
        })
    }
}
