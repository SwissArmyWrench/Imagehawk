use std::process::Command; // Used to run Docker commands on the host system

use serde_json::Value; // Used to parse output JSON from Docker/DockerHub to find image hashes

/// This struct contains a Rust-ified version of a Docker image name.
/// Names without a tag are valid, such as pihole/pihole, so the `tag` field is an Option.
/// Names without a repo are (in some cases) valid, such as the Postgres database,
/// which is hosted under `postgres:13` or whatever, without a repo name, so the repo is
/// also tracked as an Option.
/// The image might be hosted on a number of places other than DockerHub, so this is
/// specified in the `remote` field which takes a Remote enum.
struct Image<'img> {
    namespace: &'img str,
    repo: Option<&'img str>,
    tag: Option<&'img str>,
    remote: Remote,
}

/// This enum simply denotes what container registry an image is sourced from.
/// At the moment only DockerHub is supported.
enum Remote {
    DockerHub,
    Ghcr,
    Lscr,
    Quay,
}

impl Image<'_> {
    /// from_str() parses a plaintext image name into an Image struct.
    /// TODO: implement regex matching to simplify all of this code
    fn from_str(namestring: &str) -> Image {
        let parts: Vec<&str> = namestring.split('/').collect(); // Separate by slashes

        // if 2 slashes (and thus 3 parts), this means a URL to another registry was included
        // Otherwise, only 1 slash means it's hosted on DockerHub
        if parts.len() == 3 {
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
            let repo: Option<&str>; // Init repo variable
            let tag: Option<&str>; // init tag variable

            // If the last portion of the name contains a colon, there is a tag
            if parts[2].contains(':') {
                // has tag
                let parts: Vec<&str> = parts[2].split(':').collect(); // split around the colon
                repo = Some(parts[0]); // assign repo
                tag = Some(parts[1]); // assign tag
            } else {
                // otherwise there is no tag and the tag should be filled with a None variant
                repo = Some(parts[0]);
                tag = None;
            }

            // Assemble struct instance and return it
            return Image {
                namespace,
                repo,
                tag,
                remote,
            };

        // if no external repo is specified repeat 
        // most of the above code but hard code
        // the remote to DockerHub
        } else if parts.len() == 2 {
            let namespace = parts[0];
            let parts: Vec<&str> = parts[1].split(':').collect();
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
        // Handle the extra empty strings that end up in the
        // output by simply ignoring them
        } else if (parts.len() == 1) && (parts[0].is_empty()) {
            // do nothing
        // Handle the cases like postgres:13 with a None enum
        } else if parts.len() == 1 {
            let parts: Vec<&str> = parts[0].split(':').collect();
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


        // if nothing has happened as this point return an error struct
        // and complain about it on stderr
        eprintln!("NOTHING RETURNED YET\n{:?}", parts);
        Image {
            namespace: "error",
            repo: Some("error"),
            tag: None,
            remote: Remote::DockerHub,
        }
    }

    /// This function returns the image as a docker name string.
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
    // Working with iterator to remove the quotation marks at each end
    let mut output_iter: Vec<&str> = output.split('\n').collect();
    let _ = output_iter.remove(output_iter.len() - 1);
    let output_iter = output_iter.iter().map(|x| x.trim_matches('"'));
    let output_iter = output_iter.map(|string| Image::from_str(string));
    let images: Vec<Image> = output_iter.collect();
    
    let mut warnings = Vec::<Image>::new();
    for image in images {
        // API calls and comparisons here
        let response = reqwest::blocking::get(format!(
            "https://hub.docker.com/v2/namespaces/{0}/repositories/{1}/tags/{2}",
            &image.namespace,
            &image.repo.unwrap_or("error"),
            &image.tag.unwrap_or("latest"),
        ));
        let status = response.as_ref().unwrap().status();
        let api_supported = match image.remote {
            Remote::DockerHub => true,
            Remote::Quay => false,
            Remote::Lscr => false,
            Remote::Ghcr => false,
        };
        if !(status.as_str() == "200" && api_supported) {
            continue;
        }
        let json = response.unwrap().text().unwrap();
        let parsed_json: Value = serde_json::from_str(&json).expect("unable to parse JSON");
        let digest = parsed_json.get("digest");
        let digest = digest.unwrap(); // .expect("tried to unwrap a None");
        let remote_hash = digest.as_str().unwrap();

        let local_image = Command::new("docker")
            .arg("image")
            .arg("inspect")
            .arg(image.dump())
            .output()
            .unwrap()
            .stdout;
        let local_image_json: Value =
            serde_json::from_str(&String::from_utf8(local_image).unwrap()).unwrap();
        let local_image_hash = local_image_json
            .get(0)
            .unwrap()
            .get("RepoDigests")
            .unwrap()
            .get(0)
            .unwrap()
            .as_str()
            .unwrap();

        // Trim hashes
        let remote_hash = remote_hash.split(':').collect::<Vec<&str>>()[1];
        let local_image_hash = local_image_hash.split(':').collect::<Vec<&str>>()[1];

        if remote_hash != local_image_hash {
            warnings.push(image);
        } else {
            // do nothing 
        }
    }
    println!("{} images may need updating:", &warnings.len());
    for image in warnings {
        println!("{}", image.dump());
    }

}
