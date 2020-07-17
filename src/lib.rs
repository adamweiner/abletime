//! [abletime](https://github.com/adamweiner/abletime) is a utility for calculating time spent on projects by
//! inspecting creation and modification timestamps of project files in a particular directory.

extern crate chrono;

use chrono::prelude::{DateTime, Local};
use chrono::Duration;
use regex::Regex;
use semver::Version;

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;

// https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
const SEMVER_REGEX: &str = r"(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?";

/// Represents a version of a project and all its relevant metadata.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProjectFile {
    pub created_datetime: DateTime<Local>,
    pub modified_datetime: DateTime<Local>,
    pub time_spent: Duration,
    pub name: String,
    pub version: Option<Version>,
}

impl fmt::Display for ProjectFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{: <21} {: <13} {}",
            self.created_datetime.format("%a %b %e %T"),
            format_duration(&self.time_spent),
            self.name,
        )
    }
}

/// Build and return a vector of ProjectFiles in some directory, sorted by creation timestamp.
fn initialize_project_files(directory: String, project_file_suffix: String) -> Result<Vec<ProjectFile>, io::Error> {
    let semver_regex: Regex = Regex::new(SEMVER_REGEX).unwrap();
    let mut project_files: Vec<ProjectFile> = Vec::new();

    // initialize project_files with all valid files found in provided directory
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let is_file = path.is_file();
        let is_valid_filetype = str::ends_with(path.to_str().unwrap(), &project_file_suffix);
        if is_file && is_valid_filetype {
            let name = path.file_name().and_then(OsStr::to_str).unwrap();
            // extract semantic version from file name if one can be found
            let version: Option<Version> = match semver_regex.find(name) {
                Some(version) => Some(Version::parse(version.as_str()).unwrap()),
                None => None,
            };
            let project_file = ProjectFile {
                created_datetime: DateTime::<Local>::from(entry.metadata()?.created()?),
                modified_datetime: DateTime::<Local>::from(entry.metadata()?.modified()?),
                time_spent: Duration::zero(), // initialize with zero value, calculated after all files are initialized
                name: name.to_string(),
                version,
            };
            project_files.push(project_file);
        }
    }

    // sort by creation timestamp. even if all files are versioned, assume that versions are created in order and
    // that sorting by version would produce the same result
    project_files.sort();

    Ok(project_files)
}

/// Calculate time spent on the provided project files.
fn calculate_time_spent(project_files: &mut Vec<ProjectFile>, max_time_between_saves: Duration) {
    for i in 0..project_files.len() {
        // start with delta between modified time and creation time
        let time_spent_single_file = project_files[i].modified_datetime - project_files[i].created_datetime;
        if time_spent_single_file < max_time_between_saves {
            project_files[i].time_spent = time_spent_single_file
        }

        // for all but the most recent project file, prefer the delta between creation time and the next version's
        // creation time unless a minor or major version boundary is detected
        if i < project_files.len() - 1
            && !is_session_boundary(&project_files[i].version, &project_files[i + 1].version, true)
        {
            let time_spent_before_next_version =
                project_files[i + 1].created_datetime - project_files[i].created_datetime;
            if time_spent_before_next_version < max_time_between_saves {
                project_files[i].time_spent = time_spent_before_next_version
            }
        }
    }
}

/// Check two (optional) versions to see if they represent a session boundary.
fn is_session_boundary(
    current_version: &Option<Version>,
    next_version: &Option<Version>,
    require_versions: bool,
) -> bool {
    // the next project file is one minor or major version greater than the current
    if current_version.is_some()
        && next_version.is_some()
        && (next_version.as_ref().unwrap().minor > current_version.as_ref().unwrap().minor
            || next_version.as_ref().unwrap().major > current_version.as_ref().unwrap().major)
    {
        return true;
    // if both project files must be versioned, we're done here
    } else if require_versions {
        return false;
    // one project file is versioned, and the other is not
    } else if (current_version.is_some() && next_version.is_none())
        || (next_version.is_some() && current_version.is_none())
    {
        return true;
    }
    false
}

/// Format durations as hh:mm:ss.ms.
fn format_duration(original_duration: &Duration) -> String {
    let mut duration = *original_duration;
    let hours = duration.num_hours();
    duration = duration - Duration::hours(hours);
    let minutes = duration.num_minutes();
    duration = duration - Duration::minutes(minutes);
    let seconds = duration.num_seconds();
    duration = duration - Duration::seconds(seconds);
    let milliseconds = duration.num_milliseconds();

    format!("{}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
}

/// Return the sum of time spent on all provided project files.
fn sum_project_durations(project_files: &[ProjectFile]) -> Duration {
    let mut total_duration: Duration = Duration::zero();
    for project_file in project_files {
        total_duration = total_duration + project_file.time_spent;
    }
    total_duration
}

/// Print to stdout the time spent on each project file, as well as a summary for the session (minor version)
/// if applicable.
fn print_session_summary(project_files: &[ProjectFile]) {
    if project_files.is_empty() {
        return;
    }

    if project_files[0].version.is_some() {
        println!(
            "Version {}.{}.x - {}",
            project_files[0].version.as_ref().unwrap().major,
            project_files[0].version.as_ref().unwrap().minor,
            format_duration(&sum_project_durations(project_files))
        );
    }
    for project_file in project_files {
        println!("{}", project_file);
    }
    println!() // extra newline for readability
}

/// Find all project files in the given directory and calculate time spent on each.
pub fn scan_project_files(
    directory: String,
    project_file_suffix: String,
    max_minutes_between_saves: i64,
) -> Result<Vec<ProjectFile>, io::Error> {
    let mut project_files: Vec<ProjectFile> = initialize_project_files(directory, project_file_suffix)?;

    // if max_minutes_between_saves is <= 0, effectively disable the max time check by using Duration's max value
    let max_time_between_saves: Duration = if max_minutes_between_saves > 0 {
        Duration::minutes(max_minutes_between_saves)
    } else {
        Duration::max_value()
    };

    calculate_time_spent(&mut project_files, max_time_between_saves);

    Ok(project_files)
}

/// Print to stdout a summary of time spent on each project file, time spent on each session/minor version
/// (if applicable), and total time spent on the project.
pub fn print_project_summary(project_files: &[ProjectFile]) {
    if project_files.is_empty() {
        println!("No project files found");
        return;
    }

    println!("{: <21} {: <13} Name", "Start time", "Duration");
    let mut current_version: &Option<Version>;
    let mut current_version_idx: usize = 0;
    for idx in 0..project_files.len() {
        current_version = &project_files[idx].version;
        if idx < project_files.len() - 1 {
            let next_version = &project_files[idx + 1].version;
            // print summary at session boundaries
            if is_session_boundary(current_version, next_version, false) {
                print_session_summary(&project_files[current_version_idx..idx + 1]);
                current_version_idx = idx + 1;
            }
        } else {
            // print last session summary
            print_session_summary(&project_files[current_version_idx..]);
        }
    }
    println!(
        "Total project time\n{}",
        format_duration(&sum_project_durations(project_files))
    );
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!("3:25:45.000", format_duration(&Duration::seconds(12345)));
        assert_eq!("0:05:21.000", format_duration(&Duration::seconds(321)));
        assert_eq!("0:00:00.522", format_duration(&Duration::milliseconds(522)));
    }

    #[test]
    fn test_calculate_time_spent() {
        let mut project_files: Vec<ProjectFile> = Vec::new();

        // single file: project time is modified - created
        let created_datetime_a = Local::now();
        let modified_datetime_a = created_datetime_a + Duration::seconds(1);
        let project_file_a = ProjectFile {
            created_datetime: created_datetime_a,
            modified_datetime: modified_datetime_a,
            time_spent: Duration::zero(),
            name: String::from("abletime 0.1.0.als"),
            version: Version::parse("0.1.0").ok(),
        };
        project_files.push(project_file_a);
        calculate_time_spent(&mut project_files, Duration::max_value());
        assert_eq!(project_files[0].time_spent, modified_datetime_a - created_datetime_a);

        // multiple files: project time is (file b created - file a created) + (file b modified - file b created)
        let created_datetime_b = Local::now();
        let modified_datetime_b = created_datetime_b + Duration::seconds(1);
        let project_file_b = ProjectFile {
            created_datetime: created_datetime_b,
            modified_datetime: modified_datetime_b,
            time_spent: Duration::zero(),
            name: String::from("abletime 0.1.1.als"),
            version: Version::parse("0.1.1").ok(),
        };
        project_files.push(project_file_b);
        calculate_time_spent(&mut project_files, Duration::max_value());
        assert_eq!(project_files[0].time_spent, created_datetime_b - created_datetime_a);
        assert_eq!(project_files[1].time_spent, modified_datetime_b - created_datetime_b);
    }

    #[test]
    fn test_sum_project_durations() {
        let mut project_files: Vec<ProjectFile> = Vec::new();
        assert_eq!(sum_project_durations(&project_files), Duration::zero());

        let project_file_a = ProjectFile {
            created_datetime: Local::now(),
            modified_datetime: Local::now(),
            time_spent: Duration::seconds(1),
            name: String::from("abletime 0.1.0.als"),
            version: Version::parse("0.1.0").ok(),
        };
        project_files.push(project_file_a);
        let project_file_b = ProjectFile {
            created_datetime: Local::now(),
            modified_datetime: Local::now(),
            time_spent: Duration::seconds(10),
            name: String::from("abletime 0.1.1.als"),
            version: Version::parse("0.1.1").ok(),
        };
        project_files.push(project_file_b);
        assert_eq!(sum_project_durations(&project_files), Duration::seconds(11));
    }

    #[test]
    fn test_is_session_boundary() {
        // session boundary: the next project file is one minor version greater than the current
        let mut current_version: Option<Version> = Version::parse("0.1.0").ok();
        let mut next_version: Option<Version> = Version::parse("0.2.0").ok();
        assert_eq!(is_session_boundary(&current_version, &next_version, false), true);
        assert_eq!(is_session_boundary(&current_version, &next_version, true), true);

        // session boundary: the next project file is one major version greater than the current
        next_version = Version::parse("1.0.0").ok();
        assert_eq!(is_session_boundary(&current_version, &next_version, false), true);
        assert_eq!(is_session_boundary(&current_version, &next_version, true), true);

        // not a session boundary: the next project file is one patch version greater than the current
        next_version = Version::parse("0.1.1").ok();
        assert_eq!(is_session_boundary(&current_version, &next_version, false), false);
        assert_eq!(is_session_boundary(&current_version, &next_version, true), false);

        // session boundary: current version is some, next version is none
        current_version = None;
        assert_eq!(is_session_boundary(&current_version, &next_version, false), true);

        // not a session boundary: current version is some, next version is none, but versions are required
        assert_eq!(is_session_boundary(&current_version, &next_version, true), false);

        // session boundary: current version is some, next version is none
        current_version = Version::parse("0.1.0").ok();
        next_version = None;
        assert_eq!(is_session_boundary(&current_version, &next_version, false), true);

        // not a session boundary: current version is none, next version is some, but versions are required
        current_version = None;
        next_version = Version::parse("0.1.0").ok();
        assert_eq!(is_session_boundary(&current_version, &next_version, true), false);

        // not a session boundary: both versions are none
        current_version = None;
        next_version = None;
        assert_eq!(is_session_boundary(&current_version, &next_version, true), false);
        assert_eq!(is_session_boundary(&current_version, &next_version, false), false);
    }
}
