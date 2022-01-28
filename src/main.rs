use std::collections::HashMap;
use std::error::Error;

use git2::{Oid, Repository};
use regex::Regex;

fn main() {
    match calculate_version(".") {
        Ok(version) => println!("{}", version),
        Err(e) => println!("Failed to calulate version: {}", e),
    };
}

fn calculate_version(path: &str) -> Result<String, Box<dyn Error>> {
    let regex = Regex::new(r"(?P<major>\d+)(?:[.](?P<minor>\d+[.])(?:[.](?P<patch>\d+[.]))?)?")?;
    let repo = Repository::open(path)?;

    let mut version = Version {
        major: 0,
        minor: 0,
        patch: 0,
    };


    let tags: HashMap<Oid, String> = HashMap::from_iter(repo.tag_names(None)?.iter()
        .filter_map(|name| name)
        .filter_map(|name| {
            let path = format!("refs/tags/{}", name);
            let reference = repo.find_reference(path.as_str()).ok()?;
            let commit = reference.peel_to_commit().ok()?;

            Some((commit.id(), name.to_string()))
        })
    );


    let mut walk = repo.revwalk()?;
    walk.push_head()?;
    walk.set_sorting(git2::Sort::TIME | git2::Sort::REVERSE)?;

    for rev_option in walk {
        if let Ok(commit) = repo.find_commit(rev_option.unwrap()) {
            let tag_version = tags.get(&commit.id()).and_then(|name| regex.captures(name)).and_then(|capture| {
                Some(Version {
                    major: capture.name("major").map(|n| n.as_str()).and_then(|v| v.parse::<i32>().ok())?,
                    minor: capture.name("minor").map(|n| n.as_str()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0),
                    patch: capture.name("patch").map(|n| n.as_str()).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0),
                })
            });

            if let Some(value) = tag_version {
                version = value;
            } else {
                version = Version {
                    major: version.major,
                    minor: version.minor,
                    patch: version.patch + 1,
                }
            }
        }
    }

    return Ok(format!("{}.{}.{}", version.major, version.minor, version.patch));
}

struct Version {
    major: i32,
    minor: i32,
    patch: i32,
}