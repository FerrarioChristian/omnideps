use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestManifestNode {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestManifestEdge {
    pub testid: String,
    pub source: String,
    pub sink: String,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestManifest {
    pub files: Vec<String>,
    pub nodes: Vec<TestManifestNode>,
    pub edges: Vec<TestManifestEdge>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestReportNode {
    pub name: String,
    pub kind: String,
    pub exists: bool,
    pub same_kind: bool,
    pub actual_kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestReportEdge {
    pub testid: String,
    pub source: String,
    pub sink: String,
    pub kind: String,
    pub source_exists: bool,
    pub sink_exists: bool,
    pub edge_exists: bool,
    pub same_kind: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestReport {
    pub files: Vec<String>,
    pub nodes: Vec<TestReportNode>,
    pub edges: Vec<TestReportEdge>,
    pub node_not_found_count: usize,
    pub edge_not_found_count: usize,
}

impl TestManifest {
    pub fn load(path: &str) -> anyhow::Result<TestManifest> {
        let yaml = std::fs::read(path)?;
        let manifest: TestManifest = serde_yaml::from_slice(&yaml)?;
        Ok(manifest)
    }

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }
}

impl TestReportNode {
    pub fn craft(
        node: &TestManifestNode,
        exists: bool,
        same_kind: bool,
        actual_kind: String,
    ) -> TestReportNode {
        TestReportNode {
            name: node.name.clone(),
            kind: node.kind.clone(),
            exists,
            same_kind,
            actual_kind,
        }
    }
}

impl TestReportEdge {
    pub fn craft(
        edge: &TestManifestEdge,
        source_exists: bool,
        sink_exists: bool,
        edge_exists: bool,
        same_kind: bool,
    ) -> TestReportEdge {
        TestReportEdge {
            testid: edge.testid.clone(),
            source: edge.source.clone(),
            sink: edge.sink.clone(),
            kind: edge.kind.clone(),
            source_exists,
            sink_exists,
            edge_exists,
            same_kind,
        }
    }
}

impl TestReport {
    pub fn craft(manifest: &TestManifest) -> TestReport {
        TestReport {
            files: manifest.files.clone(),
            nodes: vec![],
            edges: vec![],
            node_not_found_count: 0,
            edge_not_found_count: 0,
        }
    }

    pub fn load(path: &str) -> anyhow::Result<TestReport> {
        let yaml = std::fs::read(path)?;
        let report: TestReport = serde_yaml::from_slice(&yaml)?;
        Ok(report)
    }

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    pub fn bool_to_markdown(value: bool) -> String {
        if value {
            "![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8)".to_string()
        } else {
            "![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8)".to_string()
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut markdown = String::new();
        markdown += "# Report\n";

        markdown += "## Files\n";
        for file in &self.files {
            markdown += &format!("- {}\n", file);
        }

        markdown += "\n## Nodes\n";
        markdown += "| Name | Kind | Node Exists | Kind Is Correct | Actual Kind |\n";
        markdown += "| ---- | ---- | ----------- | --------------- | ----------- |\n";
        for node in &self.nodes {
            markdown += &format!(
                "| {} | {} | {} | {} | {} |\n",
                node.name,
                node.kind,
                Self::bool_to_markdown(node.exists),
                Self::bool_to_markdown(node.same_kind),
                node.actual_kind
            );
        }

        markdown += "\n## Edges\n";
        markdown += "| Test Id | Source | Sink | Kind | Source Exists | Sink Exists | Edge Exists | Kind Is Correct |\n";
        markdown += "| ------- | ------ | ---- | ---- | ------------- | ----------- | ----------- | --------------- |\n";
        for edge in &self.edges {
            markdown += &format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                edge.testid,
                edge.source,
                edge.sink,
                edge.kind,
                Self::bool_to_markdown(edge.source_exists),
                Self::bool_to_markdown(edge.sink_exists),
                Self::bool_to_markdown(edge.edge_exists),
                Self::bool_to_markdown(edge.same_kind)
            );
        }

        markdown += "\n## Results \n";
        markdown += "| Count | Total | Found | Not Found | Error Rate |\n";
        markdown += "| ----- | ----- | ----- | --------- | ---------- |\n";
        markdown += &format!(
            "| {} | {} | {} | {} | {:.4} |\n",
            "Nodes",
            self.nodes.len(),
            self.nodes.len() - self.node_not_found_count,
            self.node_not_found_count,
            if self.nodes.is_empty() { 0.0 } else { (self.node_not_found_count as f64) / (self.nodes.len() as f64) }
        );
        markdown += &format!(
            "| {} | {} | {} | {} | {:.4} |\n",
            "Edges",
            self.edges.len(),
            self.edges.len() - self.edge_not_found_count,
            self.edge_not_found_count,
            if self.edges.is_empty() { 0.0 } else { (self.edge_not_found_count as f64) / (self.edges.len() as f64) }
        );

        markdown
    }

    pub fn save_to_markdown(&self, path: &str) -> anyhow::Result<()> {
        let markdown = self.to_markdown();
        std::fs::write(path, markdown)?;
        Ok(())
    }

    pub fn save_to_json(&self, path: &str) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
