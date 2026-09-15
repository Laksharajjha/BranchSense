//! Dataset generator for `BranchSense`.

use branchsense_evaluation::model::EvaluationCase;
use branchsense_semantic::{
    DatasetSchemaVersion, EvalOutcome, EvalRepositoryIdentity, EvalRevision,
};
use std::env;
use std::error::Error;
use std::path::Path;
use std::process::Command;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn git(repo_path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!("git command failed: {args:?}").into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[allow(clippy::too_many_lines)]
fn extract_cases(repo_path: &Path, limit: usize) -> Result<Vec<EvaluationCase>> {
    let repo_url = match git(repo_path, &["remote", "get-url", "origin"]) {
        Ok(url) => url,
        Err(_) => "local-repo".to_string(),
    };
    let repo_id = EvalRepositoryIdentity::new(repo_url.clone(), Some(repo_url));

    let merges_output =
        git(repo_path, &["log", "--merges", "--format=%H %P", "-n", &limit.to_string()])?;
    let mut cases = Vec::new();

    for line in merges_output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let merge_commit = parts[0];
        let parent1 = parts[1]; // Branch B
        let parent2 = parts[2]; // Branch A

        let Ok(merge_base) = git(repo_path, &["merge-base", parent1, parent2]) else {
            continue;
        };

        // Detect textual conflict
        let mut textual_conflict = None;
        if let Ok(output) = Command::new("git")
            .current_dir(repo_path)
            .args(["merge-tree", &merge_base, parent1, parent2])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("<<<<<<<")
                || stdout.contains("changed in both")
                || stdout.contains("CONFLICT")
            {
                textual_conflict = Some(true);
            } else {
                textual_conflict = Some(false);
            }
        }

        let outcome =
            EvalOutcome::new().with_textual_merge_conflict(textual_conflict.unwrap_or(false));
        // Other outcomes left as None since we don't have CI labels.

        let case_id = format!("{}-{}", repo_id.id(), merge_commit);
        let case = EvaluationCase::new(
            case_id,
            DatasetSchemaVersion::current(),
            repo_id.clone(),
            EvalRevision::new(merge_base.clone()),
            EvalRevision::new(parent2),
            EvalRevision::new(parent1),
            EvalRevision::new(merge_base),
            outcome,
        );
        cases.push(case);
    }

    cases.sort_by(|a, b| a.case_id().cmp(b.case_id()));
    Ok(cases)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <repo-path>", args[0]);
        std::process::exit(1);
    }
    let repo_path = Path::new(&args[1]);
    let limit = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(50);

    let cases = extract_cases(repo_path, limit)?;
    for case in cases {
        let json = serde_json::to_string(&case)?;
        println!("{json}");
    }

    Ok(())
}
