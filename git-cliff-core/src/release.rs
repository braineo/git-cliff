use crate::commit::Commit;
use crate::error::Result;
use next_version::NextVersion;
use semver::Version;
use serde::{
	Deserialize,
	Serialize,
};

/// Representation of a release.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release<'a> {
	/// Release version, git tag.
	pub version:   Option<String>,
	/// Commits made for the release.
	pub commits:   Vec<Commit<'a>>,
	/// Commit ID of the tag.
	#[serde(rename = "commit_id")]
	pub commit_id: Option<String>,
	/// Timestamp of the release in seconds, from epoch.
	pub timestamp: i64,
	/// Previous release.
	pub previous:  Option<Box<Release<'a>>>,
}

impl<'a> Release<'a> {
	/// Calculates the next version based on the commits.
	pub fn calculate_next_version(&self) -> Result<String> {
		match self
			.previous
			.as_ref()
			.and_then(|release| release.version.clone())
		{
			Some(version) => {
				let next_version = Version::parse(version.trim_start_matches('v'))?
					.next(
						self.commits
							.iter()
							.map(|commit| commit.message.trim_end().to_string())
							.collect::<Vec<String>>(),
					)
					.to_string();
				Ok(next_version)
			}
			None => {
				warn!("No releases found, using 0.0.1 as the next version.");
				Ok(String::from("0.0.1"))
			}
		}
	}
}

/// Representation of a list of releases.
#[derive(Serialize)]
pub struct Releases<'a> {
	/// Releases.
	pub releases: &'a Vec<Release<'a>>,
}

impl<'a> Releases<'a> {
	/// Returns the list of releases as JSON.
	pub fn as_json(&self) -> Result<String> {
		Ok(serde_json::to_string(self.releases)?)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn bump_version() -> Result<()> {
		for (expected_version, commits) in [
			("1.1.0", vec!["feat: add xyz", "fix: fix xyz"]),
			("1.0.1", vec!["fix: add xyz", "fix: aaaaaa"]),
			("2.0.0", vec!["feat!: add xyz", "feat: zzz"]),
			("2.0.0", vec!["feat!: add xyz\n", "feat: zzz\n"]),
		] {
			let release = Release {
				version:   None,
				commits:   commits
					.into_iter()
					.map(|v| Commit::from(v.to_string()))
					.collect(),
				commit_id: None,
				timestamp: 0,
				previous:  Some(Box::new(Release {
					version: Some(String::from("1.0.0")),
					..Default::default()
				})),
			};
			let next_version = release.calculate_next_version()?;
			assert_eq!(expected_version, next_version);
		}
		let empty_release = Release {
			previous: Some(Box::new(Release {
				version: None,
				..Default::default()
			})),
			..Default::default()
		};
		let next_version = empty_release.calculate_next_version()?;
		assert_eq!("0.0.1", next_version);
		Ok(())
	}
}
