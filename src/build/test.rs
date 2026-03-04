#[cfg(test)]
mod test {

    use std::{
        env,
        time::{Duration, SystemTime},
    };

    use anyhow::Result;
    use filetime::FileTime;

    use crate::{
        build::find_stale_files::{files_to_recompile, find_modified_files},
        dependancy_graph::graph::DependancyGraph,
    };

    fn setup() -> Result<()> {
        let mut current = env::current_dir()?;
        current.push("test_filesystem");
        current.push("inc_build_test");

        let mut build = current.clone();
        build.push("build");
        let time: FileTime = FileTime::from(SystemTime::now() - Duration::from_millis(1000));
        filetime::set_file_mtime(build, time)?;

        let mut dep2 = current.clone();
        dep2.push("dir1");
        dep2.push("Dep2.java");

        filetime::set_file_mtime(dep2, FileTime::now())?;
        return Ok(());
    }
    #[test]
    fn incrimental_build_test_1() -> Result<()> {
        setup()?;

        let mut current = env::current_dir()?;
        current.push("test_filesystem/inc_build_test");

        let graph = DependancyGraph::create(&current)?;

        for (_key, entry) in graph.nodes.iter() {
            println!(" {}", entry.file_name,);
            for dep in &entry.dependancies {
                println!("  - {}", dep);
            }
            println!("");
        }

        let mut build = current.clone();
        build.push("build");

        let stale = find_modified_files(&build, &current)?;

        let mut test1 = current.clone();
        test1.push("dir1/Dep2.java");

        assert_eq!(
            vec![test1],
            stale,
            "Expected stale files does not match stale files"
        );

        let mut recompiled = files_to_recompile(graph, stale)?;
        recompiled.sort();

        let mut test1 = current.clone();
        test1.push("Test1.java");
        let mut dep1 = current.clone();
        dep1.push("dir1");
        dep1.push("Dep1.java");

        let mut dep2 = current.clone();
        dep2.push("dir1");
        dep2.push("Dep2.java");

        let mut dep3 = current.clone();
        dep3.push("dir2");
        dep3.push("Dep3.java");

        let mut dep4 = current.clone();
        dep4.push("dir1");
        dep4.push("Dep4.java");

        let mut expected_recompile = vec![test1, dep1, dep2, dep3, dep4];

        expected_recompile.sort();

        assert_eq!(
            expected_recompile, recompiled,
            "Expected recompile files does not match recompile files"
        );

        return Ok(());
    }
}
