use std::collections::HashMap;

use crate::maven_central::{
    MavenError, MavenId,
    pom::{pom::MavenPom, pom_list::MavenDependancyList},
};

impl MavenPom {
    pub fn deserialize(
        id: &MavenId,
        content: &str,
    ) -> Result<MavenPom, MavenError> {
        let mut pom: MavenPom = quick_xml::de::from_str(content)?;
        log::info!("Successfully parsed POM {}", id);

        pom.group_id = id.group.to_string();
        pom.version = id.version.to_string();

        // creates the dependency_management_map
        pom.dependency_management_map = match &pom.dependency_management {
            Some(boms) => boms
                .dependencies
                .dependency
                .iter()
                .map(|bom| {
                    let hash =
                        MavenDependancyList::hash_maven_bom_id(&bom.group_id, &bom.artifact_id);

                    (
                        hash,
                        bom.version
                            .as_ref()
                            .expect("bom does not have version")
                            .clone(),
                    )
                })
                .collect(),
            None => HashMap::new(),
        };

        pom.properties.map.extend(default_properties_map(&pom));

        Ok(pom)
    }
}

fn default_properties_map(pom: &MavenPom) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();

    // common built-in project properties
    map.insert("project.groupId".to_string(), pom.group_id.clone());
    map.insert("pom.groupId".to_string(), pom.group_id.clone());
    map.insert("groupId".to_string(), pom.group_id.clone());

    map.insert("project.artifactId".to_string(), pom.artifact_id.clone());
    map.insert("pom.artifactId".to_string(), pom.artifact_id.clone());
    map.insert("artifactId".to_string(), pom.artifact_id.clone());

    map.insert("project.version".to_string(), pom.version.clone());
    map.insert("pom.version".to_string(), pom.version.clone());
    map.insert("version".to_string(), pom.version.clone());

    // parent properties
    if let Some(parent) = &pom.parent {
        map.insert(
            "project.parent.groupId".to_string(),
            parent.group_id.clone(),
        );
        map.insert(
            "project.parent.artifactId".to_string(),
            parent.artifact_id.clone(),
        );
        map.insert("project.parent.version".to_string(), parent.version.clone());
        if let Some(rel) = &parent.relative_path {
            map.insert("project.parent.relativePath".to_string(), rel.clone());
        }
    }

    map
}
