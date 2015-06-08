// Copyright 2015 MaidSafe.net limited.
//
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.
#![allow(unused_variables)]

use cbor::{Decoder, Encoder, CborError};
use rand;
use rustc_serialize::{Decodable, Encodable};
use sodiumoxide;
use std::collections::{BTreeMap, HashMap};
use std::sync::mpsc;
use std::boxed::Box;
use std::sync::mpsc::Receiver;
use time::{Duration, SteadyTime};

use crust;
use lru_time_cache::LruCache;
use message_filter::MessageFilter;
use NameType;
use node_interface::{Interface, CreatePersonas};
use routing_table::{RoutingTable};
use relay::RelayMap;
use routing_membrane::RoutingMembrane;
use types;
use types::{MessageId, Bytes};
use error::{RoutingError};
use std::thread::spawn;

type ConnectionManager = crust::ConnectionManager;
type Event = crust::Event;
pub type Endpoint = crust::Endpoint;
type PortAndProtocol = crust::Port;

type RoutingResult = Result<(), RoutingError>;

/// DHT node
pub struct RoutingNode<G : CreatePersonas> {
    genesis: Box<G>,
    id: types::Id,
    own_name: NameType,
    event_input: Receiver<Event>,
    connection_manager: ConnectionManager,
    all_connections: (HashMap<Endpoint, NameType>, BTreeMap<NameType, Vec<Endpoint>>),
    routing_table: RoutingTable,
    relay_map: RelayMap,
    accepting_on: Vec<Endpoint>,
    next_message_id: MessageId,
    bootstrap_endpoint: Option<Endpoint>,
    bootstrap_node_id: Option<NameType>,
    filter: MessageFilter<types::FilterType>,
    public_id_cache: LruCache<NameType, types::PublicId>,
    connection_cache: BTreeMap<NameType, SteadyTime>
}

impl<G : CreatePersonas> RoutingNode<G> {
    pub fn new(genesis: G) -> RoutingNode<G> {
        sodiumoxide::init();  // enable shared global (i.e. safe to multithread now)
        let (event_output, event_input) = mpsc::channel();
        let id = types::Id::new();
        let own_name = id.get_name();
        let mut cm = crust::ConnectionManager::new(event_output);
        // TODO: Default Protocol and Port need to be passed down
        let ports_and_protocols : Vec<PortAndProtocol> = Vec::new();
        // TODO: Beacon port should be passed down
        let beacon_port = Some(5483u16);
        let listeners = match cm.start_listening2(ports_and_protocols, beacon_port) {
            Err(reason) => {
                println!("Failed to start listening: {:?}", reason);
                (vec![], None)
            }
            Ok(listeners_and_beacon) => listeners_and_beacon
        };
        println!("{:?}  -- listening on : {:?}", own_name, listeners.0);
        RoutingNode { genesis: Box::new(genesis),
                      id : id,
                      own_name : own_name.clone(),
                      event_input: event_input,
                      connection_manager: cm,
                      all_connections: (HashMap::new(), BTreeMap::new()),
                      routing_table : RoutingTable::new(&own_name),
                      relay_map: RelayMap::new(&own_name),
                      accepting_on: listeners.0,
                      next_message_id: rand::random::<MessageId>(),
                      bootstrap_endpoint: None,
                      bootstrap_node_id: None,
                      filter: MessageFilter::with_expiry_duration(Duration::minutes(20)),
                      public_id_cache: LruCache::with_expiry_duration(Duration::minutes(10)),
                      connection_cache: BTreeMap::new(),
                    }
    }

    /// run_membrane spawns a new thread and moves a newly constructed Membrane into this thread.
    /// Routing node uses the genesis object to create a new instance of the personas to embed
    /// inside the membrane.
    //  TODO: a (two-way) channel should be passed in to control the membrane.
    //        connection_manager should also be moved into the membrane;
    //        firstly moving most ownership of the constructor into this function.
    fn run_membrane<T: Interface + 'static>(&mut self) {
        let mut membrane = RoutingMembrane::<T>::new(self.genesis.create_personas());
        spawn(move || membrane.run());
    }
}

fn encode<T>(value: &T) -> Result<Bytes, CborError> where T: Encodable {
    let mut enc = Encoder::from_memory();
    try!(enc.encode(&[value]));
    Ok(enc.into_bytes())
}

fn decode<T>(bytes: &Bytes) -> Result<T, CborError> where T: Decodable {
    let mut dec = Decoder::from_bytes(&bytes[..]);
    match dec.decode().next() {
        Some(result) => result,
        None => Err(CborError::UnexpectedEOF)
    }
}

fn ignore<R,E>(_: Result<R,E>) {}

#[cfg(test)]
mod test {

}
