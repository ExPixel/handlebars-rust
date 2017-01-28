#![allow(unused_imports, dead_code)]
extern crate env_logger;
extern crate handlebars;
#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
extern crate rustc_serialize;

#[cfg(feature = "unstable")]
extern crate serde;
#[cfg(feature = "serde_type")]
extern crate serde_json;
#[cfg(feature = "unstable")]
#[macro_use]
extern crate serde_derive;

// default feature, using rustc_serialize `Json` as data type
#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
use rustc_serialize::json::{Json, ToJson};
// serde_type feature, using serde_json as data type
#[cfg(feature = "serde_type")]
use serde_json::value::{self, Value as Json, ToJson, Map};

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
use std::collections::BTreeMap as Map;

use std::error::Error;

use handlebars::{Handlebars, RenderError, RenderContext, Helper, Context, JsonRender, Decorator,
                 to_json};

// default format helper
fn format_helper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    // get parameter from helper or throw an error
    let param = try!(h.param(0).ok_or(RenderError::new("Param 0 is required for format helper.")));
    let rendered = format!("{} pts", param.value().render());
    try!(rc.writer.write(rendered.into_bytes().as_ref()));
    Ok(())
}

fn format_decorator(d: &Decorator,
                    _: &Handlebars,
                    rc: &mut RenderContext)
                    -> Result<(), RenderError> {
    let suffix = d.param(0).map(|v| v.value().render()).unwrap_or("".to_owned());
    rc.register_local_helper("format", Box::new(move |h: &Helper, _: &Handlebars, rc: &mut RenderContext|{
        // get parameter from helper or throw an error
        let param = try!(h.param(0).ok_or(RenderError::new("Param 0 is required for format helper.")));
        let rendered = format!("{} {}", param.value().render(), suffix);
        try!(rc.writer.write(rendered.into_bytes().as_ref()));
        Ok(())
    }));
    Ok(())
}

// another custom helper
fn rank_helper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let rank = try!(h.param(0)
                    .and_then(|v| v.value().as_u64())
                    .ok_or(RenderError::new("Param 0 with u64 type is required for rank helper."))) as usize;
    let teams = try!(h.param(1)
              .and_then(|v| v.value().as_array())
              .ok_or(RenderError::new("Param 1 with array type is required for rank helper")));
    let total = teams.len();
    if rank == 0 {
        try!(rc.writer.write("champion".as_bytes()));
    } else if rank >= total - 2 {
        try!(rc.writer.write("relegation".as_bytes()));
    } else if rank <= 2 {
        try!(rc.writer.write("acl".as_bytes()));
    }
    Ok(())
}

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
static TYPES: &'static str = "rustc_serialize";
#[cfg(feature = "serde_type")]
static TYPES: &'static str = "serde_json";

// define some data
#[cfg_attr(all(feature = "serde_type"), derive(Serialize))]
pub struct Team {
    name: String,
    pts: u16,
}

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
impl ToJson for Team {
    fn to_json(&self) -> Json {
        let mut m: Map<String, Json> = Map::new();
        m.insert("name".to_string(), self.name.to_json());
        m.insert("pts".to_string(), self.pts.to_json());
        m.to_json()
    }
}

// produce some data
pub fn make_data() -> Map<String, Json> {
    let mut data = Map::new();

    data.insert("year".to_string(), to_json(&"2015".to_owned()));

    let teams = vec![Team {
                         name: "Jiangsu Suning".to_string(),
                         pts: 43u16,
                     },
                     Team {
                         name: "Shanghai SIPG".to_string(),
                         pts: 39u16,
                     },
                     Team {
                         name: "Hebei CFFC".to_string(),
                         pts: 27u16,
                     },
                     Team {
                         name: "Guangzhou Evergrand".to_string(),
                         pts: 22u16,
                     },
                     Team {
                         name: "Shandong Luneng".to_string(),
                         pts: 12u16,
                     },
                     Team {
                         name: "Beijing Guoan".to_string(),
                         pts: 7u16,
                     },
                     Team {
                         name: "Hangzhou Greentown".to_string(),
                         pts: 7u16,
                     },
                     Team {
                         name: "Shanghai Shenhua".to_string(),
                         pts: 4u16,
                     }];

    data.insert("teams".to_string(), to_json(&teams));
    data.insert("engine".to_string(), to_json(&TYPES.to_owned()));
    data
}

fn main() {
    env_logger::init().unwrap();
    // create the handlebars registry
    let mut handlebars = Handlebars::new();

    // register template from a file and assign a name to it
    // deal with errors
    if let Err(e) = handlebars.register_template_file("table",
                                                      "./examples/decorator/template.hbs") {
        panic!("{}", e);
    }

    // register some custom helpers
    handlebars.register_helper("format", Box::new(format_helper));
    handlebars.register_helper("ranking_label", Box::new(rank_helper));
    handlebars.register_decorator("format_suffix", Box::new(format_decorator));

    // make data and render it
    let data = make_data();
    println!("{}",
             handlebars.render("table", &data).unwrap_or_else(|e| format!("{}", e)));
}
