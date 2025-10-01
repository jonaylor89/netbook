use color_eyre::Result;
use std::path::{Path, PathBuf};

/// Discovers or creates a collection file in the current directory
pub fn discover_collection() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    // Priority 1: .netbook/collection.json (project-specific)
    let netbook_dir_collection = current_dir.join(".netbook").join("collection.json");
    if netbook_dir_collection.exists() {
        return Ok(netbook_dir_collection);
    }

    // Priority 2: netbook.json (simple approach)
    let simple_collection = current_dir.join("netbook.json");
    if simple_collection.exists() {
        return Ok(simple_collection);
    }

    // Priority 3: Return default path for new collection
    Ok(netbook_dir_collection)
}

/// Gets the .netbook directory path for the given collection
pub fn get_netbook_dir(collection_path: &Path) -> PathBuf {
    if let Some(parent) = collection_path.parent() {
        if parent.ends_with(".netbook") {
            return parent.to_path_buf();
        }
        parent.join(".netbook")
    } else {
        PathBuf::from(".netbook")
    }
}

/// Creates a new collection with an example request
pub fn create_initial_collection(path: &Path) -> Result<()> {
    let example_collection = serde_json::json!([
        {
            "name": "Example GET Request",
            "method": "GET",
            "url": "https://jsonplaceholder.typicode.com/users/1",
            "headers": {
                "Accept": "application/json"
            },
            "notes": "Example request - edit this or delete it and add your own!"
        },
        {
            "name": "Example POST Request",
            "method": "POST",
            "url": "https://jsonplaceholder.typicode.com/posts",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "title": "My Title",
                "body": "My content",
                "userId": 1
            },
            "notes": "Example POST with JSON body"
        }
    ]);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(&example_collection)?;
    std::fs::write(path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_get_netbook_dir() {
        let collection = PathBuf::from("/home/user/project/.netbook/collection.json");
        let netbook_dir = get_netbook_dir(&collection);
        assert_eq!(netbook_dir, PathBuf::from("/home/user/project/.netbook"));

        let simple = PathBuf::from("/home/user/project/netbook.json");
        let netbook_dir2 = get_netbook_dir(&simple);
        assert_eq!(netbook_dir2, PathBuf::from("/home/user/project/.netbook"));
    }

    #[test]
    fn test_create_initial_collection() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".netbook").join("collection.json");

        create_initial_collection(&path).unwrap();
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Example GET Request"));
        assert!(content.contains("Example POST Request"));
    }
}
