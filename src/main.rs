#![allow(unused_variables)]
#![allow(dead_code)]

mod repo;
use repo::Repo;

use std::io::Write;
use std::env;
use clap::{Arg, App};
use std::collections::btree_map;



fn read_hub(filename: String) -> String{
	std::fs::read_to_string(filename).expect("Couldnt read hub file")
}



fn main(){
	let m = App::new(clap::crate_name!())
		.version(clap::crate_version!())
		.author(clap::crate_authors!())
		.arg(Arg::with_name("hub_file")
			.long("file")
			.short("f")
			.help("File containing packages")
			.required(false)
			.takes_value(true)
		)
		.get_matches();
	
	println!("Package file: {}", m.value_of("hub_file").unwrap_or("hub.toml"));

	let toml_string: String = read_hub(m.value_of("hub_file").unwrap_or("hub.toml").to_string());
	let toml_de: toml::Value = toml::from_str(toml_string.as_str()).unwrap();
	
	let lock_string: String = std::fs::read_to_string("hub.lock").unwrap_or("".to_string());
	let lock_de: toml::Value = toml::from_str(lock_string.as_str()).unwrap();

	let mut repos: Vec<Box<Repo>> = vec!();

	for repo in toml_de["packages"].as_table().unwrap(){
		let mut constructed_repo: Repo = repo.1.clone().try_into().unwrap();
		constructed_repo.populate();
		let repo_box = Box::new(constructed_repo);
		repos.push(repo_box);
	}


	let mut to_toml: btree_map::BTreeMap<String, Repo> = btree_map::BTreeMap::new();

	for repo in &mut repos{
		println!("{} {} {} {} {}", &repo.user, &repo.repo, &repo.arch, &repo.file_type, &repo.version);

		let ver: String = lock_de.as_table()
			.and_then(|n| n.get(&repo.repo))
			.and_then(|n| n.get("version"))
			.and_then(|n| n.as_str()).unwrap_or("").to_string();
		if ver != repo.version{
			if repo.get_package {
				print!("Getting package ");
				repo.download_package();
			}
			if repo.get_source{
				println!("Getting source");
				repo.download_source();
			}
		}else{
			println!("Already up to date");
		}

		to_toml.insert(repo.repo.clone(), *repo.clone());
	}
	let lock_string = toml::to_string_pretty(&to_toml).unwrap();

	let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open("hub.lock").expect("fail to touch");
	f.write_all(lock_string.as_bytes()).unwrap();	
	return ();

}
