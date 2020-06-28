//! [abletime](https://github.com/adamweiner/abletime) is a utility meant for calculating time spent on projects by
//! inspecting creation and modification timestamps of project files in a particular directory.

extern crate chrono;

use chrono::prelude::{DateTime, Local};
use chrono::Duration;

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;

/// Represents a version of a project and all its relevant metadata.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProjectFile {
    pub created_datetime: DateTime<Local>,
    pub modified_datetime: DateTime<Local>,
    pub time_spent: Duration,
    pub name: String,
}

impl fmt::Display for ProjectFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{: <21} {: <13} {}",
            self.created_datetime.format("%a %b %e %T"),
            format_duration(self.time_spent),
            self.name,
        )
    }
}

/// Build and return a vector of ProjectFiles in some directory, sorted by creation timestamp.
fn initialize_project_files(directory: String, project_file_suffix: String) -> Result<Vec<ProjectFile>, io::Error> {
    let mut project_files: Vec<ProjectFile> = Vec::new();

    // initialize project_files with all valid files found in provided directory
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let is_file = path.is_file();
        let is_valid_filetype = str::ends_with(path.to_str().unwrap(), &project_file_suffix);
        if is_file && is_valid_filetype {
            let project_file = ProjectFile {
                created_datetime: DateTime::<Local>::from(entry.metadata()?.created()?),
                modified_datetime: DateTime::<Local>::from(entry.metadata()?.modified()?),
                time_spent: Duration::zero(), // initialize with zero value, calculated after all files are initialized
                name: path.file_name().and_then(OsStr::to_str).unwrap().to_string(),
            };
            project_files.push(project_file);
        }
    }

    // sort by creation timestamp
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

        // for all but the most recent project file, prefer the delta between creation time and the
        // next version's creation time
        if i < project_files.len() - 1 {
            let time_spent_before_next_version =
                project_files[i + 1].created_datetime - project_files[i].created_datetime;
            if time_spent_before_next_version < max_time_between_saves {
                project_files[i].time_spent = time_spent_before_next_version
            }
        }
    }
}

/// Format durations as hh:mm:ss.ms.
fn format_duration(original_duration: Duration) -> String {
    let mut duration = original_duration;
    let hours = duration.num_hours();
    duration = duration - Duration::hours(hours);
    let minutes = duration.num_minutes();
    duration = duration - Duration::minutes(minutes);
    let seconds = duration.num_seconds();
    duration = duration - Duration::seconds(seconds);
    let milliseconds = duration.num_milliseconds();

    format!("{}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
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

/// Print to stdout a summary of time spent on each project file, as well as total time spent on the project.
pub fn print_project_summary(project_files: Vec<ProjectFile>) {
    if project_files.is_empty() {
        println!("No project files found");
        return;
    }

    println!("{: <21} {: <13} Name", "Start time", "Duration");
    let mut elapsed: Duration = Duration::zero();
    for project_file in project_files {
        println!("{}", project_file);
        elapsed = elapsed + project_file.time_spent;
    }
    println!("\nTotal project time\n{}", format_duration(elapsed));
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!("3:25:45.000", format_duration(Duration::seconds(12345)));
        assert_eq!("0:05:21.000", format_duration(Duration::seconds(321)));
        assert_eq!("0:00:00.522", format_duration(Duration::milliseconds(522)));
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
        };
        project_files.push(project_file_b);
        calculate_time_spent(&mut project_files, Duration::max_value());
        assert_eq!(project_files[0].time_spent, created_datetime_b - created_datetime_a);
        assert_eq!(project_files[1].time_spent, modified_datetime_b - created_datetime_b);
    }
}
