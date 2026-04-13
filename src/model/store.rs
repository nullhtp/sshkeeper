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

    pub fn all_mut(&mut self) -> &mut Vec<Connection> {
        &mut self.connections
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Connection> {
        self.connections.iter().find(|c| c.id == id)
    }

    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut Connection> {
        self.connections.iter_mut().find(|c| c.id == id)
    }

    pub fn filter_by_group(&self, group: &str) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|c| c.group.as_deref() == Some(group))
            .collect()
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

    pub fn has_duplicate(&self, host: &str, user: Option<&str>, port: u16) -> bool {
        self.connections.iter().any(|c| {
            c.host == host && c.user.as_deref() == user && c.port == port
        })
    }

    fn sort(&mut self) {
        self.connections.sort_by(|a, b| a.name.cmp(&b.name));
    }
}
