#![allow(unused_variables)]
#![allow(dead_code)]

mod repo;
use repo::Repo;

use std::env;
use clap::{Arg, App};
use std::collections::btree_map;



fn read_hub(filename: String) -> String{
	std::fs::read_to_string(filename).expect("Couldnt read file")
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
	
	let mut repos: Vec<Box<Repo>> = vec!();

	for repo in toml_de["packages"].as_table().unwrap(){
		let constructed_repo: Repo = repo.1.clone().try_into().unwrap();
		let repo_box = Box::new(constructed_repo);
		repos.push(repo_box);
	}

	let mut to_toml: btree_map::BTreeMap<String, Repo> = btree_map::BTreeMap::new();

	for repo in &mut repos{
		println!("{} {} {} {}", &repo.user, &repo.repo, &repo.arch, &repo.file_type);
		repo.populate();
		if repo.get_package {
			print!("Getting package ");
			repo.download_package();
		}
		if repo.get_source{
			println!("Getting source");
			repo.download_source();
		}
		to_toml.insert(repo.repo.clone(), *repo.clone());
	}
	// println!("{}", toml::to_string_pretty(&to_toml).unwrap());
	return ();

}
