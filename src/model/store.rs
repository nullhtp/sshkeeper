use super::Connection;

pub struct ConnectionStore {
    connections: Vec<Connection>,
}

impl ConnectionStore {
    pub fn new(connections: Vec<Connection>) -> Self {
        Self { connections }
    }

    pub fn all(&self) -> &[Connection] {
        &self.connections
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Connection> {
        self.connections.iter().find(|c| c.id == id)
    }

    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut Connection> {
        self.connections.iter_mut().find(|c| c.id == id)
    }

    pub fn groups(&self) -> Vec<Option<String>> {
        let mut groups: Vec<Option<String>> = self
            .connections
            .iter()
            .map(|c| c.group.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        groups.sort_by(|a, b| match (a, b) {
            (Some(a), Some(b)) => a.cmp(b),
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, None) => std::cmp::Ordering::Equal,
        });
        groups
    }

    pub fn search(&self, query: &str) -> Vec<&Connection> {
        if query.is_empty() {
            return self.connections.iter().collect();
        }
        self.connections
            .iter()
            .filter(|c| c.matches_query(query))
            .collect()
    }

    pub fn add(&mut self, conn: Connection) {
        self.connections.push(conn);
        self.sort();
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let len = self.connections.len();
        self.connections.retain(|c| c.id != id);
        self.connections.len() < len
    }

    fn sort(&mut self) {
        self.connections.sort_by(|a, b| a.name.cmp(&b.name));
    }
}
