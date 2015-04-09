// Copyright 2015 MaidSafe.net limited
// This MaidSafe Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
// By contributing code to the MaidSafe Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0, found in the root
// directory of this project at LICENSE, COPYING and CONTRIBUTOR respectively and also
// available at: http://www.maidsafe.net/licenses
// Unless required by applicable law or agreed to in writing, the MaidSafe Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS
// OF ANY KIND, either express or implied.
// See the Licences for the specific language governing permissions and limitations relating to
// use of the MaidSafe
// Software.

#![allow(unused_assignments)]

use cbor::CborTagEncode;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

use types;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FindGroupResponse {
  pub target_id : types::DhtId,
  pub group : Vec<types::PublicPmid>
}

impl FindGroupResponse {
    pub fn generate_random() -> FindGroupResponse {
        let mut vec: Vec<types::PublicPmid> = Vec::with_capacity(99);
        for i in 0..99 {
            vec.push(types::PublicPmid::generate_random());
        }

        FindGroupResponse {
            target_id: types::DhtId::generate_random(),
            group: vec,
        }
    }
}

impl Encodable for FindGroupResponse {
  fn encode<E: Encoder>(&self, e: &mut E)->Result<(), E::Error> {
    CborTagEncode::new(5483_001, &(&self.target_id, &self.group)).encode(e)
  }
}

impl Decodable for FindGroupResponse {
  fn decode<D: Decoder>(d: &mut D)->Result<FindGroupResponse, D::Error> {
    try!(d.read_u64());
    let (target_id, group) = try!(Decodable::decode(d));
    Ok(FindGroupResponse { target_id: target_id, group: group})
  }
}

#[cfg(test)]
mod test {
    extern crate cbor;

    use super::*;

    #[test]
    fn find_group_response_serialisation() {
        let obj_before = FindGroupResponse::generate_random();

        let mut e = cbor::Encoder::from_memory();
        e.encode(&[&obj_before]).unwrap();

        let mut d = cbor::Decoder::from_bytes(e.as_bytes());
        let obj_after: FindGroupResponse = d.decode().next().unwrap().unwrap();

        assert_eq!(obj_before, obj_after);
    }
}
