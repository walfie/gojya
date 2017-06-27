#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate structopt_derive;
#[macro_use]
extern crate html5ever;

extern crate futures;
extern crate structopt;
extern crate tokio_core;
extern crate olifants;

mod error;
mod args;

use error::*;
use futures::Stream;
use html5ever::QualName;
use html5ever::rcdom::{self, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use olifants::Client;
use std::default::Default;
use std::string::String;
use structopt::StructOpt;
use tokio_core::reactor::Core;

quick_main!(|| -> Result<()> {
    let mut core = Core::new().chain_err(|| "could not create Core")?;
    let client = Client::new(&core.handle(), "gojya").chain_err(
        || "could not create Client",
    )?;


    let opt = args::Opt::from_args();

    // TODO: Assume HTTPS for instance if protocol unspecified
    let timeline = client
        .timeline(&opt.instance, opt.access_token, opt.stream_type.0)
        .for_each(|event| {
            use olifants::timeline::Event::*;

            match event {
                Update(status) => handle_event(*status),
                _ => (),
            };

            Ok(())
        });

    core.run(timeline).chain_err(|| "timeline failed")
});

fn handle_event(status: olifants::api::v1::Status) -> () {
    let node = html5ever::parse_fragment(
        RcDom::default(),
        Default::default(),
        QualName::new(None, ns!(), local_name!("")),
        Vec::new(),
    ).one(status.content);

    // TODO: Handle content warning text
    let content = flatten(String::new(), node.document);
    print!(
        "@{} {}\n{}",
        status.account.acct,
        status.account.display_name,
        content
    );
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
