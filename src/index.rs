use git2;
use std::fs::{remove_file, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::path_util::is_hidden;
use crate::repo::num_hours_since_last_commit;
use app_dirs::{app_dir, get_app_dir, AppDataType, AppInfo};
use dirs::home_dir;
use walkdir::{DirEntry, WalkDir};

const APP: AppInfo = AppInfo {
    name: "git-shell",
    author: "pka",
};
const CACHE_FILE: &'static str = "repos.txt";

type Repo = String;

pub struct Index {
    /// The base directory to walk when searching for git repositories.
    ///
    /// Default: $HOME.
    pub basedir: PathBuf,

    /// Path a cache file for git-global's usage.
    ///
    /// Default: `repos.txt` in the user's XDG cache directory.
    pub cache_file: PathBuf,
}

impl Index {
    pub fn new() -> Index {
        // Find the user's home directory.
        let homedir = home_dir().expect("Could not determine home directory.");
        // Set the options that aren't user-configurable.
        let cache_file = match get_app_dir(AppDataType::UserCache, &APP, "cache") {
            Ok(mut dir) => {
                dir.push(CACHE_FILE);
                dir
            }
            Err(_) => panic!("TODO: work without XDG"),
        };
        Index {
            basedir: homedir,
            cache_file: cache_file,
        }
    }

    /// Returns all known git repos, populating the cache first, if necessary.
    pub fn get_repos(&mut self) -> Vec<Repo> {
        if !self.has_cache() {
            let repos = self.find_repos();
            self.cache_repos(&repos);
        }
        self.get_cached_repos()
    }

    pub fn get_repos_by_last_commit(&mut self) -> Vec<Repo> {
        let repos = self.get_repos();
        let mut repos_with_last_commit = repos
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if let Ok(repo) = git2::Repository::open(&p) {
                    (i, num_hours_since_last_commit(&repo))
                } else {
                    (i, i64::max_value())
                }
            })
            .collect::<Vec<_>>();
        repos_with_last_commit.sort_by(|a, b| a.1.cmp(&b.1));
        // return sorted repo list
        repos_with_last_commit
            .iter()
            .map(|(i, _)| repos[*i].clone())
            .collect()
    }

    /// Clears the cache of known git repos, forcing a re-scan on the next
    /// `get_repos()` call.
    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        if self.has_cache() {
            remove_file(&self.cache_file).expect("Failed to delete cache file.");
        }
    }

    /// Returns boolean indicating if the cache file exists.
    fn has_cache(&self) -> bool {
        self.cache_file.exists()
    }

    /// Returns `true` if this entry should be included in scans.
    fn filter_dirs(&self, entry: &DirEntry) -> bool {
        entry.file_type().is_dir() && (!is_hidden(entry) || entry.file_name() == ".git")
    }

    /// Walks the configured base directory, looking for git repos.
    fn find_repos(&self) -> Vec<Repo> {
        let mut repos = Vec::new();
        println!(
            "Scanning for git repos under {}; this may take a while...",
            self.basedir.display()
        );
        for entry in WalkDir::new(&self.basedir)
            .follow_links(true)
            .same_file_system(true)
            .into_iter()
            .filter_entry(|e| self.filter_dirs(e))
        {
            if let Ok(entry) = entry {
                if entry.file_name() == ".git" {
                    let parent_path = entry.path().parent().expect("Could not determine parent.");
                    if let Some(path) = parent_path.to_str() {
                        if git2::Repository::open(&path).is_ok() {
                            repos.push(path.to_string());
                        }
                    }
                }
            }
        }
        repos
    }

    /// Writes the given repo paths to the cache file.
    fn cache_repos(&self, repos: &Vec<Repo>) {
        if !self.cache_file.as_path().exists() {
            // Try to create the cache directory if the cache *file* doesn't
            // exist; app_dir() handles an existing directory just fine.
            match app_dir(AppDataType::UserCache, &APP, "cache") {
                Ok(_) => (),
                Err(e) => panic!("Could not create cache directory: {}", e),
            }
        }
        let mut f = File::create(&self.cache_file).expect("Could not create cache file.");
        for repo in repos.iter() {
            match writeln!(f, "{}", repo) {
                Ok(_) => (),
                Err(e) => panic!("Problem writing cache file: {}", e),
            }
        }
    }

    /// Returns the list of repos found in the cache file.
    fn get_cached_repos(&self) -> Vec<Repo> {
        let mut repos = Vec::new();
        if self.cache_file.exists() {
            let f = File::open(&self.cache_file).expect("Could not open cache file.");
            let reader = BufReader::new(f);
            for line in reader.lines() {
                match line {
                    Ok(repo_path) => repos.push(repo_path),
                    Err(_) => (), // TODO: handle errors
                }
            }
        }
        repos
    }
}
