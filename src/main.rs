use std::process::Command;

struct Image<'img> {
    namespace: &'img str,
    repo: Option<&'img str>,
    tag: Option<&'img str>,
    remote: Remote,
}

enum Remote {
    DockerHub,
    Ghcr,
    Lscr,
    Quay,
}

impl Image<'_> {
    fn from_str(namestring: &str) -> Image {
        let parts: Vec<&str> = namestring.split("/").collect();

        // println!("{:?}", &parts);

        if parts.len() == 3 {
            println!("EXTERNAL. Not yet supported, substituting w/ Pihole image\n");
            let namespace = parts[1];
            let remote = if parts[0] == "lscr.io" {
                Remote::Lscr
            } else if parts[0] == "ghcr.io" {
                Remote::Ghcr
            } else if parts[0] == "quay.io" {
                Remote::Quay
            } else {
                Remote::DockerHub
            };
            let repo: Option<&str>;
            let tag: Option<&str>;
            if parts[2].contains(":") {
                // has tag
                let parts: Vec<&str> = parts[2].split(":").collect();
                repo = Some(parts[0]);
                tag = Some(parts[1]);

            } else {
                repo = Some(parts[0]);
                tag = None;
            }

            return Image {
                namespace,
                repo, 
                tag,
                remote,
            };
        } else if parts.len() == 2 {
            println!("NS AND REPO\n");
            let namespace = parts[0];
            let parts: Vec<&str> = parts[1].split(":").collect();
            let repo = parts[0];
            let tag: Option<&str> = if parts.len() == 2 {
                Some(parts[1])
            } else {
                None
            };

            return Image {
                namespace,
                repo: Some(repo),
                tag,
                remote: Remote::DockerHub,
            };
        } else if (parts.len() == 1) && (parts[0].is_empty()) {
            // do nothing
        } else if parts.len() == 1 {
            println!("NO REPO\n");
            let parts: Vec<&str> = parts[0].split(":").collect();
            let namespace = parts[0];
            let repo = None;
            let tag = Some(parts[1]);

            return Image {
                namespace,
                repo,
                tag,
                remote: Remote::DockerHub,
            };
        };

        // println!("{:?}", &parts);

        println!("NOTHING RETURNED YET\n{:?}", parts);
        Image {
            namespace: "error",
            repo: Some("error"),
            tag: None,
            remote: Remote::DockerHub,
        }
    }

    fn print(&self) {
        let parsed_tag = self.tag.unwrap_or("NO TAG");
        println!(
            "Namespace: {}\nRepo: {}\nTag: {}\nRemote: {}\n",
            self.namespace,
            self.repo.unwrap_or("NO REPO GIVEN"),
            parsed_tag,
            match self.remote {
                Remote::DockerHub => "DockerHub",
                Remote::Quay => "Quay.io",
                Remote::Lscr => "LSCR.io",
                Remote::Ghcr => "GHCR.io",
            }

        );
    }

    fn dump(&self) -> String {
        let repo_and_tag = match (self.repo, self.tag) {
            (Some(repo), Some(tag)) => format!("/{repo}:{tag}"),
            (Some(repo), None) => format!("/{repo}"),
            (None, Some(tag)) => format!(":{tag}"),
            (None, None) => String::from(""),
        };

        let remote_and_name = match self.remote {
            Remote::DockerHub => format!("{}", self.namespace),
            Remote::Quay => format!("quay.io/{}", self.namespace),
            Remote::Lscr => format!("lscr.io/{}", self.namespace),
            Remote::Ghcr => format!("ghcr.io/{}", self.namespace),
        };
        format!("{remote_and_name}{repo_and_tag}")

    }
       
    
}

fn main() {
    // let mut dockerps = Command::new("docker");
    // let mut dockerps = dockerps.arg("ps");
    let output = String::from_utf8(
        Command::new("docker")
            .arg("ps")
            .arg("--format")
            .arg("\"{{ .Image }}\"")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    // println!("{}", &output);
    // Working with iterator to remove the quotation marks at each end
    let mut output_iter: Vec<&str> = output.split('\n').collect();
    let _ = output_iter.remove(output_iter.len() -1);
    let output_iter = output_iter.iter().map(|x| x.trim_matches('"'));
    let output_iter = output_iter.map(|string| Image::from_str(string));
    // println!("{:?}", &output_iter);
    let mut images: Vec<Image> = output_iter.collect();
    // images.remove(images.len() - 1);
    // images.retain(|img| img.tag == Some("latest"));
    for image in images {
        println!("{}", image.dump());
        image.print();
        // API calls and comparisons here
    }

    //let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    // println!("{}", parsed[0]["Image"]);
}
