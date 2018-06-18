#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate structopt_derive;
#[macro_use]
extern crate html5ever;

extern crate ansi_term;
extern crate chrono;
extern crate futures;
extern crate structopt;
extern crate tokio_core;
extern crate olifants;

mod error;
mod args;

use ansi_term::Colour;
use args::Args;
use chrono::offset::Local;
use error::*;
use futures::Stream;
use html5ever::QualName;
use html5ever::rcdom::{self, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use olifants::Client;
use std::default::Default;
use std::string::String;
use tokio_core::reactor::Core;

quick_main!(|| -> Result<()> {
    let mut core = Core::new().chain_err(|| "could not create Core")?;
    let client = Client::new(&core.handle(), "gojya").chain_err(
        || "could not create Client",
    )?;


    let args = Args::init()?;

    // If the connection is dropped or an error occurs, wait and retry
    loop {
        let timeline = client
            .timeline(
                &args.instance_url,
                args.access_token.clone(),
                args.endpoint.clone(),
            )
            .chain(::futures::stream::once(Err("stream closed".into())))
            .for_each(|event| {
                use olifants::timeline::Event::*;

                match event {
                    Update(status) => handle_event(*status),
                    _ => (),
                };

                Ok(())
            });

        println!("Connecting to {}", args.instance_url);

        if let Err(e) = core.run(timeline) {
            // TODO: stderr
            println!(
                "Encountered error:\n{}",
                error_chain::ChainedError::display(&e)
            );
        }

        // TODO: Exponential backoff
        let delay = ::std::time::Duration::from_secs(5);
        println!("Retrying in 5 seconds...");
        std::thread::sleep(delay);
    }

    // This needs to be here to satisfy the return type, even though it's unreachable
    #[allow(unreachable_code)] Ok(())
});

fn handle_event(status: olifants::api::v1::Status) -> () {
    let spoiler = remove_html(&status.spoiler_text);
    let content = remove_html(&status.content);

    let body = if spoiler.is_empty() {
        content
    } else {
        format!(
            "{}\n\n{}",
            spoiler,
            Colour::White.on(Colour::White).paint(content)
        )
    };

    // TODO: Add flag for 12-hour time
    let timestamp = status.created_at.with_timezone(&Local).format(
        "%Y/%m/%d %H:%M:%S",
    );

    print!(
        "{}{} {}\n{}\n{}",
        Colour::Green.paint("@"),
        Colour::Green.paint(status.account.acct),
        Colour::Cyan.paint(status.account.display_name),
        Colour::Blue.paint(format!("{}", timestamp)),
        body
    );
}

fn remove_html(text: &str) -> String {
    let node = html5ever::parse_fragment(
        RcDom::default(),
        Default::default(),
        QualName::new(None, ns!(), local_name!("")),
        Vec::new(),
    ).one(text);

    flatten(String::new(), node.document)
}

fn flatten(mut acc: String, node: rcdom::Handle) -> String {
    match node.data {
        NodeData::Text { ref contents } => acc.push_str(&contents.borrow()),
        _ => (),
    }

    for child in node.children.borrow().iter() {
        acc = flatten(acc, child.clone());
    }

    match node.data {
        NodeData::Element { ref name, .. } => {
            match name.local {
                local_name!("p") => acc.push_str("\n\n"),
                local_name!("br") => acc.push_str("\n"),
                _ => (),
            }
        }

        _ => (),
    }

    acc
}
