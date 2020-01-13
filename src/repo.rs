use regex::Regex;
use serde::{Serialize, Deserialize};
use indicatif::{ProgressBar, ProgressStyle};

struct DownloadProgress {
    inner: Option<reqwest::blocking::Response>,
    progress_bar: ProgressBar,
}

impl DownloadProgress{
    fn new_with_response(resp: reqwest::blocking::Response) -> DownloadProgress{
        let total_size = resp.headers().get(reqwest::header::CONTENT_LENGTH)
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|ct_len| ct_len.parse().ok())
            .unwrap_or(0);
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.white/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("=> "));

        DownloadProgress{
            progress_bar: pb,
            inner: Some(resp),
        }
    }
}


impl std::io::Read for DownloadProgress{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.inner{
            Some(x) => {
                x.read(buf).map(|n| {
                    self.progress_bar.inc(n as u64);
                    n
                })
            },
            None => {
                panic!("wot");
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Repo{
	pub user: String,
	pub repo: String,
	pub arch: String,
	pub file_type: String,
	pub package_name: String,
	pub get_source: bool,
	pub get_package: bool,
	pub version: String,
	#[serde(skip_serializing)]
	pub parsed_response: serde_json::Value,
}

impl Default for Repo{
	fn default() -> Repo{
		Repo{
			user: "".to_string(),
			repo: "".to_string(),
			arch: "".to_string(),
			file_type: "".to_string(),
			package_name: "".to_string(),
			get_source: false,
			get_package: true,
			version: "".to_string(),
			parsed_response: serde_json::json!(null),
		}
	}
}

impl Repo{
	pub fn populate(&mut self){
		self.parsed_response = self.get_json();
		self.version = self.parsed_response["tag_name"].as_str().unwrap().to_string();
	}

	pub fn get_json(&self) -> serde_json::Value{
		let client = reqwest::blocking::Client::new();
		let addr = format!("https://api.github.com/repos/{}/{}/releases/latest", self.user, self.repo);
		let res = client.get(&addr).header(reqwest::header::USER_AGENT, "foo").send().unwrap().text().unwrap();
		serde_json::from_str(&res).unwrap()
	}
	
	
	
	pub fn get_package_url(&mut self) -> Result<String, String> {
		let re = Regex::new(format!(".*{}.*\\.{}$", self.arch.as_str() , self.file_type).as_str()).unwrap();
		let mut package_matches: Vec<String> = vec!();
		for asset in self.parsed_response["assets"].as_array().unwrap(){
			let name = asset["name"].as_str().unwrap();
			if re.is_match(name){
				self.package_name = name.to_string();
				package_matches.push(asset["browser_download_url"].as_str().unwrap().to_string());
			}
		}
		match package_matches.len() {
			0 => Err("No Matches found".to_string()),
			1 => Ok(package_matches[0].clone()),
			_ => Err("Multiple matches".into())
		}
	}
	
	
	
	pub fn download_source(&mut self){
		let url = self.parsed_response["zipball_url"].as_str().unwrap().to_string();
	
		let client = reqwest::blocking::Client::new();
		let resp = client.get(&url).header(reqwest::header::USER_AGENT, "foo").send().unwrap();
    
        let mut pb = DownloadProgress::new_with_response(resp);

		match std::fs::create_dir("source"){
			Ok(_x) => (),
			Err(_x) => (),
		};
        let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(format!("source/{}", self.repo.to_string())).expect("fail to touch");
        
		std::io::copy(&mut pb, &mut f).expect("copy fail");
	}

	pub fn download_package(&mut self){
		let url = match self.get_package_url(){
			Ok(x) => x,
			Err(x) => {
				println!("{}", x);
				return;
			},
		};
	
		println!("{}", self.package_name.clone());
        let resp = reqwest::blocking::get(url.as_str()).expect("failed to get response");

        let mut pb = DownloadProgress::new_with_response(resp);

		match std::fs::create_dir("downloads"){
			Ok(_x) => (),
			Err(_x) => (),
		};
        let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(format!("downloads/{}", self.package_name.to_string())).expect("fail to touch");
        
		std::io::copy(&mut pb, &mut f).expect("copy fail");
	}
	
}