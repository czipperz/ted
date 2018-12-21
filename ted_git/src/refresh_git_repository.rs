use git2::*;
use git_mode::*;
use std::path::Path;
use ted_core::*;

pub fn refresh_git_repository(path: &Path, buffer: &mut Buffer) -> Result<(), String> {
    buffer.read_only = false;
    buffer.buffer_modes.push(git_mode());
    let mut buf = String::new();
    let repo = Repository::discover(path).map_err(|e| e.to_string())?;
    let workdir = repo.workdir().unwrap();
    buffer.name = BufferName {
        display_name: format!("*git* {}", workdir.file_name().unwrap().to_str().unwrap()),
        file_path: Some(workdir.to_path_buf()),
    };
    buf.push_str(&workdir.display().to_string());
    buf.push_str(": ");
    buf.push_str(&format!("{:?}", repo.state()));
    buf.push('\n');
    let statuses = repo.statuses(None).map_err(|e| e.to_string())?;
    if !statuses.is_empty() {
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        for status in statuses.iter() {
            use std::rc::Rc;
            let file = Rc::new(
                String::from_utf8_lossy(status.path_bytes())
                    .to_owned()
                    .to_string(),
            );
            let stat = status.status();
            if stat.is_index_new()
                | stat.is_index_modified()
                | stat.is_index_deleted()
                | stat.is_index_typechange()
                | stat.is_index_renamed()
            {
                staged.push((file.clone(), stat));
            }
            if stat.is_wt_new()
                | stat.is_wt_modified()
                | stat.is_wt_deleted()
                | stat.is_wt_typechange()
                | stat.is_wt_renamed()
            {
                if !stat.is_ignored() {
                    unstaged.push((file, stat));
                }
            }
        }

        if !staged.is_empty() {
            buf.push_str("\nStaged files: \n");
        }
        for (file, stat) in staged {
            if stat.is_index_new() {
                buf.push('N');
            } else if stat.is_index_modified() {
                buf.push('M');
            } else if stat.is_index_deleted() {
                buf.push('D');
            } else if stat.is_index_typechange() {
                buf.push('T');
            } else if stat.is_index_renamed() {
                buf.push('R');
            } else {
                unreachable!();
            }
            buf.push(' ');
            buf.push_str(&file);
            buf.push('\n');
        }

        if !unstaged.is_empty() {
            buf.push_str("\nUnstaged files: \n");
        }
        for (file, stat) in unstaged {
            if stat.is_wt_new() {
                buf.push('N');
            } else if stat.is_wt_modified() {
                buf.push('M');
            } else if stat.is_wt_deleted() {
                buf.push('D');
            } else if stat.is_wt_typechange() {
                buf.push('T');
            } else if stat.is_wt_renamed() {
                buf.push('R');
            } else {
                unreachable!();
            }
            buf.push(' ');
            buf.push_str(&file);
            buf.push('\n');
        }
    }
    buffer.clear()?;
    buffer.insert_str(0, &buf)?;
    buffer.erase_history();
    buffer.read_only = true;
    Ok(())
}
