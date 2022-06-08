//! bin2json is a utility to run different kind of binary parser compatible with the rustruct library
//! then output the attributes and data tree they generates to json

use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Read};
use std::collections::HashMap;

use tap::session::Session;
use tap::node::Node;
use tap_plugin_magic::{datatypes, plugins_datatype};

use serde_derive::Deserialize;
use serde_json::json;
use log::warn;
use clap::{crate_authors, crate_description, crate_name, crate_version, Arg, App, AppSettings};

#[derive(Deserialize,Clone)]
struct Config 
{
  plugins_types : HashMap<String, Vec<String> >,
}

enum Cmd
{
  Device(String), 
  File(String),
  Plugins,
}

struct Arguments
{
  cmd : Cmd,
  config : Config,
  output : Option<String>,
}

fn usage() -> Arguments
{
  let matches = App::new(crate_name!())
    .version(crate_version!())
    .author(crate_authors!())
    .about(crate_description!())
    .setting(AppSettings::ArgRequiredElseHelp)
    .arg(Arg::with_name("file")
        .short("f")
        .long("file")
        .value_name("FILE")
        .conflicts_with("DEVICE")
        .help("Path to the files to parse")
        .takes_value(true))
    .arg(Arg::with_name("output")
        .short("o")
        .long("output")
        .value_name("OUTPUT")
        .help("Output file")
        .takes_value(true))
    .arg(Arg::with_name("device")
        .short("d")
        .long("device")
        .value_name("DEVICE")
        .help("Path to a device to parse")
        .conflicts_with("PATH")
        .takes_value(true)) 
    .arg(Arg::with_name("config")
        .short("c")
        .long("config")
        .value_name("FILE")
        .help("Config file path")
        .takes_value(true))
    .arg(Arg::with_name("plugins")
        .short("v")
        .long("plugins")
        .conflicts_with("PATH")
        .conflicts_with("DEVICE")
        .help("List embedded plugins")
        .takes_value(false))
    .get_matches();

  let config_file = matches.value_of("config").unwrap_or("bin2json.toml");
  let config = File::open(config_file)
    .and_then(|mut file| 
    {
      let mut buffer = String::new();
      file.read_to_string(&mut buffer)?;
      Ok(buffer)
    })
    .and_then(|buffer| 
       toml::from_str::<Config>(&buffer)
       .map_err(|err| io::Error::new(io::ErrorKind::Other, err)))
    .map_err(|err| warn!("Can't read config file: {}", err))
    .ok().unwrap();

  let output = matches.value_of("output").map(|value| value.to_string());

  if matches.is_present("file") {
    Arguments{ cmd : Cmd::File(matches.value_of("file").unwrap().to_string()), config, output}
  }
  else if matches.is_present("device") {
    Arguments{ cmd : Cmd::Device(matches.value_of("device").unwrap().to_string()), config, output}
  }
  else if matches.is_present("plugins") {
    Arguments{ cmd : Cmd::Plugins, config, output}
  }
  else {
    panic!()
  }
}


/// it must be used only for test or in a sandboxed env
fn register_plugins(session :&mut Session)
{
  session.plugins_db.register(Box::new(tap_plugin_local::Plugin::new())); 
  session.plugins_db.register(Box::new(tap_plugin_exif::Plugin::new())); 
  session.plugins_db.register(Box::new(tap_plugin_ntfs::Plugin::new()));
  session.plugins_db.register(Box::new(tap_plugin_mft::Plugin::new())); 
  session.plugins_db.register(Box::new(tap_plugin_prefetch::Plugin::new()));
  session.plugins_db.register(Box::new(tap_plugin_partition::Plugin::new())); 
  session.plugins_db.register(Box::new(tap_plugin_lnk::Plugin::new()));
  session.plugins_db.register(Box::new(tap_plugin_evtx::Plugin::new()));
  session.plugins_db.register(Box::new(tap_plugin_registry::Plugin::new()));
  #[cfg(feature = "device")]
  session.plugins_db.register(Box::new(tap_plugin_device::Plugin::new()));
}

fn display_plugins_db(session : &Session)
{
  for plugin in session.plugins_db.iter()
  {
    if plugin.name() != "local" 
    { eprintln!("\t\t{} : {}", plugin.name(), plugin.help()); }
  }
}

fn main() 
{
    pretty_env_logger::init();
 
    let arguments = usage();
    let plugins_types = arguments.config.plugins_types;
    let mut session = Session::new();
    //we register the plugin we need;
    register_plugins(&mut session); 

    let (plugin_name, plugin_argument) = match arguments.cmd
    {
      Cmd::File(path) => ("local", json!({"mount_point": session.tree.root_id, "files" : [path] }).to_string()),
      Cmd::Device(path) => ("device", json!({"mount_point": session.tree.root_id, "path" : path}).to_string()),
      Cmd::Plugins => { display_plugins_db(&session); return ;}
    };

    session.run(plugin_name, plugin_argument, false).unwrap();

    //scan recursively 
    while !datatypes(&session.tree).is_empty()
    {
      warn!("plugins list to match {:?}", plugins_types);
      let plugins = plugins_datatype(&session.tree, &plugins_types);

      warn!("plugins {:?}", plugins);
      for (node_id, plugin_name) in plugins
      {
        let arguments = match plugin_name.as_str()
        {
          "exif" => json!({"files" : [node_id]}).to_string(),
          "prefetch" | "evtx" | "lnk" | "registry" => json!({"file" : node_id }).to_string(),
          _ => json!({"file" : node_id}).to_string(),
        };
        warn!("scheduling plugin {} args {}", &plugin_name, arguments);
        let _err = session.schedule(&plugin_name, arguments, false);
      }  
      session.join();
    }
 
    let mut file = arguments.output.map(|file_name| File::create(file_name).unwrap());

    let nodes = session.tree.children_rec(None).unwrap();

    for node_id in nodes
    {
       let node = session.tree.get_node_from_id(node_id).unwrap();
       let path = session.tree.node_path(node_id);
       let json = json!({"name" : node.name(), "path" : path, "attributes" : &node as &Node}).to_string() + "\n";

       match file 
       {
         Some(ref mut file) => file.write_all(json.as_bytes()).unwrap(),
         None => println!("{}", json),
       }
    }
}
