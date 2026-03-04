#[cfg(test)]
mod tests {
    use std::env;

    use anyhow::Result;

    use crate::dependancy_graph::graph::DependancyGraph;

    #[test]
    fn dependancy_list_test_1() -> Result<()> {
        let mut current = env::current_dir()?;
        current.push("test_filesystem");
        current.push("dep_graph_test");

        let graph = DependancyGraph::create(&current)?;
        dbg!(&graph);

        let mut dep_list = graph.dependancy_list("dir1.Dep1");
        dep_list.sort();

        current.push("dir2");
        current.push("Dep2.java");
        let dir1 = current.clone();

        current.pop();
        current.pop();

        current.push("Test1.java");

        let dir2 = current.clone();

        let mut expected_list = vec![dir1, dir2];
        expected_list.sort();

        assert_eq!(expected_list, dep_list, "Paths do not match");

        return Ok(());
    }
    #[test]
    fn dependancy_list_test_2() -> Result<()> {
        let mut current = env::current_dir()?;
        current.push("test_filesystem");
        current.push("dep_graph_test");

        let graph = DependancyGraph::create(&current)?;

        let mut dep_list = graph.dependancy_list("dir2.Dep2");
        dep_list.sort();

        current.push("Test1.java");

        let dir2 = current.clone();

        let mut expected_list = vec![dir2];
        expected_list.sort();

        assert_eq!(expected_list, dep_list, "Paths do not match");

        return Ok(());
    }
}
