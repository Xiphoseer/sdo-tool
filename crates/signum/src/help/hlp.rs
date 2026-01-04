use std::collections::BTreeMap;
use std::path::Path;
use std::{fs, io};

use crate::chsets::encoding::AtariStrLines;

/// Represents a term in the help file.
#[derive(Default)]
pub struct Term {
    /// The title of the term.
    pub title: Option<String>,
    /// The content of the term, split into pages.
    pub content: Vec<String>,
    /// The aliases for the term.
    pub aliases: Vec<String>,
    /// The subterms of the term.
    pub subterms: BTreeMap<String, SubTerm>,
}

impl Term {
    /// Get a mutable reference to a subterm, if it exists.
    pub fn get_subterm(&mut self, subterm: &str) -> Option<&mut SubTerm> {
        self.subterms.get_mut(subterm)
    }

    /// Get or insert a subterm.
    pub fn get_or_insert_subterm(&mut self, subterm: &str) -> &mut SubTerm {
        self.subterms.entry(subterm.to_string()).or_default()
    }
}

/// Represents a subterm in the help file.
#[derive(Default)]
pub struct SubTerm {
    /// The title of the subterm.
    pub title: Option<String>,
    /// The content of the subterm, split into pages.
    pub content: Vec<String>,
    /// The aliases for the subterm.
    pub aliases: Vec<String>,
}

/// Signum 3/4 help file.
pub struct HelpFile {
    /// The terms in the help file.
    pub terms: BTreeMap<String, Term>,
}

impl Default for HelpFile {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpFile {
    /// Create a new instance of `HelpFile`.
    pub fn new() -> Self {
        HelpFile {
            terms: BTreeMap::new(),
        }
    }

    /// Get a mutable reference to a term, if it exists.
    pub fn get_term(&mut self, term: &str) -> Option<&mut Term> {
        self.terms.get_mut(term)
    }

    /// Get or insert a term.
    pub fn get_or_insert_term(&mut self, term: &str) -> &mut Term {
        self.terms.entry(term.to_string()).or_default()
    }

    /// Read help data from the format specified by the Signum documentation.
    pub fn from_reader<R: io::BufRead>(reader: R) -> io::Result<HelpFile> {
        let mut lines = AtariStrLines::new(reader);

        // Validate header
        let header = lines.next().transpose()?.unwrap_or_default();
        if !header.starts_with("HELP-FILE") {
            eprintln!("Not a valid Signum HLP file.");
            std::process::exit(1);
        }

        let mut help_file = HelpFile::new();
        let mut current_term: Option<String> = None;
        let mut current_subterm: Option<String> = None;

        for line in lines {
            let line = line?;
            if line.strip_prefix("@stop").is_some() {
                current_term = None;
                current_subterm = None;
                continue;
            }
            if line.strip_prefix("@@stop").is_some() {
                current_subterm = None;
                continue;
            }
            if let Some(rest) = line.strip_prefix("@+") {
                if let Some(term) = &current_term {
                    let alias = clean(rest);
                    if let Some(subterm) = &current_subterm {
                        help_file
                            .get_term(term)
                            .and_then(|t| t.get_subterm(subterm))
                            .unwrap()
                            .aliases
                            .push(alias);
                    } else {
                        help_file.get_term(term).unwrap().aliases.push(alias);
                    }
                }
                continue;
            }
            if let Some(rest) = line.strip_prefix("@@+") {
                if let Some(term) = &current_term {
                    if let Some(subterm) = &current_subterm {
                        let alias = clean(rest);
                        help_file
                            .get_term(term)
                            .and_then(|t| t.get_subterm(subterm))
                            .unwrap()
                            .aliases
                            .push(alias);
                    }
                }
                continue;
            }
            if let Some(rest) = line.strip_prefix("@@") {
                let subterm = clean(rest);
                if let Some(term) = &current_term {
                    help_file
                        .get_term(term)
                        .unwrap()
                        .get_or_insert_subterm(&subterm);
                    current_subterm = Some(subterm);
                }
                continue;
            }

            // Pagination marker: start a new page
            if line.strip_prefix("@/").is_some() {
                if let Some(term) = &current_term {
                    if let Some(subterm) = &current_subterm {
                        help_file
                            .get_term(term)
                            .and_then(|t| t.get_subterm(subterm))
                            .unwrap()
                            .content
                            .push(String::new());
                    } else {
                        help_file
                            .get_term(term)
                            .unwrap()
                            .content
                            .push(String::new());
                    }
                }
                continue;
            }

            if line.strip_prefix("@!").is_some() {
                // Comment line, ignore
            }

            if let Some(rest) = line.strip_prefix("@") {
                let term = clean(rest);
                help_file.get_or_insert_term(&term);
                current_term = Some(term);
                current_subterm = None;
                continue;
            }

            // Content lines: append to the current page
            if let Some(term) = &current_term {
                if let Some(subterm) = &current_subterm {
                    let sub = help_file
                        .get_term(term)
                        .and_then(|t| t.get_subterm(subterm))
                        .unwrap();
                    if sub.content.is_empty() {
                        sub.content.push(String::new());
                    }
                    if sub.content.len() == 1
                        && sub.content[0].is_empty()
                        && (line.starts_with('\t') || line.starts_with("  "))
                    {
                        sub.title = Some(line.trim().to_string());
                    }
                    if let Some(page) = sub.content.last_mut() {
                        if !page.is_empty() {
                            page.push('\n');
                        }
                        page.push_str(&line);
                    }
                } else {
                    let t = help_file.get_term(term).unwrap();
                    if t.content.is_empty() {
                        t.content.push(String::new());
                    }
                    if t.content.len() == 1
                        && t.content[0].is_empty()
                        && (line.starts_with('\t') || line.starts_with("  "))
                    {
                        t.title = Some(line.trim().to_string());
                    }
                    if let Some(page) = t.content.last_mut() {
                        if !page.is_empty() {
                            page.push('\n');
                        }
                        page.push_str(&line);
                    }
                }
            }
        }

        Ok(help_file)
    }

    /// Read a help file from the specified path.
    pub fn read_help_file<P: AsRef<Path>>(path: P) -> io::Result<HelpFile> {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        HelpFile::from_reader(reader)
    }
}

fn clean(input: &str) -> String {
    input.trim().replace("...", "…")
}
